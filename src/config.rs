use std::os::raw::c_void;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use platform_value::{Identifier, IdentifierBytes32};
use dash_sdk::dapi_client::AddressList;
use std::sync::Arc;
use std::str::FromStr;
use dpp::data_contract::DataContract;
use drive_proof_verifier::ContextProvider;
use drive_proof_verifier::error::ContextProviderError;
use serde::Deserialize;
use std::collections::BTreeMap;

use lazy_static::lazy_static;
use parking_lot::Mutex;
use dash_sdk::mock::provider::GrpcContextProvider;
use dash_sdk::{RequestSettings, Sdk};
use dpp::data_contract::accessors::v0::DataContractV0Getters;
use ferment_interfaces::{boxed, unbox_any};
use http::Uri;
use tokio::runtime::{Builder, Runtime};
use crate::logs::setup_logs;
use crate::provider::{Cache, CallbackContextProvider};
use crate::sdk::DashSdk;

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

pub const DPNS_DATACONTRACT_OWNER_ID: [u8; 32] = [0; 32];

pub const ADDRESS_LIST: [&str; 32] = [
    "34.214.48.68",
    // "35.166.18.166",
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

// #[ferment_macro::opaque]
// pub trait EntryPoint2 {
//
// }
//#[ferment_macro::opaque]
pub trait EntryPoint {
    fn get_runtime(&self) -> Arc<Runtime>;
    fn get_sdk(&self) -> Arc<Sdk>;
    fn get_data_contract(&self, identifier: &Identifier) -> Option<Arc<DataContract>>;
    fn add_data_contract(&mut self, data_contract: &DataContract);
    fn get_request_settings(&self) -> RequestSettings;
}





// should this be exported as Arc<> by the functions?
// #[ferment_macro::opaque]
// pub struct RustSdk {
//     pub entry_point: Box<dyn EntryPoint>
// }
//
// impl RustSdk {
//     fn get_runtime(self) -> Arc<Runtime> {
//         self.entry_point.get_runtime()
//     }
//     fn get_sdk(self) -> Arc<Sdk> {
//         self.entry_point.get_sdk()
//     }
//
//     pub fn get_data_contract(&self, identifier: &Identifier) -> Option<Arc<DataContract>> {
//         self.entry_point.get_data_contract(identifier)
//     }
//
//     pub fn add_data_contract(&mut self, data_contract: &DataContract) {
//         self.entry_point.set_data_contract(data_contract)
//     }
//
//     pub fn get_request_settings(&self) -> RequestSettings {
//         self.entry_point.get_request_settings()
//     }
// }

// #[ferment_macro::opaque]
// pub struct RustSdk2 {
//     pub entry_point: Box<dyn EntryPoint2>
// }


//
// #[ferment_macro::opaque]
// pub struct RustSdk3 {
//     pub entry_point: Box<DashSdk>
// }

// #[ferment_macro::opaque]
// pub struct RustSdk4 {
//     pub entry_point: * mut DashSdk
// }

//#[ferment_macro::opaque]
// pub struct DashSharedCoreWithRuntime {
//     pub sdk: *mut Sdk,
//     pub runtime: *mut Runtime,
// }

//#[ferment_macro::export]
// impl DashSharedCoreWithRuntime {
//     pub fn new(quorum_public_key_callback: u64, data_contract_callback: u64) -> Self {
//         setup_logs();
//         let rt =
//             Builder::new_current_thread()
//                 .enable_all() // Enables all I/O and time drivers
//                 .build()
//                 .expect("Failed to create a runtime");
//
//         // Execute the async block using the Tokio runtime
//         rt.block_on(async {
//             let cfg = Config::new();
//             let sdk = if quorum_public_key_callback != 0 {
//                 // use the callbacks to obtain quorum public keys
//                 cfg.setup_api_with_callbacks(quorum_public_key_callback, data_contract_callback).await
//             } else {
//                 // use Dash Core for quorum public keys
//                 cfg.setup_api().await
//             };
//
//             let rt = Builder::new_current_thread()
//                     .enable_all() // Enables all I/O and time drivers
//                     .build()
//                     .expect("Failed to create a runtime");
//             Self {
//                 sdk: boxed(sdk.as_ref().clone()),
//                 runtime: boxed(rt)
//             }
//         })
//     }
// }


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
            config.platform_host = "54.213.204.85".to_string();
            config.platform_port = 1443;
            config.core_port = 19998;
            config.platform_ssl = true
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
            tracing::info!("default address list is empty");
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

    pub fn new_address_list(&self, address_list: Vec<String>) -> AddressList {
        let scheme = if self.platform_ssl { "https" } else { "http" };
        let uris: Vec<Uri> = address_list.into_iter().map(|host| {
            let uri = format!("{}://{}:{}", scheme, host, self.platform_port);
            Uri::from_str(&uri).expect("valid address list")
        }).collect();

        AddressList::from_iter(uris)
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
    pub async fn setup_api(&self) -> Arc<Sdk> {
        let sdk = {
            // Dump all traffic to disk
            let builder = dash_sdk::SdkBuilder::new(self.address_list()).with_core(
                &self.core_ip,
                self.core_port,
                &self.core_user,
                &self.core_password,
            );

            builder.build().expect("cannot initialize api")
        };

        sdk.into()
    }

    pub async fn setup_api_list(&self, address_list: Vec<String>) -> Arc<Sdk> {
        let sdk = {
            // Dump all traffic to disk
            let builder = dash_sdk::SdkBuilder::new(self.new_address_list(address_list)).with_core(
                &self.core_ip,
                self.core_port,
                &self.core_user,
                &self.core_password,
            );

            builder.build().expect("cannot initialize api")
        };

        sdk.into()
    }

    pub async fn setup_api_with_callbacks(&self, q: u64, d: u64) -> Arc<Sdk> {
        let mut context_provider = CallbackContextProvider::new(
            std::ptr::null(),
            q,
            d,
            None,
            Arc::new(Cache::new(NonZeroUsize::new(100).expect("Non Zero"))),
            NonZeroUsize::new(100).expect("Non Zero"),
        ).expect("context provider");
        let mut sdk = {
            // Dump all traffic to disk
            let builder = dash_sdk::SdkBuilder::new(self.address_list());
            builder.build().expect("cannot initialize api")
        };
        // not ideal because context provider has a clone of the sdk
        context_provider.set_sdk(Some(Arc::new(sdk.clone())));
        sdk.set_context_provider(context_provider);
        sdk.into()
    }

    pub async fn setup_api_with_callbacks_cache(
        &self,
        context_provider_context: * const c_void,
        q: u64,
        d: u64,
        data_contract_cache: Arc<Cache<Identifier, DataContract>>,
    ) -> Arc<Sdk> {
        let mut context_provider = CallbackContextProvider::new(
            context_provider_context,
            q,
            d,
            None,
            data_contract_cache,
            NonZeroUsize::new(100).expect("Non Zero")
        ).expect("context provider");
        let mut sdk = {
            // Dump all traffic to disk
            let builder = dash_sdk::SdkBuilder::new(self.address_list());
            builder.build().expect("cannot initialize api")
        };
        // not ideal because context provider has a clone of the sdk
        context_provider.set_sdk(Some(Arc::new(sdk.clone())));
        sdk.set_context_provider(context_provider);
        sdk.into()
    }

    pub async fn setup_api_with_callbacks_cache_list(
        &self,
        q: u64,
        d: u64,
        data_contract_cache: Arc<Cache<Identifier, DataContract>>,
        address_list: Vec<String>
    ) -> Arc<Sdk> {
        let mut context_provider = CallbackContextProvider::new(
            std::ptr::null(),
            q,
            d,
            None,
            data_contract_cache,
            NonZeroUsize::new(100).expect("Non Zero")
        ).expect("context provider");
        let mut sdk = {
            // Dump all traffic to disk
            let builder = dash_sdk::SdkBuilder::new(self.new_address_list(address_list));
            builder.build().expect("cannot initialize api")
        };
        // not ideal because context provider has a clone of the sdk
        context_provider.set_sdk(Some(Arc::new(sdk.clone())));
        sdk.set_context_provider(context_provider);
        sdk.into()
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

// #[ferment_macro::export]
// pub fn create_dashsharedcore(quorum_public_key_callback: u64,
//                   data_contract_callback: u64) -> DashSharedCore {
//     return DashSharedCore::new(std::ptr::null())
// }

// #[ferment_macro::export]
// pub fn create_sdk(
//     quorum_public_key_callback: u64,
//     data_contract_callback: u64
// ) -> RustSdk {
//     setup_logs();
//     let rt = Arc::new(
//         Builder::new_multi_thread()
//         .enable_all() // Enables all I/O and time drivers
//         .build()
//         .expect("Failed to create a runtime")
//     );
//
//     // Execute the async block using the Tokio runtime
//     rt.block_on(async {
//         let cfg = Config::new();
//         tracing::info!("cfg created");
//         let data_contract_cache = Arc::new(Cache::new(NonZeroUsize::new(100).expect("Non Zero")));
//         let sdk = if quorum_public_key_callback != 0 {
//             // use the callbacks to obtain quorum public keys
//             cfg.setup_api_with_callbacks_cache(quorum_public_key_callback, data_contract_callback, data_contract_cache.clone()).await
//         } else {
//             // use Dash Core for quorum public keys
//             cfg.setup_api().await
//         };
//         tracing::info!("sdk created");
//         RustSdk {
//             entry_point: Box::new(DashSdk {
//                 config: Arc::new(cfg),
//                 runtime: rt.clone(),
//                 sdk: sdk,
//                 data_contract_cache: data_contract_cache,
//                 request_settings: RequestSettings {
//                     connect_timeout: None,
//                     timeout: None,
//                     retries: Some(5),
//                     ban_failed_address: Some(true),
//                 }
//             }
//             )
//         }
//     })
// }
//
//
//
// pub fn destroy_sdk(rust_sdk: * mut RustSdk) {
//     unsafe  { unbox_any(rust_sdk) };
// }


// #[ferment_macro::export]
// pub fn create_sdk2(
//     quorum_public_key_callback: u64,
//     data_contract_callback: u64
// ) -> RustSdk2 {
//     setup_logs();
//     let rt = Arc::new(
//         Builder::new_current_thread()
//             .enable_all() // Enables all I/O and time drivers
//             .build()
//             .expect("Failed to create a runtime")
//     );
//
//     // Execute the async block using the Tokio runtime
//     rt.block_on(async {
//         let cfg = Config::new();
//         let sdk = if quorum_public_key_callback != 0 {
//             // use the callbacks to obtain quorum public keys
//             cfg.setup_api_with_callbacks(quorum_public_key_callback, data_contract_callback).await
//         } else {
//             // use Dash Core for quorum public keys
//             cfg.setup_api().await
//         };
//         RustSdk2 {
//             entry_point: Box::new(DashSdk {
//                 config: cfg,
//                 runtime: rt.clone(),
//                 sdk: sdk
//             })
//         }
//     })
// }

// #[ferment_macro::export]
// pub fn create_sdk5(
//     quorum_public_key_callback: u64,
//     data_contract_callback: u64
// ) -> RustSdk5 {
//     setup_logs();
//     let rt = Arc::new(
//         Builder::new_current_thread()
//             .enable_all() // Enables all I/O and time drivers
//             .build()
//             .expect("Failed to create a runtime")
//     );
//
//     // Execute the async block using the Tokio runtime
//     rt.block_on(async {
//         let cfg = Config::new();
//         let sdk = if quorum_public_key_callback != 0 {
//             // use the callbacks to obtain quorum public keys
//             cfg.setup_api_with_callbacks(quorum_public_key_callback, data_contract_callback).await
//         } else {
//             // use Dash Core for quorum public keys
//             cfg.setup_api().await
//         };
//         RustSdk5 {
//             entry_point: Box::into_raw(Box::new(DashSdk {
//                 config: cfg,
//                 runtime: rt.clone(),
//                 sdk: sdk
//             })) as * mut c_void
//         }
//     })
// }

// #[ferment_macro::export]
// #[derive(Clone, Debug)]
// pub struct RustSdk {
//     // pub sdk_pointer: *const c_void,
//     // pub runtime: *const c_void
//     pub sdk_pointer: u64,
//     pub runtime: u64
// }
//
// #[ferment_macro::export]
// pub fn create_rust_sdk(
//     quorum_public_key_callback: u64,
//     data_contract_callback: u64
// ) -> Arc<RustSdk> {
//     let rt = Builder::new_current_thread()
//         .enable_all() // Enables all I/O and time drivers
//         .build()
//         .expect("Failed to create a runtime");
//
//     // Execute the async block using the Tokio runtime
//     rt.block_on(async {
//         let cfg = Config::new();
//         let sdk = cfg.setup_api_with_callbacks(quorum_public_key_callback, data_contract_callback).await;
//
//         let rt = Builder::new_current_thread()
//             .enable_all() // Enables all I/O and time drivers
//             .build()
//             .expect("Failed to create a runtime");
//
//         let rust_sdk = Arc::new(RustSdk {
//             // sdk_pointer: Arc::into_raw(sdk) as *const c_void,
//             // runtime: Arc::into_raw(Arc::new(rt)) as *const c_void
//             sdk_pointer: Arc::into_raw(sdk) as u64,
//             runtime: Arc::into_raw(Arc::new(rt)) as u64
//         });
//         tracing::warn!("after creating sdk: {:?}", rust_sdk);
//         return rust_sdk.into()
//     })
// }
//
// #[ferment_macro::export]
// pub fn destroy_rust_sdk(rust_sdk: Arc<RustSdk>) {
//     let ptr = rust_sdk.sdk_pointer as *mut Arc<Sdk>;
//     if !ptr.is_null() {
//         unsafe {
//             Box::from_raw(ptr);
//         }
//     }
//     let ptr2 = rust_sdk.runtime as *mut Arc<Runtime>;
//     if !ptr2.is_null() {
//         unsafe {
//             Box::from_raw(ptr2);
//         }
//     }
// }

// #[ferment_macro::opaque]
// pub type BlockHashByHeight = unsafe extern "C" fn(u32) -> [u8; 32];

// #[ferment_macro::opaque]
// pub struct FFICoreProvider {
//     pub block_hash_by_height: BlockHashByHeight,
// }
//
// impl CoreProvider for FFICoreProvider {
//     fn get_block_hash_by_height(&self, _height: u32) -> [u8; 32] {
//         [0u8; 32]
//         // (self.block_hash_by_height)(height)
//     }
// }
//
// #[ferment_macro::opaque]
// pub trait CoreProvider {
//     fn get_block_hash_by_height(&self, height: u32) -> [u8; 32];
// }
// #[ferment_macro::opaque]
// pub struct DashSharedCore {
//     pub processor: *mut Processor,
//     pub cache: BTreeMap<String, String>,
//     pub context: *const std::os::raw::c_void,
// }
//
// #[ferment_macro::opaque]
// impl DashSharedCore {
//     pub fn new(
//         context: *const std::os::raw::c_void) -> Self {
//         Self {
//             processor: boxed(Processor { chain_id: Box::new(Config::new()) }),
//             cache: Default::default(),
//             context
//         }
//     }
// }
//
// #[ferment_macro::opaque]
// pub struct Processor {
//     pub chain_id: Box<dyn MyConfig>,
// }



// #[test]
// fn test_rust_sdk() {
//     let my_sdk = create_sdk(0, 0);
//     let my_boxed_sdk = boxed(my_sdk);
//     destroy_sdk(my_boxed_sdk);
// }


