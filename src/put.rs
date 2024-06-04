use std::collections::{BTreeMap, HashMap};
use std::io;
use std::io::{Cursor, Write};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
use dash_sdk::{Error, RequestSettings, Sdk};
use dash_sdk::platform::Fetch;
use dash_sdk::platform::transition::put_document::PutDocument;
use dash_sdk::platform::transition::put_identity::PutIdentity;
use dash_sdk::platform::transition::put_settings::PutSettings;
use dash_sdk::platform::transition::TxId;
use dashcore::hashes::Hash;
use dashcore::signer::sign;
use dpp::bincode::{Decode, Encode};
use dpp::consensus::ConsensusError;
use dpp::dashcore::{InstantLock, Network, OutPoint, PrivateKey, Transaction, Txid};
use dpp::dashcore::bls_sig_utils::BLSSignature;
use dpp::dashcore::consensus::Decodable;
use dpp::dashcore::hash_types::CycleHash;
use dpp::dashcore::hashes::sha256d;
use dpp::data_contract::{DataContract, DataContractV0};
use dpp::data_contract::accessors::v0::DataContractV0Getters;
use dpp::data_contract::DataContract::V0;
use dpp::data_contract::document_type::DocumentType;
use dpp::data_contract::document_type::methods::DocumentTypeV0Methods;
use dpp::document::{Document, DocumentV0Getters};
use dpp::document::DocumentV0;
use dpp::identity::Identity;
use dpp::identity::identity_public_key::accessors::v0::IdentityPublicKeyGettersV0;
use dpp::identity::identity_public_key::IdentityPublicKey;
use dpp::identity::identity_public_key::v0::IdentityPublicKeyV0;
use dpp::identity::{KeyType, Purpose, SecurityLevel};
use dpp::identity::signer::Signer;
use dpp::identity::state_transition::asset_lock_proof::chain::ChainAssetLockProof;
use dpp::identity::state_transition::asset_lock_proof::InstantAssetLockProof;
use dpp::prelude::{AssetLockProof, BlockHeight, CoreBlockHeight};
use dpp::ProtocolError;
use dpp::util::entropy_generator::{DefaultEntropyGenerator, EntropyGenerator};
use platform_value::{Identifier, Value};
use platform_value::string_encoding::Encoding;
use platform_value::BinaryData;
use platform_version::version::PlatformVersion;
use serde::{Deserialize, Serialize};
use tokio::runtime::Builder;
use tracing::trace;
use rand::random;
use simple_signer::signer::SimpleSigner;
use crate::config::Config;
use crate::fetch_identity::setup_logs;
use crate::provider::Cache;

//#[ferment_macro::export]
pub type SignerCallback = extern "C" fn(key_data: * const u8, key_len: u32, data: * const u8, data_len: u32, result: * mut u8) -> u32;

// #[ferment_macro::export]
// pub type SignerCallback2 = fn(key_data: * const u8, key_len: u32, data: * const u8, data_len: u32, result: * mut u8) -> u32;
// #[ferment_macro::export]
// pub type SignerCallback3 = extern "C" fn(key: Vec<u8>, data: Vec<u8>, result: Vec<u8>) -> u32;
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
        trace!("CallbackSigner::sign({:?}, {:?})", key_data.as_slice(), data);
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

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[ferment_macro::export]
pub struct OutPointFFI {
    /// The referenced transaction's txid.
    pub txid: [u8; 32],
    /// The index of the referenced output in its transaction's vout.
    pub vout: u32,
}

#[ferment_macro::export]
pub fn OutPointFFI_clone(a: OutPointFFI) -> OutPointFFI {
    a.clone()
}

impl From<OutPointFFI> for OutPoint {
    fn from(value: OutPointFFI) -> Self {
        Self {
            txid: Txid::from_raw_hash(sha256d::Hash::from_slice(value.txid.as_slice()).unwrap()),
            vout: value.vout,
        }
    }
}

// #[derive(Clone, Eq, PartialEq)]
// /// Instant send lock is a mechanism used by the Dash network to
// /// confirm transaction within 1 or 2 seconds. This data structure
// /// represents a p2p message containing a data to verify such a lock.
// pub struct InstantLockFFI {
//     /// Instant lock version
//     pub version: u8,
//     /// Transaction inputs locked by this instant lock
//     pub inputs: Vec<OutPointFFI>,
//     /// Transaction hash locked by this lock
//     pub txid: [u8; 32],
//     /// Hash to figure out which quorum was used to sign this IS lock
//     pub cyclehash: [u8; 32],
//     /// Quorum signature for this IS lock
//     pub signature: [u8; 96],
// }

#[derive(Clone, PartialEq, Eq, Debug)]
#[ferment_macro::export]
pub struct ChainAssetLockProofFFI {
    /// Core height on which the asset lock transaction was chain locked or higher
    pub core_chain_locked_height: u32,
    /// A reference to Asset Lock Special Transaction ID and output index in the payload
    pub out_point: OutPointFFI,
}

#[ferment_macro::export]
pub fn ChainAssetLockProofFFI_clone(a: ChainAssetLockProofFFI) -> ChainAssetLockProofFFI {
    a.clone()
}

