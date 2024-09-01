use platform_value::types::identifier::{Identifier, IdentifierBytes32};
use dpp::identity::identity::Identity;
use dpp::errors::protocol_error::ProtocolError;
use platform_version::version::PlatformVersion;
use dpp::document::{Document, DocumentV0Getters};
use dash_sdk::platform::{DocumentQuery, Fetch, FetchMany};
use dash_sdk::platform::types::identity::PublicKeyHash;
use dpp::data_contract::DataContract;
use serde::Deserialize;
use tokio::runtime::{Runtime, Builder};
use dpp::dashcore::PubkeyHash;
use drive_proof_verifier::types::IdentityBalance;
use platform_value::string_encoding::Encoding;
use crate::config::{Config, DPNS_DATACONTRACT_ID, DPNS_DATACONTRACT_OWNER_ID, EntryPoint};
use crate::fetch_document::get_document;
use crate::logs::setup_logs;
use crate::sdk::{create_dash_sdk, DashSdk};

pub fn test_identifier() -> Identifier {
    Identifier::from_string("7Yowk46VwwHqmD5yZyyygggh937aP6h2UW7aQWBdWpM5", Encoding::Base58).unwrap()
}

// #[ferment_macro::export]
// pub fn fetch_identity_with_core(identifier: Identifier) -> Result<Identity, String> {
//     match identity_read(&identifier) {
//         Ok(identity) => Ok(identity),
//         Err(err) => Err(err.to_string())
//     }
// }

// #[ferment_macro::export]
// pub fn fetch_identity(identifier: Identifier,
//                        quorum_public_key_callback: u64,//QuorumPublicKeyCallback,
//                        data_contract_callback: u64 //DataContractCallback
// ) -> Result<Identity, String> {
//     tracing::info!("fetch_identity4");
//     match identity_read_with_callbacks(&identifier, quorum_public_key_callback, data_contract_callback) {
//         Ok(identity) => Ok(identity),
//         Err(err) => Err(err.to_string())
//     }
// }

#[ferment_macro::export]
pub fn fetch_identity_with_sdk(
    rust_sdk: *mut DashSdk,
    identifier: Identifier
) -> Result<Identity, String> {
    tracing::info!("fetch_identity_with_sdk");
    unsafe {
        match identity_read_with_sdk(rust_sdk, &identifier) {
            Ok(identity) => Ok(identity),
            Err(err) => Err(err.to_string())
        }
    }
}

#[ferment_macro::export]
pub fn fetch_identity_balance_with_sdk(
    rust_sdk: *mut DashSdk,
    identifier: Identifier
) -> Result<u64, String> {
    tracing::info!("fetch_identity_with_sdk");
    unsafe {
        match identity_read_balance_with_sdk(rust_sdk, &identifier) {
            Ok(balance) => Ok(balance),
            Err(err) => Err(err.to_string())
        }
    }
}

#[ferment_macro::export]
pub fn fetch_identity_with_keyhash_sdk(
    rust_sdk: *mut DashSdk,
    key_hash: [u8; 20]
) -> Result<Identity, String> {
    tracing::info!("fetch_identity_with_keyhash_sdk");
    unsafe {
        match identity_from_keyhash_sdk(rust_sdk, &PublicKeyHash(key_hash)) {
            Ok(identity) => Ok(identity),
            Err(err) => Err(err.to_string())
        }
    }
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
        let id: Identifier = id.clone();

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

// fn identity_read_with_callbacks(id: &Identifier, q: u64, d: u64) -> Result<Identity, ProtocolError> {
//     setup_logs();
//     // Create a new Tokio runtime
//     let rt = Builder::new_current_thread()
//         .enable_all() // Enables all I/O and time drivers
//         .build()
//         .expect("Failed to create a runtime");
//
//     // Execute the async block using the Tokio runtime
//     rt.block_on(async {
//         // Your async code here
//         let cfg = Config::new();
//         let id: Identifier = id.clone();
//         tracing::info!("Setting up SDK");
//         let sdk = if q != 0 {
//             cfg.setup_api_with_callbacks(q, d).await
//         } else {
//             cfg.setup_api().await
//         };
//         tracing::info!("Finished SDK, {:?}", sdk);
//         tracing::info!("Call fetch");
//         let identity_result = Identity::fetch(&sdk, id,).await;
//
//         match identity_result {
//             Ok(Some(identity)) => {
//                 // If you have an assertion here, note that assertions in async blocks will panic in the async context
//                 // assert_eq!(identity.id(), id);
//                 // Instead of an assertion, you might return an Ok or Err based on your logic
//                 Ok(identity)
//             },
//             Ok(None) => Err(ProtocolError::IdentifierError("Identity not found".to_string())), // Placeholder for actual error handling
//             Err(e) => Err(ProtocolError::IdentifierError(
//                 format!("Identifier not found: failure: {})", e))
//             ) // Convert your error accordingly
//         }
//     })
// }

unsafe fn identity_read_with_sdk(rust_sdk: *mut DashSdk, id: &Identifier) -> Result<Identity, ProtocolError> {

    let rt = unsafe { (*rust_sdk).get_runtime() }.clone();

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        let id: Identifier = id.clone();
        tracing::info!("Setting up SDK");
        let sdk = unsafe { (*rust_sdk).get_sdk() };
        tracing::info!("Finished SDK, {:?}", sdk);
        tracing::info!("Call fetch");
        let settings = unsafe { (*rust_sdk).get_request_settings() };
        let identity_result = Identity::fetch_with_settings(&sdk, id, settings).await;

        match identity_result {
            Ok(Some(identity)) => Ok(identity),
            Ok(None) => Err(ProtocolError::IdentifierError("Identity not found".to_string())), // Placeholder for actual error handling
            Err(e) => Err(ProtocolError::IdentifierError(
                format!("Identifier not found: failure: {})", e))
            )
        }
    })
}


