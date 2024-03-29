pub mod fetch_identity;
pub mod identity;
mod config;
mod provider;

extern crate ferment_macro;

use drive_proof_verifier::ContextProvider;
use platform_value::types::binary_data::BinaryData;


#[ferment_macro::export]
pub fn get_binary_data() -> BinaryData {
    BinaryData::new(vec![])
}
#[ferment_macro::export]
pub fn get_binary_data2() -> BinaryData {
     BinaryData(vec![0, 1, 2, 3])
}
