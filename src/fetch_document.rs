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
use crate::config::{Config, DPNS_DATACONTRACT_ID, DashSdk, RustSdk, create_sdk, EntryPoint};
use crate::logs::setup_logs;

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


#[ferment_macro::export]
pub unsafe fn fetch_documents_with_query_and_sdk(
                                  rust_sdk: *mut RustSdk,
                                  contract_id: Identifier,
                                  document_type: String,
                                  where_clauses: Vec<WhereClause>,
                                  order_clauses: Vec<OrderClause>,
                                  limit: u32,
                                  start: Option<StartPoint>
) -> Result<Vec<Document>, String> {
    setup_logs();

    let rt = (*rust_sdk).entry_point.get_runtime();

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        let sdk = (*rust_sdk).entry_point.get_sdk();

        let data_contract_id = contract_id;
        tracing::warn!("using existing data contract id and fetching...");
        // let contract = Arc::new(
        //     DataContract::fetch(&sdk, data_contract_id.clone())
        //         .await
        //         .expect("fetch data contract")
        //         .expect("data contract not found"),
        // );

        // let contract_fetch_result = match(sdk.context_provider()) {
        //     Some(context_provider) => {
        //         context_provider.get_data_contract(&contract_id)
        //     },
        //     None => return Err("data contract not found".to_string())
        // };

        let contract = match ((*rust_sdk).get_data_contract(&contract_id)) {
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

        // let contract_fetch_result =
        //     DataContract::fetch(&sdk, data_contract_id.clone())
        //         .await;
        tracing::warn!("contract_fetch_result: {:?}", contract);
        // let contract_result = match contract_fetch_result {
        //     Ok(contract) => contract,
        //     Err(e) => return Err(e.to_string())
        // };
        //tracing::warn!("contract_result: {:?}", contract_result);

        // let contract = match contract_result {
        //     Some(c) => c,
        //     None => return Err("contract not found".to_string())
        // };

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
        let settings = unsafe { (*rust_sdk).entry_point.get_request_settings() };
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

// #[ferment_macro::export]
// pub unsafe fn fetch_documents_with_query_and_sdk5(
//     rust_sdk: *mut RustSdk5,
//     contract_id: Identifier,
//     document_type: String,
//     where_clauses: Vec<WhereClause>,
//     order_clauses: Vec<OrderClause>,
//     limit: u32,
//     start: Option<StartPoint>
// ) -> Result<Vec<Document>, String> {
//     setup_logs();
//
//     let r_sdk = unsafe {
//         let box_ptr = (*rust_sdk).entry_point as *mut Box<DashSdk>;
//         let box_of_box = Box::from_raw(box_ptr);
//         let my_struct_box: Box<DashSdk> = *box_of_box;
//         my_struct_box
//     };
//
//     let rt = r_sdk.get_runtime();
//
//     // Execute the async block using the Tokio runtime
//     rt.block_on(async {
//         let sdk = r_sdk.get_sdk();
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

#[test]
fn docs_full_query_test() {
    let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
    let docs_result = fetch_documents_with_query(contract_id, "domain".to_string(),
                                                 vec![WhereClause {
                                              field: "normalizedLabel".to_string(),
                                              operator: WhereOperator::Equal,
                                              value: Value::Null,
                                          }],
                                                 vec![],
                                                 100,
                                                 None,
                                                 0, 0);

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
fn docs_full_query_sdk_test() {
    let mut sdk = create_sdk(0, 0);
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
fn docs_startswith_query_sdk_test() {
    let mut sdk = create_sdk(0, 0);
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
    let mut sdk = create_sdk(0, 0);
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
    let mut sdk = create_sdk(0, 0);

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

// #[test]
// fn docs_full_query_sdk5_test() {
//     let mut sdk = create_sdk5(0, 0);
//     // tracing::warn!("sdk: {:?}", sdk.entry_point.get_sdk());
//     let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
//     let docs_result = unsafe {
//         fetch_documents_with_query_and_sdk5(
//             &mut sdk,
//             contract_id,
//             "domain".to_string(),
//             vec![],
//             vec![],
//             100,
//             None
//         )
//     };
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