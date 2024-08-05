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
use crate::config::RustSdk;
use crate::config::create_sdk;
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
pub fn fetch_identity_with_sdk(
    rust_sdk: *mut RustSdk,
    identifier: Identifier
) -> Result<Identity, String> {
    println!("fetch_identity_with_sdk");
    unsafe {
        match identity_read_with_sdk(rust_sdk, &identifier) {
            Ok(identity) => Ok(identity),
            Err(err) => Err(err.to_string())
        }
    }
}

#[ferment_macro::export]
pub fn fetch_identity_balance_with_sdk(
    rust_sdk: *mut RustSdk,
    identifier: Identifier
) -> Result<u64, String> {
    println!("fetch_identity_with_sdk");
    unsafe {
        match identity_read_balance_with_sdk(rust_sdk, &identifier) {
            Ok(balance) => Ok(balance),
            Err(err) => Err(err.to_string())
        }
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

#[ferment_macro::export]
pub fn fetch_identity_with_keyhash_sdk(
    rust_sdk: *mut RustSdk,
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


//use serde::Deserialize;
use dpp::dashcore::PubkeyHash;
use drive_proof_verifier::types::IdentityBalance;
use platform_value::string_encoding::Encoding;
use crate::config::{Config, DPNS_DATACONTRACT_ID, DPNS_DATACONTRACT_OWNER_ID};
use crate::fetch_document::get_document;
use crate::logs::setup_logs;


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
        let id: Identifier = id.clone();
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

unsafe fn identity_read_with_sdk(rust_sdk: *mut RustSdk, id: &Identifier) -> Result<Identity, ProtocolError> {

    let rt = unsafe { (*rust_sdk).entry_point.get_runtime() }.clone();

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        let id: Identifier = id.clone();
        println!("Setting up SDK");
        let sdk = unsafe { (*rust_sdk).entry_point.get_sdk() };
        println!("Finished SDK, {:?}", sdk);
        println!("Call fetch");
        let identity_result = Identity::fetch(&sdk, id).await;

        match identity_result {
            Ok(Some(identity)) => Ok(identity),
            Ok(None) => Err(ProtocolError::IdentifierError("Identity not found".to_string())), // Placeholder for actual error handling
            Err(e) => Err(ProtocolError::IdentifierError(
                format!("Identifier not found: failure: {})", e))
            )
        }
    })
}


unsafe fn identity_read_balance_with_sdk(rust_sdk: *mut RustSdk, id: &Identifier) -> Result<u64, ProtocolError> {

    let rt = unsafe { (*rust_sdk).entry_point.get_runtime() }.clone();

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        // Your async code here
        let cfg = Config::new();
        let id: Identifier = id.clone();
        println!("Setting up SDK");
        let sdk = unsafe { (*rust_sdk).entry_point.get_sdk() };
        println!("Finished SDK, {:?}", sdk);
        println!("Call fetch");
        let identity_result = IdentityBalance::fetch(&sdk, id).await;

        match identity_result {
            Ok(Some(identity)) => Ok(identity),
            Ok(None) => Err(ProtocolError::IdentifierError("Identity not found".to_string())), // Placeholder for actual error handling
            Err(e) => Err(ProtocolError::IdentifierError(
                format!("Identifier not found: failure: {})", e))
            )
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

unsafe fn identity_from_keyhash_sdk(rust_sdk: *mut RustSdk, pubkey_hash: &PublicKeyHash) -> Result<Identity, ProtocolError> {
    setup_logs();
    // Create a new Tokio runtime
    //let rt = tokio::runtime::Runtime::new().expect("Failed to create a runtime");
    let rt = unsafe { (*rust_sdk).entry_point.get_runtime() };

    // Execute the async block using the Tokio runtime
    rt.block_on(async {
        // Your async code here
        let cfg = Config::new();
        let key_hash = pubkey_hash.clone();
        println!("Setting up SDK");
        let sdk = unsafe { (*rust_sdk).entry_point.get_sdk() };
        println!("Finished SDK, {:?}", sdk);
        println!("Call fetch");
        let identity_result = Identity::fetch(&sdk, key_hash).await;

        match identity_result {
            Ok(Some(identity)) => Ok(identity),
            Ok(None) => Err(ProtocolError::IdentifierError("Identity not found".to_string())), // Placeholder for actual error handling
            Err(e) => Err(ProtocolError::IdentifierError(
                format!("Identifier not found: failure: {})", e))
            )
        }
    })
}

#[test]
fn fetch_identity_test() {
    let result = fetch_identity_with_core(Identifier::from_string("3GupYWrQggzFBVZgL7fyHWensbWLwZBYFSbTXiSjXN5S", Encoding::Base58).unwrap());
    match result {
        Ok(identity) => println!("success fetching identity: {:?}", identity),
        Err(err) => panic!("error fetching identity: {}", err)
    }
}

#[test]
fn fetch_identity_with_sdk_test() {
    let mut rust_sdk = create_sdk(0, 0);
    let result = fetch_identity_with_sdk(
        &mut rust_sdk,
        Identifier::from_string("3GupYWrQggzFBVZgL7fyHWensbWLwZBYFSbTXiSjXN5S", Encoding::Base58).unwrap()
    );
    match result {
        Ok(identity) => println!("success fetching identity: {:?}", identity),
        Err(err) => panic!("error fetching identity: {}", err)
    }
}

#[test]
fn fetch_identity_balance_with_sdk_test() {
    let mut rust_sdk = create_sdk(0, 0);
    let result = fetch_identity_balance_with_sdk(
        &mut rust_sdk,
        Identifier::from_string("3GupYWrQggzFBVZgL7fyHWensbWLwZBYFSbTXiSjXN5S", Encoding::Base58).unwrap()
    );
    match result {
        Ok(balance) => println!("success fetching identity: {:?}", balance),
        Err(err) => panic!("error fetching identity: {}", err)
    }
}

#[test]
fn get_documents_test() {
    let result = get_document();
    println!("ownerId = {}", result)
}