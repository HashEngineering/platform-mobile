use platform_value::types::identifier::{Identifier, IdentifierBytes32};
use dpp::identity::identity::Identity;
use dpp::errors::protocol_error::ProtocolError;
use platform_version::version::PlatformVersion;
use dpp::document::{Document, DocumentV0Getters};
use dash_sdk::platform::{DocumentQuery, Fetch, FetchMany};
use dash_sdk::platform::types::identity::PublicKeyHash;
use dpp::prelude::DataContract;
use serde::Deserialize;
use tokio::runtime::{Runtime, Builder};

// #[ferment_macro::export]
// pub fn fetch_identity(identifier: Identifier) -> Identity {
//     Identity::create_basic_identity(identifier.into(), PlatformVersion::latest()).expect("failed")
//
//     // Result::Err(ProtocolError::IdentifierError("error with id".into()))
// }
//
// #[ferment_macro::export]
// pub fn fetch_identity2(identifier: Identifier) -> Identity {
//     identity_read(&identifier).expect("not found")
// }
//#[ferment_macro::export]
pub struct FermentError { error_message: String }
#[ferment_macro::export]
pub fn fetch_identity_with_core(identifier: Identifier) -> Result<Identity, String> {
    match identity_read(&identifier) {
        Ok(identity) => Ok(identity),
        Err(err) => Err(err.to_string())
    }
}

#[ferment_macro::export]
pub fn fetch_identity(identifier: Identifier,
                       quorum_public_key_callback: u64,//QuorumPublicKeyCallback,
                       data_contract_callback: u64 //DataContractCallback
) -> Result<Identity, String> {
    println!("fetch_identity4");
    match identity_read_with_callbacks(&identifier, quorum_public_key_callback, data_contract_callback) {
        Ok(identity) => Ok(identity),
        Err(err) => Err(err.to_string())
    }
}

#[ferment_macro::export]
pub fn fetch_identity_with_keyhash(key_hash: [u8; 20],
                      quorum_public_key_callback: u64,
                      data_contract_callback: u64
) -> Result<Identity, String> {
    println!("fetch_identity4");
    match identity_from_keyhash_with_callbacks(&PublicKeyHash(key_hash), quorum_public_key_callback, data_contract_callback) {
        Ok(identity) => Ok(identity),
        Err(err) => Err(err.to_string())
    }
}


//use serde::Deserialize;
use dpp::dashcore::PubkeyHash;
use crate::config::{Config, DPNS_DATACONTRACT_ID, DPNS_DATACONTRACT_OWNER_ID};
use crate::fetch_document::get_document;


async fn test_identity_read() {
    setup_logs();

    use dpp::identity::accessors::IdentityGettersV0;
    use crate::config::Config;
    let cfg = Config::new();
    let id: dpp::prelude::Identifier = cfg.existing_identity_id;

    let sdk = cfg.setup_api().await;

    let identity = Identity::fetch(&sdk, id)
        .await
        .expect("fetch identity")
        .expect("found identity");

    assert_eq!(identity.id(), id);
}


fn identity_read(id: &Identifier) -> Result<Identity, ProtocolError> {
    setup_logs();
    // Create a new Tokio runtime
    //let rt = tokio::runtime::Runtime::new().expect("Failed to create a runtime");
    let rt = Builder::new_current_thread()
        .enable_all() // Enables all I/O and time drivers
        .build()
        .expect("Failed to create a runtime");

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        // Your async code here
        let cfg = Config::new();
        let id: dpp::prelude::Identifier = id.clone();

        let sdk = cfg.setup_api().await;

        let identity_result = Identity::fetch(&sdk, id).await;

        match identity_result {
            Ok(Some(identity)) => {
                // If you have an assertion here, note that assertions in async blocks will panic in the async context
                // assert_eq!(identity.id(), id);
                // Instead of an assertion, you might return an Ok or Err based on your logic
                Ok(identity)
            },
            Ok(None) => Err(ProtocolError::IdentifierError("Identity not found".to_string())), // Placeholder for actual error handling
            Err(e) => Err(ProtocolError::IdentifierError("Identifier not found: failure".to_string())), // Convert your error accordingly
        }
    })
}

fn identity_read_with_callbacks(id: &Identifier, q: u64, d: u64) -> Result<Identity, ProtocolError> {
    setup_logs();
    // Create a new Tokio runtime
    //let rt = tokio::runtime::Runtime::new().expect("Failed to create a runtime");
    let rt = Builder::new_current_thread()
        .enable_all() // Enables all I/O and time drivers
        .build()
        .expect("Failed to create a runtime");
    
    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        // Your async code here
        let cfg = Config::new();
        let id: dpp::prelude::Identifier = id.clone();
        println!("Setting up SDK");
        let sdk = if q != 0 {
            cfg.setup_api_with_callbacks(q, d).await
        } else {
            cfg.setup_api().await
        };
        println!("Finished SDK, {:?}", sdk);
        println!("Call fetch");
        let identity_result = Identity::fetch(&sdk, id).await;

        match identity_result {
            Ok(Some(identity)) => {
                // If you have an assertion here, note that assertions in async blocks will panic in the async context
                // assert_eq!(identity.id(), id);
                // Instead of an assertion, you might return an Ok or Err based on your logic
                Ok(identity)
            },
            Ok(None) => Err(ProtocolError::IdentifierError("Identity not found".to_string())), // Placeholder for actual error handling
            Err(e) => Err(ProtocolError::IdentifierError(
                format!("Identifier not found: failure: {})", e))
            ) // Convert your error accordingly
        }
    })
}

fn identity_from_keyhash_with_callbacks(pubkey_hash: &PublicKeyHash, q: u64, d: u64) -> Result<Identity, ProtocolError> {
    setup_logs();
    // Create a new Tokio runtime
    //let rt = tokio::runtime::Runtime::new().expect("Failed to create a runtime");
    let rt = Builder::new_current_thread()
        .enable_all() // Enables all I/O and time drivers
        .build()
        .expect("Failed to create a runtime");

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        // Your async code here
        let cfg = Config::new();
        let key_hash = pubkey_hash.clone();
        println!("Setting up SDK");
        let sdk = if q != 0 {
            cfg.setup_api_with_callbacks(q, d).await
        } else {
            cfg.setup_api().await
        };
        println!("Finished SDK, {:?}", sdk);
        println!("Call fetch");
        let identity_result = Identity::fetch(&sdk, key_hash).await;

        match identity_result {
            Ok(Some(identity)) => {
                // If you have an assertion here, note that assertions in async blocks will panic in the async context
                // assert_eq!(identity.id(), id);
                // Instead of an assertion, you might return an Ok or Err based on your logic
                Ok(identity)
            },
            Ok(None) => Err(ProtocolError::IdentifierError("Identity not found".to_string())), // Placeholder for actual error handling
            Err(e) => Err(ProtocolError::IdentifierError(
                format!("Identifier not found: failure: {})", e))
            )
        }
    })
}

pub fn setup_logs() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            "info,dash_sdk=trace,h2=info",
        ))
        .pretty()
        .with_ansi(false)
        .with_writer(std::io::stdout)
        .try_init()
        .ok();
}

#[test]
fn fetch_identity_test() {
    let result = fetch_identity_with_core(Identifier(IdentifierBytes32(DPNS_DATACONTRACT_OWNER_ID)));
    match result {
        Ok(identity) => println!("success fetching identity: {:?}", identity),
        Err(err) => panic!("error fetching identity: {}", err)
    }
}

#[test]
fn get_documents_test() {
    let result = get_document();
    println!("ownerId = {}", result)
}