impl From<ChainAssetLockProofFFI> for ChainAssetLockProof {
    fn from(value: ChainAssetLockProofFFI) -> Self {
        ChainAssetLockProof {
            core_chain_locked_height: value.core_chain_locked_height,
            out_point: value.out_point.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[ferment_macro::export]
pub struct InstantAssetLockProofFFI {
    /// The transaction's Instant Lock
    pub instant_lock: Vec<u8>,
    /// Asset Lock Special Transaction
    pub transaction: Vec<u8>,
    /// Index of the output in the transaction payload
    pub output_index: u32,
}

#[ferment_macro::export]
pub fn InstantAssetLockProofFFI_clone(a: InstantAssetLockProofFFI) -> InstantAssetLockProofFFI {
    a.clone()
}

impl From<InstantAssetLockProofFFI> for InstantAssetLockProof {
    fn from(value: InstantAssetLockProofFFI) -> Self {
        let mut islock_cursor = Cursor::new(value.instant_lock);
        let mut transaction_cursor = Cursor::new(value.transaction);
        InstantAssetLockProof {
            instant_lock: InstantLock::consensus_decode(&mut islock_cursor).unwrap(),
            transaction: Transaction::consensus_decode(&mut transaction_cursor).unwrap(),
            output_index: value.output_index,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[ferment_macro::export]
pub enum AssetLockProofFFI {
    Instant(InstantAssetLockProofFFI),
    Chain(ChainAssetLockProofFFI),
}

impl From<AssetLockProofFFI> for AssetLockProof {
    fn from(value: AssetLockProofFFI) -> Self {
        match value {
            AssetLockProofFFI::Instant(instant) => AssetLockProof::Instant(instant.into()),
            AssetLockProofFFI::Chain(chain) => AssetLockProof::Chain(chain.into())
        }
    }
}

#[ferment_macro::export]
pub fn put_identity(
    identity: Identity,
    asset_lock_proof: AssetLockProofFFI,
    asset_lock_proof_private_key: Vec<u8>,
    signer_callback: u64,
    q: u64,
    d: u64,
    is_testnet: bool
) -> Result<Identity, String> {
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
        trace!("Setting up SDK");
        let sdk = cfg.setup_api_with_callbacks(q, d).await;
        trace!("Finished SDK, {:?}", sdk);
        trace!("Set up network, private key and signer");

        let network = if is_testnet {
            Network::Testnet
        } else {
            Network::Dash
        };
        let private_key = PrivateKey::from_slice(asset_lock_proof_private_key.as_slice(), network).expect("private key");
        let signer = CallbackSigner::new(signer_callback).expect("signer");

        trace!("Call Identity::put_to_platform_and_wait_for_response");
        let identity_result = Identity::put_to_platform_and_wait_for_response(
            &identity,
            &sdk,
            asset_lock_proof.into(),
            &private_key,
            &signer).await;

        match identity_result {
            Ok(identity) => Ok(identity),
            Err(err) => Err(err.to_string())
        }
    })
}

#[ferment_macro::export]
pub fn put_document(
    document: Document,
    data_contract_id: Identifier,
    document_type_str: String,
    identity_public_key: IdentityPublicKey,
    block_height: BlockHeight,
    core_block_height: CoreBlockHeight,
    signer_callback: u64,
    quorum_key_callback: u64,
    d: u64
) -> Result<Document, String> {

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
        trace!("Setting up SDK");
        let sdk = cfg.setup_api_with_callbacks(quorum_key_callback, d).await;
        trace!("Finished SDK, {:?}", sdk);
        trace!("Set up entropy, data contract and signer");

        let data_contract = match DataContract::fetch(&sdk, data_contract_id).await {
            Ok(Some(contract)) => contract,
            Ok(None) => return Err("no contract".to_string()),
            Err(e) => return Err(e.to_string())
        };

        let document_type = data_contract
            .document_type_for_name(&document_type_str)
            .expect("expected a profile document type");

        let signer = CallbackSigner::new(signer_callback).expect("signer");
        let entropy_generator = DefaultEntropyGenerator;
        let entropy = entropy_generator.generate().unwrap();
        //let document_entropy = entropy_generator.generate().unwrap();
        trace!("document_entropy: {:?}", entropy);
        trace!("IdentityPublicKey: {:?}", identity_public_key);

        // recreate the document using the same entropy value as when it is submitted below
        let new_document_result = document_type.create_document_from_data(
            document.properties().into(),
            document.owner_id(),
            block_height,
            core_block_height,
            entropy,
            PlatformVersion::latest()
        );

        let new_document = match new_document_result {
            Ok(doc) => doc,
            Err(e) => return Err(e.to_string())
        };

        let settings = PutSettings {
            request_settings: RequestSettings {
                connect_timeout: None,
                timeout: None,
                retries: None, //Some(2),
                ban_failed_address: Some(true),
            },
            identity_nonce_stale_time_s: None,
            user_fee_increase: None,
        };

        trace!("Call Document::put_to_platform_and_wait_for_response");
        let document_result = new_document.put_to_platform_and_wait_for_response(
            &sdk,
            document_type.to_owned_document_type(),
            entropy,
            identity_public_key,
            Arc::new(data_contract),
            &signer,
        ).await;

        document_result.map_err(|err| err.to_string())
    })
}