use std::collections::BTreeMap;
use std::sync::Arc;
use dash_sdk::platform::{DocumentQuery, Fetch, FetchMany};
use dapi_grpc::platform::v0::get_documents_request::get_documents_request_v0::Start;
use dash_sdk::platform::proto::GetDataContractRequest;
use dash_sdk::Sdk;
use dpp::bincode::config::Limit;
use dpp::data_contract::DataContract;
use dpp::document::{Document, DocumentV0Getters};
use drive::query::{ordering::OrderClause, conditions::WhereClause, conditions::WhereOperator};
use platform_value::{types::identifier::Identifier, IdentifierBytes32, Value};
use tokio::runtime::{Builder, Runtime};
use crate::config::{Config, EntryPoint};
use crate::logs::setup_logs;
use crate::sdk::{create_dash_sdk_using_core_testnet, create_dash_sdk_using_single_evonode};
use crate::sdk::DashSdk;
use dash_sdk::Error;
use drive_proof_verifier::types::Documents;
use rs_dapi_client::{DapiClientError, RequestSettings};
use rs_dapi_client::transport::BoxFuture;
use dpp::version::LATEST_PLATFORM_VERSION;
use dpp::data_contract::accessors::v0::DataContractV0Getters;
use dpp::document::serialization_traits::DocumentPlatformConversionMethodsV0;
use crate::provider::Cache;

#[ferment_macro::export]
pub fn document_to_string(document: Document)-> String {
    document.to_string()
}

#[ferment_macro::export]
pub enum StartPoint {
    StartAfter(Vec<u8>),
    StartAt(Vec<u8>),
}

impl Into<Start> for StartPoint {
    fn into(self) -> Start {
        match self {
            StartPoint::StartAt(v) => Start::StartAt(v),
            StartPoint::StartAfter(v) => Start::StartAfter(v)
        }
    }
}

fn fetch_documents_with_retry(
    sdk: Arc<Sdk>,  // No need for a reference; pass the Arc by value
    data_contract_cache: Arc<Cache<Identifier, DataContract>>,
    query: DocumentQuery,
    request_settings: RequestSettings,
    retries_left: usize,
) -> BoxFuture<'static, Result<Documents, Error>> {
    // Clone the Arc<Sdk> here to ensure it's owned by the future
    Box::pin(async move {
        match Document::fetch_many_with_settings(&sdk, query.clone(), request_settings).await {
            Ok(documents) => Ok(documents),
            Err(error) => {
                if retries_left > 1 {
                    if error.to_string().contains("contract not found error: contract not found when querying from value with contract info") {
                        if (data_contract_cache.get(&query.data_contract.id()) != None) {
                            return fetch_documents_with_retry(sdk, data_contract_cache, query, request_settings, retries_left - 1).await;
                        }
                    }
                }
                Err(error)
            }
        }
    })
}

#[ferment_macro::export]
pub fn fetch_documents_with_query_and_sdk(
                                  rust_sdk: *mut DashSdk,
                                  data_contract_id: Identifier,
                                  document_type: String,
                                  where_clauses: Vec<WhereClause>,
                                  order_clauses: Vec<OrderClause>,
                                  limit: u32,
                                  start: Option<StartPoint>
) -> Result<Vec<Document>, String> {
    let rt = unsafe { (*rust_sdk).get_runtime() };

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        let sdk = unsafe { (*rust_sdk).get_sdk() };

        tracing::warn!("using existing data contract id and fetching...");

        let contract = match unsafe { (*rust_sdk).get_data_contract(&data_contract_id) } {
            Some(data_contract) => data_contract.clone(),
            None => {
                let request_settings = unsafe { (*rust_sdk).get_request_settings() };
                match (DataContract::fetch_with_settings(&sdk, data_contract_id.clone(), request_settings)
                         .await) {
                    Ok(Some(data_contract)) => {
                        unsafe { (*rust_sdk).add_data_contract(&data_contract); };
                        Arc::new(data_contract)
                    },
                    Ok(None) => return Err("data contract not found".to_string()),
                    Err(e) => return Err(e.to_string())
                }
            }
        };

        tracing::warn!("contract_fetch_result: {:?}", contract);

        tracing::warn!("fetching many...");
        // Fetch multiple documents so that we get document ID
        let mut all_docs_query =
            DocumentQuery::new(Arc::clone(&contract), &document_type)
                .expect("create SdkDocumentQuery");
        for wc in where_clauses {
            all_docs_query = all_docs_query.with_where(wc);
        }
        for oc in order_clauses {
            all_docs_query = all_docs_query.with_order_by(oc);
        }
        all_docs_query.limit = limit;
        all_docs_query.start = match start {
            Some(s) => Some(s.into()),
            None => None
        };
        let settings = unsafe { (*rust_sdk).get_request_settings() };
        tracing::warn!("fetching many... query created");
        let extra_retries = match settings.retries {
            Some(retries) => retries,
            None => 5 as usize
        };
        let data_contract_cache = unsafe { (*rust_sdk).data_contract_cache.clone() };
        match fetch_documents_with_retry(sdk.clone(), data_contract_cache, all_docs_query, settings, extra_retries).await {
            Ok(docs) => {
                tracing::warn!("convert to Vec");
                let into_vec = |map: BTreeMap<Identifier, Option<Document>>| {
                    map.into_iter()
                        .filter_map(|(_key, value)| value)
                        .collect::<Vec<Document>>()
                };

                Ok(into_vec(docs))
            }
            Err(e) => Err(e.to_string())
        }
    })
}

