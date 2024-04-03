use std::num::NonZeroUsize;
use std::path::PathBuf;
use platform_value::{Identifier, IdentifierBytes32};
use rs_sdk::dapi_client::AddressList;
use std::sync::Arc;
use std::str::FromStr;
use dpp::data_contract::DataContract;
use drive_proof_verifier::ContextProvider;
use drive_proof_verifier::error::ContextProviderError;
use serde::Deserialize;

use lazy_static::lazy_static;
use parking_lot::Mutex;
use rs_sdk::mock::provider::GrpcContextProvider;
use crate::provider::CallbackContextProvider;

/// Existing document ID
///
// TODO: this is copy-paste from drive-abci `packages/rs-sdk/tests/fetch/main.rs` where it's private,
// consider defining it in `data-contracts` crate
pub const DPNS_DASH_TLD_DOCUMENT_ID: [u8; 32] = [
    215, 242, 197, 63, 70, 169, 23, 171, 110, 91, 57, 162, 215, 188, 38, 11, 100, 146, 137, 69, 55,
    68, 209, 224, 212, 242, 106, 141, 142, 255, 55, 207,
];

pub const DPNS_DATACONTRACT_ID: [u8; 32] = [
    230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109, 222, 123, 91, 126, 10, 29, 113, 42, 9,
    196, 13, 87, 33, 246, 34, 191, 83, 197, 49, 85,
];

pub const DPNS_DATACONTRACT_OWNER_ID: [u8; 32] = [
    48, 18, 193, 155, 152, 236, 0, 51, 173, 219, 54, 205, 100, 183, 245, 16, 103, 15, 42, 53, 26,
    67, 4, 181, 246, 153, 65, 68, 40, 110, 253, 172
];

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
"54.213.204.85"];

/// Configuration for dash-platform-sdk.
///
/// Content of this configuration is loaded from environment variables or `${CARGO_MANIFEST_DIR}/.env` file
/// when the [Config::new()] is called.
/// Variable names in the enviroment and `.env` file must be prefixed with [RS_SDK_](Config::CONFIG_PREFIX)
/// and written as SCREAMING_SNAKE_CASE (e.g. `RS_SDK_PLATFORM_HOST`).
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Hostname of the Dash Platform node to connect to
    #[serde(default)]
    pub platform_host: String,
    /// Port of the Dash Platform node grpc interface
    #[serde(default)]
    pub platform_port: u16,

    /// Hostname of the Dash Core node to connect to
    #[serde(default)]
    pub core_ip: String,
    /// Port of the Dash Core RPC interface running on the Dash Platform node
    #[serde(default)]
    pub core_port: u16,
    /// Username for Dash Core RPC interface
    #[serde(default)]
    pub core_user: String,
    /// Password for Dash Core RPC interface
    #[serde(default)]
    pub core_password: String,
    /// When true, use SSL for the Dash Platform node grpc interface
    #[serde(default)]
    pub platform_ssl: bool,

    /// Directory where all generated test vectors will be saved.
    ///
    /// See [SdkBuilder::with_dump_dir()](crate::SdkBuilder::with_dump_dir()) for more details.
    #[serde(default = "Config::default_dump_dir")]
    pub dump_dir: PathBuf,

    // IDs of some objects generated by the testnet
    /// ID of existing identity.
    ///
    /// Format: Base58
    #[serde(default = "Config::default_identity_id")]
    pub existing_identity_id: Identifier,
    /// ID of existing data contract.
    ///
    /// Format: Base58
    #[serde(default = "Config::default_data_contract_id")]
    pub existing_data_contract_id: Identifier,
    /// Name of document type defined for [`existing_data_contract_id`](Config::existing_data_contract_id).
    #[serde(default = "Config::default_document_type_name")]
    pub existing_document_type_name: String,
    /// ID of document of the type [`existing_document_type_name`](Config::existing_document_type_name)
    /// in [`existing_data_contract_id`](Config::existing_data_contract_id).
    #[serde(default = "Config::default_document_id")]
    pub existing_document_id: Identifier,
}

