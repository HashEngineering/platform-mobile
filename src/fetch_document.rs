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
use crate::config::{Config, DPNS_DATACONTRACT_ID, EntryPoint};
use crate::logs::setup_logs;
use crate::sdk::{create_dash_sdk, create_dash_sdk_using_single_evonode};
use crate::sdk::DashSdk;
use rs_dapi_client::DapiClientError;
#[ferment_macro::export]
pub fn get_document()-> Document {
    document_read()
}

#[ferment_macro::export]
pub fn document_to_string(document: Document)-> String {
    document.to_string()
}

#[ferment_macro::export]
pub fn get_documents(identifier: Identifier, document_type: String, q: u64, d: u64)-> Vec<Document> {
    documents_with_callbacks(identifier, document_type, q, d)
}

#[ferment_macro::export]
pub fn get_domain_document(identifier: Identifier, q: u64, d: u64)-> Vec<Document> {
    dpns_domain_by_id(identifier, q, d)
}

#[ferment_macro::export]
pub fn get_domain_document_starts_with(starts_with: String, q: u64, d: u64)-> Vec<Document> {
    dpns_domain_starts_with(starts_with, q, d)
}

// error[E0277]: the trait bound `dpp::document::Document: FFIConversion<dpp::document::Document>` is not satisfied
// --> src/fermented.rs:1:218446
// |
// 1 | ...ersion :: ffi_from (o) , | o | ferment_interfaces :: FFIConversion :: ffi_from_opt (o)) } unsafe fn ffi_to_const (obj : std :: collect...
// |                                   ---------------------------------------------------  ^ the trait `FFIConversion<dpp::document::Document>` is not implemented for `dpp::document::Document`
// |                                   |
// |                                   required by a bound introduced by this call

#[ferment_macro::export]
pub fn get_documents_tree(
    identifier: Identifier,
    document_type: String,
    q: u64,
    d: u64
) -> BTreeMap<Identifier, Option<Document>> {
    documents_with_callbacks_tree(identifier, document_type, q, d)
}

#[ferment_macro::export]
pub fn get_document_with_callbacks(quorum_public_key_callback: u64,
                                   data_contract_callback: u64
)-> Identifier {
    let it = document_read_with_callbacks(quorum_public_key_callback, data_contract_callback);
    match it {
        Document::V0(doc_v0) => doc_v0.owner_id,
        _ => Identifier::default()
    }
}

fn document_read() -> Document {
    setup_logs();

    //let rt = tokio::runtime::Runtime::new().expect("Failed to create a runtime");
    let rt = Builder::new_current_thread()
        .enable_all() // Enables all I/O and time drivers
        .build()
        .expect("Failed to create a runtime");


    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        let cfg = Config::new();
        let sdk = cfg.setup_api().await;

        let data_contract_id = cfg.existing_data_contract_id;
        tracing::warn!("using existing data contract id and fetching...");
        let contract = Arc::new(
            DataContract::fetch(&sdk, data_contract_id)
                .await
                .expect("fetch data contract")
                .expect("data contract not found"),
        );

        tracing::warn!("fetching many...");
        // Fetch multiple documents so that we get document ID
        let all_docs_query =
            DocumentQuery::new(Arc::clone(&contract), &cfg.existing_document_type_name)
                .expect("create SdkDocumentQuery");
        let first_doc = Document::fetch_many(&sdk, all_docs_query)
            .await
            .expect("fetch many documents")
            .pop_first()
            .expect("first item must exist")
            .1
            .expect("document must exist");

        // Now query for individual document
        let query = DocumentQuery::new(contract, &cfg.existing_document_type_name)
            .expect("create SdkDocumentQuery")
            .with_document_id(&first_doc.id());

        let doc = Document::fetch(&sdk, query)
            .await
            .expect("fetch document")
            .expect("document must be found");

        //assert_eq!(first_doc, doc);

        doc
    })
}

