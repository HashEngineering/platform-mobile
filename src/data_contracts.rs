use std::collections::BTreeMap;
use std::sync::Arc;
use dash_sdk::platform::{DocumentQuery, Fetch};
use dpp::data_contract::accessors::v0::DataContractV0Getters;
use dpp::data_contract::DataContract;
use dpp::document::Document;
use platform_value::{Identifier, IdentifierBytes32};
use platform_value::string_encoding::Encoding;
use tokio::runtime::Builder;
use crate::config::Config;
use crate::logs::setup_logs;

#[derive(Clone, Debug)]
#[ferment_macro::export]
pub struct DataContractFFI {
    pub id: Identifier,
    pub owner_id: Identifier,
    pub doc_types: Vec<String>,
    pub version: u32
}

#[allow(non_snake_case)]
#[ferment_macro::export]
pub fn DataContractFFI_clone(value: DataContractFFI) -> DataContractFFI {
    value.clone()
}

impl Into<DataContractFFI> for DataContract {
    fn into(self) -> DataContractFFI {
        match self {
            DataContract::V0(contract) => {
                DataContractFFI {
                    id: contract.id(),
                    owner_id: contract.owner_id(),
                    doc_types: contract.document_types.keys().cloned().collect(),
                    version: contract.version()
                }
            }
            _ => panic!("unknown version of DataContract")
        }
    }
}

#[ferment_macro::export]
pub fn fetch_data_contract(contract_id: Identifier,
                                 quorum_public_key_callback: u64,
                                 data_contract_callback: u64) -> Result<DataContractFFI, String> {
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
        let contract =
            DataContract::fetch(&sdk, data_contract_id.clone())
                .await;
        match contract {
            Ok(Some(data_contract)) => Ok(data_contract.into()),
            Ok(None) => Err("data contract not found".to_string()),
            Err(e) => Err(e.to_string())
        }
    })
}

#[test]
fn get_data_contract_test() {
    let config = Config::new();

    let data_contract = fetch_data_contract(
        Identifier(IdentifierBytes32(config.existing_data_contract_id.into())),
        0,
        0
    ).unwrap();
    println!("dpns: {:?}", data_contract);
}

#[test]
fn get_missing_data_contract_test() {
    let config = Config::new();

    let data_contract_result = fetch_data_contract(
        Identifier::from_string("Fds5DDfXoLwpUZ71AAVYZP1uod8S7Ze2bR28JExBvZKR", Encoding::Base58).expect("identifier"),
        0,
        0
    );

    assert!(data_contract_result.is_err());
}