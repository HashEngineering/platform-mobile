use std::num::NonZeroUsize;
use std::sync::Arc;
use dash_sdk::{Error, Sdk};
use dash_sdk::platform::transition::put_identity::PutIdentity;
use dpp::dashcore::PrivateKey;
use dpp::identity::identity::Identity;
use dpp::identity::identity_public_key::accessors::v0::IdentityPublicKeyGettersV0;
use dpp::identity::identity_public_key::IdentityPublicKey;
use dpp::identity::signer::Signer;
use dpp::prelude::AssetLockProof;
use dpp::ProtocolError;
use platform_value::types::binary_data::BinaryData;
use platform_version::version::PlatformVersion;
use tokio::runtime::Builder;
use crate::config::Config;
use crate::fetch_identity::setup_logs;
use crate::provider::Cache;

#[ferment_macro::export]
type SignerCallback = extern "C" fn(key_data: * const u8, key_len: u32, data: * const u8, data_len: u32, result: * mut u8) -> u32;

//#[ferment_macro::export]
//type SignerCallback = extern "C" fn(identity_public_key: * const u8, data: * const u8) -> * const u8;
pub struct CallbackSigner {
    signer_callback: SignerCallback
}

impl CallbackSigner {
    pub fn new(
        signer_callback: u64,
    ) -> Result<Self, Error> {
        unsafe {
            let callback: SignerCallback = std::mem::transmute(signer_callback);
            Ok(Self {
                signer_callback: callback
            })
        }
    }
}



impl Signer for CallbackSigner {
    /// the public key bytes are only used to look up the private key
    fn sign(
        &self,
        identity_public_key: &IdentityPublicKey,
        data: &[u8],
    ) -> Result<BinaryData, ProtocolError> {
        // stub
        let key_data = identity_public_key.data();
        let mut result = [0u8; 128];
        let length = (self.signer_callback)(key_data.as_slice().as_ptr(), key_data.len() as u32, data.as_ptr(), data.len() as u32, result.as_mut_ptr());

        // Check the return value to determine if the operation was successful
        if length > 0 {
            // If 'length' is positive, it indicates the size of the signature
            // Create a Vec<u8> from 'result' up to 'length'
            Ok(BinaryData(result[..length as usize].to_vec()))
        } else {
            // Handle error scenario, for example by converting 'length' to an error code
            Err(ProtocolError::InvalidSigningKeyTypeError("somethign is wrong".to_string())) // Assuming there is a way to convert to ProtocolError
        }
    }
}

#[ferment_macro::export]
pub fn put_identity_create(identity: Identity, signer_callback: u64) -> Identity {
    let signer = CallbackSigner::new(signer_callback).expect("signer not valid");
    let data = [0u8; 1024];
    match signer.sign(&IdentityPublicKey::random_authentication_key(1, None, PlatformVersion::latest()), data.as_slice()) {
        Ok(sig) => println!("signature: {:?}", sig),
        Err(e) => println!("signature error: {}", e)
    }
    identity
}

fn put_identity(identity: &Identity,
                asset_lock_proof: AssetLockProof,
                asset_lock_proof_private_key: PrivateKey,
                signer: CallbackSigner,
                q: u64, d: u64) -> Result<Identity, Error> {
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
        println!("Setting up SDK");
        let sdk = cfg.setup_api_with_callbacks(q, d).await;
        println!("Finished SDK, {:?}", sdk);
        println!("Call fetch");
        let identity_result = Identity::put_to_platform_and_wait_for_response(
            identity,
            &sdk,
            asset_lock_proof,
            &asset_lock_proof_private_key,
            &signer).await;

        identity_result
    })
}