#[ferment_macro::export]
pub unsafe fn fetch_documents_with_query_and_sdk2(
    rust_sdk: *mut DashSdk,
    contract_id: Identifier,
    document_type: String,
    where_clauses: Vec<WhereClause>,
    order_clauses: Vec<OrderClause>,
    limit: u32,
    start: Option<StartPoint>
) -> Result<Vec<Document>, String> {
    let rt = (*rust_sdk).get_runtime();

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        let sdk = (*rust_sdk).get_sdk();

        let data_contract_id = contract_id;
        tracing::warn!("using existing data contract id and fetching...");

        let contract = match ((*rust_sdk).get_data_contract(&contract_id)) {
            Some(data_contract) => data_contract.clone(),
            None => {
                match (DataContract::fetch(&sdk, data_contract_id.clone())
                    .await) {
                    Ok(Some(data_contract)) => {
                        unsafe { (*rust_sdk).add_data_contract(&data_contract); };
                        Arc::new(data_contract)
                    },
                    Ok(None) => return Err("data contract not found".to_string()),
                    Err(e) => return Err(e.to_string())
                }
            }
        };

        tracing::warn!("contract_fetch_result: {:?}", contract);


        tracing::warn!("fetching many...");
        // Fetch multiple documents so that we get document ID
        let mut all_docs_query =
            DocumentQuery::new(Arc::clone(&contract), &document_type)
                .expect("create SdkDocumentQuery");
        for wc in where_clauses {
            all_docs_query = all_docs_query.with_where(wc);
        }
        for oc in order_clauses {
            all_docs_query = all_docs_query.with_order_by(oc);
        }
        all_docs_query.limit = limit;
        all_docs_query.start = match start {
            Some(s) => Some(s.into()),
            None => None
        };
        let settings = unsafe { (*rust_sdk).get_request_settings() };
        tracing::warn!("fetching many... query created");
        let docs = Document::fetch_many_with_settings(&sdk, all_docs_query, settings)
            .await;
        match docs {
            Ok(docs) => {
                tracing::warn!("convert to Vec");
                let into_vec = |map: BTreeMap<Identifier, Option<Document>>| {
                    map.into_iter()
                        .filter_map(|(_key, value)| value)
                        .collect::<Vec<Document>>()
                };

                Ok(into_vec(docs))
            }
            Err(e) => Err(e.to_string())
        }
    })
}

