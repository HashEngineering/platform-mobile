pub mod fetch_identity;
pub mod identity;
mod config;
mod provider;
pub mod clone;
pub mod operators;
pub mod fetch_document;
pub mod put;
pub mod custom;
pub mod data_contracts;
mod put_test;

extern crate ferment_macro;

use drive_proof_verifier::ContextProvider;
use platform_value::BinaryData;
use dash_sdk::platform::types::identity::PublicKeyHash;
use dpp::bincode::{Decode, Encode};
use platform_value::{Hash256, Value};

#[ferment_macro::export]
pub fn convert_to_pkh(pkh: [u8; 20]) -> PublicKeyHash {
    PublicKeyHash(pkh)
}

#[ferment_macro::export]
pub fn get_binary_data() -> BinaryData {
    BinaryData::new(vec![])
}
#[ferment_macro::export]
pub fn get_binary_data2() -> BinaryData {
     BinaryData(vec![0, 1, 2, 3])
}

#[ferment_macro::export]
pub fn get_platform_value() -> Value {
    Value::Bool(false)
}

#[ferment_macro::export]
pub fn get_platform_value_with_map() ->  Value {
    Value::Map(vec![(Value::Text("key".to_string()), Value::I32(4))])
}