fn document_read_with_callbacks(quorum_public_key_callback: u64,
                                data_contract_callback: u64) -> Document {
    setup_logs();

    //let rt = tokio::runtime::Runtime::new().expect("Failed to create a runtime");
    let rt = Builder::new_current_thread()
        .enable_all() // Enables all I/O and time drivers
        .build()
        .expect("Failed to create a runtime");

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        let cfg = Config::new();
        let sdk = cfg.setup_api_with_callbacks(quorum_public_key_callback, data_contract_callback).await;

        let data_contract_id = cfg.existing_data_contract_id;
        tracing::warn!("using existing data contract id and fetching...");
        let contract = Arc::new(
            DataContract::fetch(&sdk, data_contract_id)
                .await
                .expect("fetch data contract")
                .expect("data contract not found"),
        );

        tracing::warn!("fetching many...");
        // Fetch multiple documents so that we get document ID
        let all_docs_query =
            DocumentQuery::new(Arc::clone(&contract), &cfg.existing_document_type_name)
                .expect("create SdkDocumentQuery");
        let first_doc = Document::fetch_many(&sdk, all_docs_query)
            .await
            .expect("fetch many documents")
            .pop_first()
            .expect("first item must exist")
            .1
            .expect("document must exist");

        // Now query for individual document
        let query = DocumentQuery::new(contract, &cfg.existing_document_type_name)
            .expect("create SdkDocumentQuery")
            .with_document_id(&first_doc.id());

        let doc = Document::fetch(&sdk, query)
            .await
            .expect("fetch document")
            .expect("document must be found");

        //assert_eq!(first_doc, doc);

        doc
    })
}