unsafe fn identity_read_balance_with_sdk(rust_sdk: *mut DashSdk, id: &Identifier) -> Result<u64, ProtocolError> {

    let rt = unsafe { (*rust_sdk).get_runtime() }.clone();

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        // Your async code here
        let cfg = Config::new();
        let id: Identifier = id.clone();
        tracing::info!("Setting up SDK");
        let sdk = unsafe { (*rust_sdk).get_sdk() };
        tracing::info!("Finished SDK, {:?}", sdk);
        tracing::info!("Call fetch");
        let settings = unsafe { (*rust_sdk).get_request_settings() };
        let identity_result = IdentityBalance::fetch_with_settings(&sdk, id, settings).await;

        match identity_result {
            Ok(Some(identity)) => Ok(identity),
            Ok(None) => Err(ProtocolError::IdentifierError("Identity not found".to_string())), // Placeholder for actual error handling
            Err(e) => Err(ProtocolError::IdentifierError(
                format!("Identifier not found: failure: {})", e))
            )
        }
    })
}

// fn identity_from_keyhash_with_callbacks(
//     rust_sdk: *mut DashSdk,
//     pub_key_hash: &PublicKeyHash
// ) -> Result<Identity, ProtocolError> {
//
//     let rt = unsafe { (*rust_sdk).get_runtime() };
//
//     rt.block_on(async {
//         // Your async code here
//         let sdk = unsafe { (*rust_sdk).get_sdk() };
//         tracing::info!("Finished SDK, {:?}", sdk);
//         tracing::info!("Call fetch");
//         let identity_result = Identity::fetch(&sdk, pub_key_hash).await;
//
//         match identity_result {
//             Ok(Some(identity)) => {
//                 // If you have an assertion here, note that assertions in async blocks will panic in the async context
//                 // assert_eq!(identity.id(), id);
//                 // Instead of an assertion, you might return an Ok or Err based on your logic
//                 Ok(identity)
//             },
//             Ok(None) => Err(ProtocolError::IdentifierError("Identity not found".to_string())), // Placeholder for actual error handling
//             Err(e) => Err(ProtocolError::IdentifierError(
//                 format!("Identifier not found: failure: {})", e))
//             )
//         }
//     })
// }

unsafe fn identity_from_keyhash_sdk(rust_sdk: *mut DashSdk, pubkey_hash: &PublicKeyHash) -> Result<Identity, ProtocolError> {
    // Create a new Tokio runtime
    //let rt = tokio::runtime::Runtime::new().expect("Failed to create a runtime");
    let rt = unsafe { (*rust_sdk).get_runtime() };

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        // Your async code here
        let cfg = Config::new();
        let key_hash = pubkey_hash.clone();
        tracing::info!("Setting up SDK");
        let sdk = unsafe { (*rust_sdk).get_sdk() };
        tracing::info!("Finished SDK, {:?}", sdk);
        tracing::info!("Call fetch");
        let settings = unsafe { (*rust_sdk).get_request_settings() };
        let identity_result = Identity::fetch_with_settings(&sdk, key_hash, settings).await;

        match identity_result {
            Ok(Some(identity)) => Ok(identity),
            Ok(None) => Err(ProtocolError::IdentifierError("Identity not found".to_string())), // Placeholder for actual error handling
            Err(e) => Err(ProtocolError::IdentifierError(
                format!("Identifier not found: failure: {})", e))
            )
        }
    })
}

use dpp::identity::conversion::json::IdentityJsonConversionMethodsV0;
// #[test]
// fn fetch_identity_test() {
//     let result = fetch_identity_with_core(Identifier::from_string("9eV9jmcyM9qVSjPWrZZnb9utERHNHSzodEkzMJwn2rzP", Encoding::Base58).unwrap());
//     match result {
//         Ok(identity) => {
//             tracing::info!("success fetching identity: {:?}", identity);
//             tracing::info!("{:?}", identity);
//             if let Identity::V0(identityV0) = identity.clone() {
//                 tracing::info!("{}", identityV0.to_json().unwrap());
//             }
//             tracing::info!("{}", hex::encode(identity.serialize_to_bytes().unwrap()));
//         },
//         Err(err) => panic!("error fetching identity: {}", err)
//     }
// }

#[test]
fn fetch_identity_with_sdk_test() {
    let mut rust_sdk = create_dash_sdk(0, 0);
    let result = fetch_identity_with_sdk(
        &mut rust_sdk,
        test_identifier()
    );
    match result {
        Ok(identity) => tracing::info!("success fetching identity: {:?}", identity),
        Err(err) => panic!("error fetching identity: {}", err)
    }
}

#[test]
fn fetch_identity_balance_with_sdk_test() {
    let mut rust_sdk = create_dash_sdk(0, 0);
    let result = fetch_identity_balance_with_sdk(
        &mut rust_sdk,
        test_identifier()
    );
    match result {
        Ok(balance) => tracing::info!("success fetching identity: {:?}", balance),
        Err(err) => panic!("error fetching identity: {}", err)
    }
}