impl Config {
    /// Prefix of configuration options in the environment variables and `.env` file.
    pub const CONFIG_PREFIX: &'static str = "RS_SDK_";
    /// Load configuration from operating system environment variables and `.env` file.
    ///
    /// Create new [Config] with data from environment variables and `${CARGO_MANIFEST_DIR}/tests/.env` file.
    /// Variable names in the environment and `.env` file must be converted to SCREAMING_SNAKE_CASE and
    /// prefixed with [RS_SDK_](Config::CONFIG_PREFIX).
    pub fn new() -> Self {
        // load config from .env file, ignore errors

        let path: String = env!("CARGO_MANIFEST_DIR").to_owned() + "/.env";
        if let Err(err) = dotenvy::from_path(&path) {
            tracing::warn!(path, ?err, "failed to load config file");
        }

        let mut config: Self = envy::prefixed(Self::CONFIG_PREFIX)
             .from_env()
             .expect("configuration error");

        config.existing_data_contract_id = Identifier(IdentifierBytes32(DPNS_DATACONTRACT_ID));
        config.existing_document_id = Identifier(IdentifierBytes32(DPNS_DASH_TLD_DOCUMENT_ID));
        config.existing_document_type_name = "domain".to_string();


        if config.is_empty() {
            tracing::warn!(path, ?config, "some config fields are empty");
            #[cfg(not(feature = "offline-testing"))]
            panic!("invalid configuration")
        }

        config
    }

    /// Check if credentials of the config are empty.
    ///
    /// Checks if fields [platform_host](Config::platform_host), [platform_port](Config::platform_port),
    /// [core_port](Config::core_port), [core_user](Config::core_user) and [core_password](Config::core_password)
    /// are not empty.
    ///
    /// Other fields are ignored.
    pub fn is_empty(&self) -> bool {
        self.core_user.is_empty()
            || self.core_password.is_empty()
            || self.platform_host.is_empty()
            || self.platform_port == 0
            || self.core_port == 0
    }

    // #[allow(unused)]
    // /// Create list of Platform addresses from the configuration
    // pub fn address_list(&self) -> AddressList {
    //     let scheme = match self.platform_ssl {
    //         true => "https",
    //         false => "http",
    //     };
    //
    //     let address: String = format!("{}://{}:{}", scheme, self.platform_host, self.platform_port);
    //
    //     AddressList::from_iter(vec![http::Uri::from_str(&address).expect("valid uri")])
    // }

    #[allow(unused)]
    /// Create list of Platform addresses from the configuration
    pub fn address_list(&self) -> AddressList {
        let scheme = if self.platform_ssl { "https" } else { "http" };

        let uris: Result<Vec<http::Uri>, http::uri::InvalidUri> = if ADDRESS_LIST.is_empty() {
            println!("default address list is empty");
            let address = format!("{}://{}:{}", scheme, self.platform_host, self.platform_port);
            vec![http::Uri::from_str(&address)].into_iter().collect()
        } else {
            ADDRESS_LIST.iter().map(|host| {
                let uri = format!("{}://{}:{}", scheme, host, self.platform_port);
                http::Uri::from_str(&uri)
            }).collect()
        };

        uris.map(AddressList::from_iter).expect("valid address list")
    }

    /// Create new SDK instance
    ///
    /// Depending on the feature flags, it will connect to the configured platform node or mock API.
    ///
    /// ## Feature flags
    ///
    /// * `offline-testing` is not set - connect to the platform and generate
    /// new test vectors during execution
    /// * `offline-testing` is set - use mock implementation and
    /// load existing test vectors from disk
    pub async fn setup_api(&self) -> Arc<rs_sdk::Sdk> {
        // offline testing takes precedence over network testing
        //#[cfg(all(feature = "network-testing", not(feature = "offline-testing")))]
            let sdk = {
            // Dump all traffic to disk
            let builder = rs_sdk::SdkBuilder::new(self.address_list()).with_core(
                &self.core_ip,
                self.core_port,
                &self.core_user,
                &self.core_password,
            );

            builder.build().expect("cannot initialize api")
        };

        sdk
    }

    pub async fn setup_api_with_callbacks(&self, q: u64, d: u64) -> Arc<rs_sdk::Sdk> {
        let context_provider = CallbackContextProvider::new(
            q,
            d,
            None,
            NonZeroUsize::new(100).expect("Non Zero"),
            NonZeroUsize::new(100).expect("Non Zero")
        ).expect("context provider");
        let context_provider = Arc::new(std::sync::Mutex::new(context_provider));
        let sdk = {
            // Dump all traffic to disk
            let builder = rs_sdk::SdkBuilder::new(self.address_list());

            builder.build().expect("cannot initialize api")
        };
        sdk.set_context_provider(context_provider);
        sdk
    }

    fn default_identity_id() -> Identifier {
        data_contracts::dpns_contract::OWNER_ID_BYTES.into()
    }

    fn default_data_contract_id() -> Identifier {
        data_contracts::dpns_contract::ID_BYTES.into()
    }

    fn default_document_type_name() -> String {
        "domain".to_string()
    }
    fn default_document_id() -> Identifier {
        DPNS_DASH_TLD_DOCUMENT_ID.into()
    }

    fn default_dump_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("vectors")
    }
}