fn documents_with_callbacks(contract_id: Identifier,
                                document_type: String,
                                quorum_public_key_callback: u64,
                                data_contract_callback: u64) -> Vec<Document> {
    setup_logs();

    //let rt = tokio::runtime::Runtime::new().expect("Failed to create a runtime");
    let rt = Builder::new_current_thread()
        .enable_all() // Enables all I/O and time drivers
        .build()
        .expect("Failed to create a runtime");

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        let cfg = Config::new();
        let sdk = if quorum_public_key_callback != 0 {
            cfg.setup_api_with_callbacks(quorum_public_key_callback, data_contract_callback).await
        } else {
            cfg.setup_api().await
        };

        let data_contract_id = contract_id;
        tracing::warn!("using existing data contract id and fetching...");
        let contract = Arc::new(
            DataContract::fetch(&sdk, data_contract_id.clone())
                .await
                .expect("fetch data contract")
                .expect("data contract not found"),
        );

        tracing::warn!("fetching many...");
        // Fetch multiple documents so that we get document ID
        let all_docs_query =
            DocumentQuery::new(Arc::clone(&contract), &document_type)
                .expect("create SdkDocumentQuery");
        let docs = Document::fetch_many(&sdk, all_docs_query)
            .await
            .expect("fetch many documents");

        let into_vec = |map: BTreeMap<Identifier, Option<Document>>| {
            map.into_iter()
                .filter_map(|(_key, value)| value)
                .collect::<Vec<Document>>()
        };

        into_vec(docs)
    })
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
#[ferment_macro::export]
pub fn fetch_documents_with_query(contract_id: Identifier,
                            document_type: String,
                            where_clauses: Vec<WhereClause>,
                            order_clauses: Vec<OrderClause>,
                            limit: u32,
                            start: Option<StartPoint>,
                            quorum_public_key_callback: u64,
                            data_contract_callback: u64) -> Result<Vec<Document>, String> {
    setup_logs();

    //let rt = tokio::runtime::Runtime::new().expect("Failed to create a runtime");
    let rt = Builder::new_current_thread()
        .enable_all() // Enables all I/O and time drivers
        .build()
        .expect("Failed to create a runtime");

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        let cfg = Config::new();
        let sdk = if quorum_public_key_callback != 0 {
            cfg.setup_api_with_callbacks(quorum_public_key_callback, data_contract_callback).await
        } else {
            cfg.setup_api().await
        };

        let data_contract_id = contract_id;
        tracing::warn!("using existing data contract id and fetching...");
        let contract_result =
            DataContract::fetch(&sdk, data_contract_id.clone())
                .await;

        let contract = match contract_result {
            Ok(Some(data_contract)) => Arc::new(data_contract),
            Ok(None) => return Err("data contract not found".to_string()),
            Err(e) => return Err("fetch data contract error".to_string())
        };

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
        tracing::warn!("fetching many... query created");
        let docs = Document::fetch_many(&sdk, all_docs_query)
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

// #[ferment_macro::export]
// pub unsafe fn fetch_documents_with_query_and_sdk(
//                                   rust_sdk: Arc<RustSdk>,
//                                   contract_id: Identifier,
//                                   document_type: String,
//                                   where_clauses: Vec<WhereClause>,
//                                   order_clauses: Vec<OrderClause>,
//                                   limit: u32,
//                                   start: Option<StartPoint>
// ) -> Result<Vec<Document>, String> {
//     setup_logs();
//     tracing::warn!("sdk: {:?}", rust_sdk);
//
//     //let rt = tokio::runtime::Runtime::new().expect("Failed to create a runtime");
//     let rt = Arc::from_raw(rust_sdk.runtime as *const Runtime);
//
//     // Execute the async block using the Tokio runtime
//     rt.block_on(async {
//         let sdk = Arc::from_raw(rust_sdk.sdk_pointer as * const Sdk);
//
//         let data_contract_id = contract_id;
//         tracing::warn!("using existing data contract id and fetching...");
//         // let contract = Arc::new(
//         //     DataContract::fetch(&sdk, data_contract_id.clone())
//         //         .await
//         //         .expect("fetch data contract")
//         //         .expect("data contract not found"),
//         // );
//
//         let contract_fetch_result =
//             DataContract::fetch(&sdk, data_contract_id.clone())
//                 .await;
//         tracing::warn!("contract_fetch_result: {:?}", contract_fetch_result);
//         let contract_result = match contract_fetch_result {
//             Ok(contract) => contract,
//             Err(e) => return Err(e.to_string())
//         };
//         tracing::warn!("contract_result: {:?}", contract_result);
//
//         let contract = match contract_result {
//             Some(c) => Arc::new(c),
//             None => return Err("contract not found".to_string())
//         };
//
//         tracing::warn!("fetching many...");
//         // Fetch multiple documents so that we get document ID
//         let mut all_docs_query =
//             DocumentQuery::new(Arc::clone(&contract), &document_type)
//                 .expect("create SdkDocumentQuery");
//         for wc in where_clauses {
//             all_docs_query = all_docs_query.with_where(wc);
//         }
//         for oc in order_clauses {
//             all_docs_query = all_docs_query.with_order_by(oc);
//         }
//         all_docs_query.limit = limit;
//         all_docs_query.start = match start {
//             Some(s) => Some(s.into()),
//             None => None
//         };
//         tracing::warn!("fetching many... query created");
//         let docs = Document::fetch_many(&sdk, all_docs_query)
//             .await;
//         match docs {
//             Ok(docs) => {
//                 tracing::warn!("convert to Vec");
//                 let into_vec = |map: BTreeMap<Identifier, Option<Document>>| {
//                     map.into_iter()
//                         .filter_map(|(_key, value)| value)
//                         .collect::<Vec<Document>>()
//                 };
//
//                 Ok(into_vec(docs))
//             }
//             Err(e) => Err(e.to_string())
//         }
//     })
// }


// async fn fetch_documents_with_retry(sdk: Arc<Sdk>, query: &DocumentQuery, request_settings: RequestSettings, retries_left: u32) -> Result<Documents, Error> {
//     match Document::fetch_many_with_settings(&sdk, query.clone(), request_settings).await {
//         Ok(documents) => Ok(documents),
//         Err(error) => {
//             if retries_left > 1 {
//                 if error.to_string().contains("contract not found error: contract not found when querying from value with contract info") {
//                     return fetch_documents_with_retry(sdk, query, request_settings, retries_left - 1).await
//                 }
//             }
//             Err(error)
//         }
//     }
// }

// fn fetch_documents_with_retry(
//     sdk: &Arc<Sdk>,
//     query: DocumentQuery,  // query is cloned, so no need to borrow it
//     request_settings: RequestSettings,
//     retries_left: usize,
// ) -> BoxFuture<'static, Result<Documents, Error>> {
//     Box::pin(async move {
//         match Document::fetch_many_with_settings(&sdk, query.clone(), request_settings).await {
//             Ok(documents) => Ok(documents),
//             Err(error) => {
//                 if retries_left > 1 {
//                     if error.to_string().contains("contract not found error: contract not found when querying from value with contract info") {
//                         return fetch_documents_with_retry(sdk, query, request_settings, retries_left - 1).await;
//                     }
//                 }
//                 Err(error)
//             }
//         }
//     })
// }


#[ferment_macro::export]
pub unsafe fn fetch_documents_with_query_and_sdk(
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
                let request_settings = (*rust_sdk).get_request_settings();
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
use dpp::version::LATEST_PLATFORM_VERSION;
use dpp::data_contract::accessors::v0::DataContractV0Getters;
use dpp::document::serialization_traits::DocumentPlatformConversionMethodsV0;
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

fn dpns_domain_starts_with(starts_with: String,
                            quorum_public_key_callback: u64,
                            data_contract_callback: u64) -> Vec<Document> {
    setup_logs();

    //let rt = tokio::runtime::Runtime::new().expect("Failed to create a runtime");
    let rt = Builder::new_current_thread()
        .enable_all() // Enables all I/O and time drivers
        .build()
        .expect("Failed to create a runtime");

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        let cfg = Config::new();
        let sdk = if quorum_public_key_callback != 0 {
            cfg.setup_api_with_callbacks(quorum_public_key_callback, data_contract_callback).await
        } else {
            cfg.setup_api().await
        };

        let data_contract_id = cfg.existing_data_contract_id;
        tracing::warn!("using existing data contract id and fetching...");
        let contract = Arc::new(
            DataContract::fetch(&sdk, data_contract_id.clone())
                .await
                .expect("fetch data contract")
                .expect("data contract not found"),
        );

        tracing::warn!("fetching many...starts with");
        // Fetch multiple documents so that we get document ID
        let all_docs_query =
            DocumentQuery::new(Arc::clone(&contract), &cfg.existing_document_type_name)
                .expect("create SdkDocumentQuery")
                .with_where(WhereClause {
                    field: "normalizedLabel".to_string(),
                    operator: WhereOperator::StartsWith,
                    value: Value::Text(starts_with.to_string()),
                })
                .with_where(WhereClause {
                    field: "normalizedParentDomainName".to_string(),
                    operator: WhereOperator::Equal,
                    value: Value::Text("dash".to_string()),
                }).with_order_by(OrderClause {
                    field: "normalizedLabel".to_string(),
                    ascending: true,
                });
        let docs = Document::fetch_many(&sdk, all_docs_query)
            .await
            .expect("fetch many documents");

        let into_vec = |map: BTreeMap<Identifier, Option<Document>>| {
            map.into_iter()
                .filter_map(|(_key, value)| value)
                .collect::<Vec<Document>>()
        };

        into_vec(docs)
    })
}