// Contenders: 5
// Abstain: 0
// Lock: 0
// Identifier: 2GW4QGjKj7zJVtL8SYaGZxuCYbkRCSAVAm9R9JDTtDaX
// Serialized:AL58b+byMei1+Tsbf5maCTwfg28Jczy6Fg6LR4tB3DeSEtRBemB6s6voUDw0LXI5YYF9EzyMp0zHLYTs1XMMEKABAAcAAAGRYOWVHgAAAZFg5ZUeAAABkWDllR4ABnRlc3QxMQZ0ZXN0MTEBBGRhc2gEZGFzaAAhARLUQXpgerOr6FA8NC1yOWGBfRM8jKdMxy2E7NVzDBCgAQA=
// Votes: 0
// Identifier: 2Phpq9yHi97dSookJery5xo2e3HARxGbqKaDDSSCFZki
// Serialized:AHDmT7mDZwcKgFdSEp/7we2TDX7NX5Q9mgflQnpWEhE7FKygFLCHkgJMkAHXRSeZmNMNf+LmA16iBU196pS+UScBAAcAAAGRXZuz7QAAAZFdm7PtAAABkV2bs+0ABnRlc3QxMQZ0ZXN0MTEBBGRhc2gEZGFzaAAhARSsoBSwh5ICTJAB10UnmZjTDX/i5gNeogVNfeqUvlEnAQA=
// Votes: 0
// Identifier: CrAJrjAyy7L1ShDyvKxHRJ1Cp2XfqdVHqFhfSZLi9QdY
// Serialized:AJUAiE5IM4P/TPA4BQBSumSFvbUMmUIDJYpoTZyX4TF2sAiKULrGNc7AEM66uXW6D8oCihs3wZ2lzYxjuSNyFrMBAAcAAAGRYNznfwAAAZFg3Od/AAABkWDc538ABnRlc3QxMQZ0ZXN0MTEBBGRhc2gEZGFzaAAhAbAIilC6xjXOwBDOurl1ug/KAoobN8Gdpc2MY7kjchazAQA=
// Votes: 0
// Identifier: DRa5SomSnh8cENfu3gLWAd2UuNNQKuYEyHakHk7oKivm
// Serialized:AOLKw44CYT2J/bjXSuyObfue6V4/kdvTBn8tb6m+7W2iuJeVGJgxeHXOiwg5e+Zzc7BPmIDjEVS3mb5DQLRGIiIBAAcAAAGRay/0NgAAAZFrL/Q2AAABkWsv9DYABnRlc3QxMQZ0ZXN0MTEBBGRhc2gEZGFzaAAhAbiXlRiYMXh1zosIOXvmc3OwT5iA4xFUt5m+Q0C0RiIiAQA=
// Votes: 0
// Identifier: EeHNsyw3MTJGquJxR45K8Wnt7BvTCLyuxGDAgxHdoGnA
// Serialized:AGH4+kYLEEVx5P49R8qys8mejGccoym8xP537nFJKG1MyrTwEVcAzOVfnNN0jDdMkpGXzPCKainEbQEMSu+PuQcBAAcAAAGRXbiwhAAAAZFduLCEAAABkV24sIQABnRlc3QxMQZ0ZXN0MTEBBGRhc2gEZGFzaAAhAcq08BFXAMzlX5zTdIw3TJKRl8zwimopxG0BDErvj7kHAQA=
// Votes: 0
#[ferment_macro::export]
pub unsafe fn deserialize_document_sdk(
    rust_sdk: *mut DashSdk,
    bytes: Vec<u8>,
    data_contract_id: Identifier,
    document_type: String
) -> Result<Document, String> {

    let rt = (*rust_sdk).get_runtime();

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        let sdk = (*rust_sdk).get_sdk();

        tracing::warn!("using existing data contract id and fetching...");

        let cfg = Config::new();
        let sdk = cfg.setup_api().await;

        let contract = match ((*rust_sdk).get_data_contract(&data_contract_id)) {
            Some(data_contract) => data_contract.clone(),
            None => {
                match (DataContract::fetch(&sdk, data_contract_id.clone())
                    .await) {
                    Ok(Some(data_contract)) => Arc::new(data_contract),
                    Ok(None) => return Err("data contract not found".to_string()),
                    Err(e) => return Err(e.to_string())
                }
            }
        };

        Document::from_bytes(&bytes, contract.document_type_for_name(&document_type).unwrap(), LATEST_PLATFORM_VERSION)
            .or_else(|e| Err(format!("deserialization failed: {}", e.to_string())))
    })
}


// TODO: need to rewrite these tests
// #[test]
// fn docs_test() {
//     let contract_id = Identifier::from(dpns_contract::ID_BYTES);
//     let docs = documents_with_callbacks(contract_id, "domain".to_string(), 0, 0);
//
//     for document in docs {
//         // Use `document` here
//         tracing::info!("{:?}", document); // Assuming Document implements Debug
//     }
// }
//
// #[test]
// fn docs_query_test() {
//    //let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
//     let docs = dpns_domain_starts_with("dq-".to_string(), 0, 0);
//
//     for document in docs {
//         // Use `document` here
//         tracing::info!("{:?}", document); // Assuming Document implements Debug
//     }
// }

