use std::collections::BTreeMap;
use std::sync::Arc;
use dash_sdk::platform::{DocumentQuery, Fetch, FetchMany};
use dapi_grpc::platform::v0::get_documents_request::get_documents_request_v0::Start;
use dash_sdk::platform::proto::GetDataContractRequest;
use dpp::bincode::config::Limit;
use dpp::data_contract::DataContract;
use dpp::document::{Document, DocumentV0Getters};
use drive::query::{ordering::OrderClause, conditions::WhereClause, conditions::WhereOperator};
use platform_value::{types::identifier::Identifier, IdentifierBytes32, Value};
use tokio::runtime::Builder;
use crate::config::{Config, DPNS_DATACONTRACT_ID};
use crate::fetch_identity::setup_logs;

#[ferment_macro::export]
pub fn get_document()-> Document {
    document_read()
}

#[ferment_macro::export]
pub fn document_to_string(document: Document)-> String {
    document.to_string()
}

#[ferment_macro::export]
pub fn get_documents(identifier: &Identifier, document_type: &String, q: u64, d: u64)-> Vec<Document> {
    documents_with_callbacks(identifier, document_type, q, d)
}

#[ferment_macro::export]
pub fn get_domain_document(identifier: &Identifier, q: u64, d: u64)-> Vec<Document> {
    dpns_domain_by_id(identifier, q, d)
}

#[ferment_macro::export]
pub fn get_domain_document_starts_with(starts_with: &String, q: u64, d: u64)-> Vec<Document> {
    dpns_domain_starts_with(starts_with, q, d)
}

// error[E0277]: the trait bound `dpp::document::Document: FFIConversion<dpp::document::Document>` is not satisfied
// --> src/fermented.rs:1:218446
// |
// 1 | ...ersion :: ffi_from (o) , | o | ferment_interfaces :: FFIConversion :: ffi_from_opt (o)) } unsafe fn ffi_to_const (obj : std :: collect...
// |                                   ---------------------------------------------------  ^ the trait `FFIConversion<dpp::document::Document>` is not implemented for `dpp::document::Document`
// |                                   |
// |                                   required by a bound introduced by this call

// #[ferment_macro::export]
// pub fn get_documents_tree(
//     identifier: &Identifier,
//     document_type: &String,
//     q: u64,
//     d: u64
// ) -> BTreeMap<Identifier, Option<Document>> {
//     documents_with_callbacks_tree(identifier, document_type, q, d)
// }

#[ferment_macro::export]
pub fn get_document_with_callbacks(quorum_public_key_callback: u64,
                                   data_contract_callback: u64
)-> Identifier {
    let it = document_read_with_callbacks(quorum_public_key_callback, data_contract_callback);
    match it {
        Document::V0(docV0) => docV0.owner_id,
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

fn documents_with_callbacks(contract_id: &Identifier,
                                document_type: &String,
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
pub fn fetch_documents_with_query(contract_id: &Identifier,
                            document_type: &String,
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
        let contract = Arc::new(
            DataContract::fetch(&sdk, data_contract_id.clone())
                .await
                .expect("fetch data contract")
                .expect("data contract not found"),
        );

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

fn dpns_domain_starts_with(starts_with: &String,
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

fn dpns_domain_by_id(unique_id: &Identifier,
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
                    field: "records.dashUniqueIdentityId".to_string(),
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

fn documents_with_callbacks_tree(contract_id: &Identifier,
                            document_type: &String,
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
    let docs = documents_with_callbacks(&contract_id, &"domain".to_string(), 0, 0);

    for document in docs {
        // Use `document` here
        println!("{:?}", document); // Assuming Document implements Debug
    }
}

#[test]
fn docs_query_test() {
   //let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
    let docs = dpns_domain_starts_with(&"dq-".to_string(), 0, 0);

    for document in docs {
        // Use `document` here
        println!("{:?}", document); // Assuming Document implements Debug
    }
}

#[test]
fn docs_query_id_test() {
    let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
    let docs = dpns_domain_by_id(&contract_id, 0, 0);

    for document in docs {
        // Use `document` here
        println!("{:?}", document); // Assuming Document implements Debug
    }
}

#[test]
fn docs_full_query_test() {
    let contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
    let docsResult = fetch_documents_with_query(&contract_id, &"domain".to_string(),
                                          vec![WhereClause {
                                              field: "normalizedLabel".to_string(),
                                              operator: WhereOperator::Equal,
                                              value: Value::Null,
                                          }],
                                          vec![],
                                          100,
                                          None,
                                          0, 0);

    match docsResult {
        Ok(docs) => {
            println!("query results");
            for document in docs {
                // Use `document` here
                println!("{:?}", document); // Assuming Document implements Debug
            }
        }
        Err(e) => panic!("{}", e)
    }
}