fn dpns_domain_by_id(unique_id: Identifier,
      quorum_public_key_callback: u64,
      data_contract_callback: u64) -> Vec<Document> {
    setup_logs();

    //let rt = tokio::runtime::Runtime::new().expect("Failed to create a runtime");
    let rt = Builder::new_current_thread()
        .enable_all() // Enables all I/O and time drivers
        .build()
        .expect("Failed to create a runtime");

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        let cfg = Config::new();
        let sdk = if quorum_public_key_callback != 0 {
            cfg.setup_api_with_callbacks(quorum_public_key_callback, data_contract_callback).await
        } else {
            cfg.setup_api().await
        };

        let data_contract_id = cfg.existing_data_contract_id;
        tracing::warn!("using existing data contract id and fetching...");
        let contract = Arc::new(
            DataContract::fetch(&sdk, data_contract_id.clone())
                .await
                .expect("fetch data contract")
                .expect("data contract not found"),
        );

        tracing::warn!("fetching many...by id");
        // Fetch multiple documents so that we get document ID
        let all_docs_query =
            DocumentQuery::new(Arc::clone(&contract), &cfg.existing_document_type_name)
                .expect("create SdkDocumentQuery")
                .with_where(WhereClause {
                    field: "records.identity".to_string(),
                    operator: WhereOperator::Equal,
                    value: Value::from(unique_id),
                });
        let docs = Document::fetch_many(&sdk, all_docs_query)
            .await
            .expect("fetch many documents");

        let into_vec = |map: BTreeMap<Identifier, Option<Document>>| {
            map.into_iter()
                .filter_map(|(_key, value)| value)
                .collect::<Vec<Document>>()
        };

        into_vec(docs)
    })
}

