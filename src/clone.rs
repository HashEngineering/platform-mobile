use dpp::document::Document;
use platform_value::{Value, ValueMap};

#[ferment_macro::export]
pub fn Value_clone(value: &Value) -> Value {
    value.clone()
}

#[ferment_macro::export]
pub fn ValueMap_clone(value_map: &ValueMap) -> ValueMap {
    value_map.clone()
}

#[ferment_macro::export]
pub fn Vec_u8_clone(vec: &Vec<u8>) -> Vec<u8> {
    vec.clone()
}

#[ferment_macro::export]
pub fn Document_clone(document: &Document) -> Document {
    document.clone()
}
#[ferment_macro::export]
pub fn Arr_u8_32_clone(slice: [u8; 32]) -> [u8; 32] {
    slice.clone()
}

#[ferment_macro::export]
pub fn Arr_u8_20_clone(slice: [u8; 36]) -> [u8; 36] {
    slice.clone()
}

#[ferment_macro::export]
pub fn Arr_u8_36_clone(slice: [u8; 36]) -> [u8; 36] {
    slice.clone()
}