// #[test]
// fn docs_query_id_test() {
//     let contract_id = Identifier::from(dpns_contract::ID_BYTES);
//     let docs = dpns_domain_by_id(contract_id, 0, 0);
//
//     for document in docs {
//         // Use `document` here
//         tracing::info!("{:?}", document); // Assuming Document implements Debug
//     }
//}

// #[test]
// fn docs_full_query_test() {
//     let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
//     let docs_result = fetch_documents_with_query(contract_id, "domain".to_string(),
//                                                  vec![WhereClause {
//                                               field: "normalizedLabel".to_string(),
//                                               operator: WhereOperator::Equal,
//                                               value: Value::Null,
//                                           }],
//                                                  vec![],
//                                                  100,
//                                                  None,
//                                                  0, 0);
//
//     match docs_result {
//         Ok(docs) => {
//             tracing::info!("query results");
//             for document in docs {
//                 // Use `document` here
//                 tracing::info!("{:?}", document); // Assuming Document implements Debug
//             }
//         }
//         Err(e) => panic!("{}", e)
//     }
// }

#[test]
fn docs_full_query_sdk_test() {
    let mut sdk = create_dash_sdk_using_core_testnet();
    let contract_id = Identifier::from(dpns_contract::ID_BYTES);
    let docs_result = unsafe {
        fetch_documents_with_query_and_sdk(
            &mut sdk,
            contract_id,
            "domain".to_string(),
            vec![],
            vec![],
            100,
            None
        )
    };

    match docs_result {
        Ok(docs) => {
            tracing::info!("query results");
            for document in docs {
                // Use `document` here
                tracing::info!("{:?}", document); // Assuming Document implements Debug
            }
        }
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn docs_full_query_sdk2_test() {
    let mut sdk = create_dash_sdk_using_core_testnet();
    let contract_id = Identifier::from(dpns_contract::ID_BYTES);
    let docs_result = unsafe {
        fetch_documents_with_query_and_sdk2(
            &mut sdk,
            contract_id,
            "domain".to_string(),
            vec![],
            vec![],
            100,
            None
        )
    };

    match docs_result {
        Ok(docs) => {
            tracing::info!("query results");
            for document in docs {
                // Use `document` here
                tracing::info!("{:?}", document); // Assuming Document implements Debug
            }
        }
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn docs_startswith_query_sdk_test() {
    let mut sdk = create_dash_sdk_using_core_testnet();
    let contract_id = Identifier::from(dpns_contract::ID_BYTES);
    let docs_result = unsafe {
        fetch_documents_with_query_and_sdk(
            &mut sdk,
            contract_id,
            "domain".to_string(),
            vec![
                WhereClause { field: "normalizedLabel".into(), value: Value::Text("b0b1ee".into()), operator: WhereOperator::StartsWith },
                WhereClause { field: "normalizedParentDomainName".into(), value: Value::Text("dash".into()), operator: WhereOperator::Equal }
            ],
            vec![
                OrderClause { field: "normalizedLabel".into(), ascending: true }
            ],
            100,
            None
        )
    };

    match docs_result {
        Ok(docs) => {
            tracing::info!("query results: {}", docs.len());
            for document in docs {
                // Use `document` here
                tracing::info!("{:?}", document); // Assuming Document implements Debug
            }
        }
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn docs_startswith_query_sdk_using_single_node_test() {
    let mut sdk = create_dash_sdk_using_single_evonode("35.163.144.230".into(), 0, 0, true);
    let contract_id = Identifier::from(dpns_contract::ID_BYTES);
    let docs_result = unsafe {
        fetch_documents_with_query_and_sdk(
            &mut sdk,
            contract_id,
            "domain".to_string(),
            vec![
                WhereClause { field: "normalizedLabel".into(), value: Value::Text("b0b1ee".into()), operator: WhereOperator::StartsWith },
                WhereClause { field: "normalizedParentDomainName".into(), value: Value::Text("dash".into()), operator: WhereOperator::Equal }
            ],
            vec![
                OrderClause { field: "normalizedLabel".into(), ascending: true }
            ],
            100,
            None
        )
    };

    match docs_result {
        Ok(docs) => {
            tracing::info!("query results: {}", docs.len());
            for document in docs {
                // Use `document` here
                tracing::info!("{:?}", document); // Assuming Document implements Debug
            }
        }
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn docs_domain_query_sort_test() {
    let mut sdk = create_dash_sdk_using_core_testnet();
    let contract_id = Identifier::from(dpns_contract::ID_BYTES);
    let docs_result = unsafe {
        fetch_documents_with_query_and_sdk(
            &mut sdk,
            contract_id,
            "domain".to_string(),
            vec![
                WhereClause { field: "normalizedLabel".into(), value: Value::Text("tut".into()), operator: WhereOperator::StartsWith },
                WhereClause { field: "normalizedParentDomainName".into(), value: Value::Text("dash".into()), operator: WhereOperator::Equal }
            ],
            vec![
                OrderClause { field: "normalizedLabel".into(), ascending: true }
            ],
            100,
            None
        )
    };

    match docs_result {
        Ok(docs) => {
            tracing::info!("query results: {}", docs.len());
            for document in docs {
                match document {
                    Document::V0(document_v0) => {
                        // Use `document` here
                        tracing::info!("{:?}", document_v0.properties().get("normalizedLabel")); // Assuming Document implements Debug
                    }
                }
            }
        }
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn doc_deserialization_sdk_test() {
    let mut sdk = create_dash_sdk_using_core_testnet();

    unsafe {
        println!(
            "{:?}", deserialize_document_sdk(
                &mut sdk,
                base64::decode("AGH4+kYLEEVx5P49R8qys8mejGccoym8xP537nFJKG1MyrTwEVcAzOVfnNN0jDdMkpGXzPCKainEbQEMSu+PuQcBAAcAAAGRXbiwhAAAAZFduLCEAAABkV24sIQABnRlc3QxMQZ0ZXN0MTEBBGRhc2gEZGFzaAAhAcq08BFXAMzlX5zTdIw3TJKRl8zwimopxG0BDErvj7kHAQA=").unwrap(),
                Identifier::from(dpns_contract::ID_BYTES),
                "domain".into()
            ).unwrap()
        );
    }
}


pub const ADDRESS_LIST: [&str; 33] = [
    "34.214.48.68",
    "35.166.18.166",
    "35.165.50.126",
    "52.42.202.128",
    "52.12.176.90",
    "44.233.44.95",
    "35.167.145.149",
    "52.34.144.50",
    "44.240.98.102",
    "54.201.32.131",
    "52.10.229.11",
    "52.13.132.146",
    "44.228.242.181",
    "35.82.197.197",
    "52.40.219.41",
    "44.239.39.153",
    "54.149.33.167",
    "35.164.23.245",
    "52.33.28.47",
    "52.43.86.231",
    "52.43.13.92",
    "35.163.144.230",
    "52.89.154.48",
    "52.24.124.162",
    "44.227.137.77",
    "35.85.21.179",
    "54.187.14.232",
    "54.68.235.201",
    "52.13.250.182",
     "35.82.49.196",
    "44.232.196.6",
    "54.189.164.39",
    "54.213.204.85"
];

#[test]
fn check_all_nodes_test() {

    for address in ADDRESS_LIST {
        let mut sdk = create_dash_sdk_using_single_evonode(address.into(), 0, 0);
        let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
        let docs_result = unsafe {
            fetch_documents_with_query_and_sdk(
                &mut sdk,
                contract_id,
                "domain".to_string(),
                vec![
                    WhereClause { field: "normalizedLabel".into(), value: Value::Text("b0b1ee".into()), operator: WhereOperator::StartsWith },
                    WhereClause { field: "normalizedParentDomainName".into(), value: Value::Text("dash".into()), operator: WhereOperator::Equal }
                ],
                vec![
                    OrderClause { field: "normalizedLabel".into(), ascending: true }
                ],
                100,
                None
            )
        };

        match docs_result {
            Ok(docs) => {
                tracing::info!("{}: success", address);
            }
            Err(e) => { println!("{}: error/fail: {}", address, e) }
        }
    }
}