fn documents_with_callbacks_tree(contract_id: Identifier,
                            document_type: String,
                            quorum_public_key_callback: u64,
                            data_contract_callback: u64) -> BTreeMap<Identifier, Option<Document>> {
    setup_logs();

    //let rt = tokio::runtime::Runtime::new().expect("Failed to create a runtime");
    let rt = Builder::new_current_thread()
        .enable_all() // Enables all I/O and time drivers
        .build()
        .expect("Failed to create a runtime");

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        let cfg = Config::new();
        let sdk = if quorum_public_key_callback != 0 {
            cfg.setup_api_with_callbacks(quorum_public_key_callback, data_contract_callback).await
        } else {
            cfg.setup_api().await
        };

        let data_contract_id = contract_id;
        tracing::warn!("using existing data contract id and fetching...");
        let contract = Arc::new(
            DataContract::fetch(&sdk, data_contract_id.clone())
                .await
                .expect("fetch data contract")
                .expect("data contract not found"),
        );

        tracing::warn!("fetching many...");
        // Fetch multiple documents so that we get document ID
        let all_docs_query =
            DocumentQuery::new(Arc::clone(&contract), &document_type)
                .expect("create SdkDocumentQuery");
        let docs = Document::fetch_many(&sdk, all_docs_query)
            .await
            .expect("fetch many documents");

        docs
    })
}

// #[ferment_macro::export]
// pub fn deserialize_document(bytes: Vec<u8>) -> Result<Document, String> {
//
//
//
//     let rt = Builder::new_current_thread()
//         .enable_all() // Enables all I/O and time drivers
//         .build()
//         .expect("Failed to create a runtime");
//
//     // Execute the async block using the Tokio runtime
//     rt.block_on(async {
//
//         let cfg = Config::new();
//         let sdk = cfg.setup_api().await;
//         let data_contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
//
//         let contract =
//             DataContract::fetch(&sdk, data_contract_id)
//                 .await
//                 .expect("fetch data contract")
//                 .expect("data contract not found");
//
//         Document::from_bytes(&bytes, contract.document_type_for_name("domain".into()).unwrap(), LATEST_PLATFORM_VERSION)
//             .or_else(|e| Err(format!("deserialization failed: {}", e.to_string())))
//     })
// }
//
//
//
//
// #[test]
// fn doc_deserialization_test() {
//     println!(
//         "{:?}", deserialize_document(
//             base64::decode("AGH4+kYLEEVx5P49R8qys8mejGccoym8xP537nFJKG1MyrTwEVcAzOVfnNN0jDdMkpGXzPCKainEbQEMSu+PuQcBAAcAAAGRXbiwhAAAAZFduLCEAAABkV24sIQABnRlc3QxMQZ0ZXN0MTEBBGRhc2gEZGFzaAAhAcq08BFXAMzlX5zTdIw3TJKRl8zwimopxG0BDErvj7kHAQA=").unwrap()
//         ).unwrap()
//     );
// }

#[test]
fn docs_test() {
    let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
    let docs = documents_with_callbacks(contract_id, "domain".to_string(), 0, 0);

    for document in docs {
        // Use `document` here
        tracing::info!("{:?}", document); // Assuming Document implements Debug
    }
}

#[test]
fn docs_query_test() {
   //let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
    let docs = dpns_domain_starts_with("dq-".to_string(), 0, 0);

    for document in docs {
        // Use `document` here
        tracing::info!("{:?}", document); // Assuming Document implements Debug
    }
}

#[test]
fn docs_query_id_test() {
    let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
    let docs = dpns_domain_by_id(contract_id, 0, 0);

    for document in docs {
        // Use `document` here
        tracing::info!("{:?}", document); // Assuming Document implements Debug
    }
}

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
    let mut sdk = create_dash_sdk(0, 0);
    let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
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
    let mut sdk = create_dash_sdk(0, 0);
    let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
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
    let mut sdk = create_dash_sdk(0, 0);
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
    let mut sdk = create_dash_sdk_using_single_evonode("35.163.144.230".into(),0, 0);
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
    let mut sdk = create_dash_sdk(0, 0);
    let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
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
    let mut sdk = create_dash_sdk(0, 0);

    unsafe {
        println!(
            "{:?}", deserialize_document_sdk(
                &mut sdk,
                base64::decode("AGH4+kYLEEVx5P49R8qys8mejGccoym8xP537nFJKG1MyrTwEVcAzOVfnNN0jDdMkpGXzPCKainEbQEMSu+PuQcBAAcAAAGRXbiwhAAAAZFduLCEAAABkV24sIQABnRlc3QxMQZ0ZXN0MTEBBGRhc2gEZGFzaAAhAcq08BFXAMzlX5zTdIw3TJKRl8zwimopxG0BDErvj7kHAQA=").unwrap(),
                Identifier::from_bytes(&DPNS_DATACONTRACT_ID).unwrap(),
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
