use std::collections::BTreeMap;
use dpp::document::Document;
use dpp::document::v0::DocumentV0;
use dpp::identity::identity_public_key::TimestampMillis;
use dpp::identity::identity_public_key::KeyID;
use dpp::prelude::{BlockHeight, CoreBlockHeight, Revision};
use drive::query::{OrderClause, WhereClause, WhereOperator};
use platform_value::{Value, ValueMap};

#[ferment_macro::export]
pub fn KeyID_clone(id: KeyID) -> KeyID {
    id.clone()
}

#[ferment_macro::export]
pub fn Revision_clone(revision: Revision) -> Revision {
    revision.clone()
}
#[ferment_macro::export]
pub fn TimestampMillis_clone(time: TimestampMillis) -> TimestampMillis {
    time.clone()
}
#[ferment_macro::export]
pub fn CoreBlockHeight_clone(height: CoreBlockHeight) -> CoreBlockHeight {
    height.clone()
}
#[ferment_macro::export]
pub fn BlockHeight_clone(height: BlockHeight) -> BlockHeight {
    height.clone()
}

#[ferment_macro::export]
pub fn Value_clone(value: &Value) -> Value {
    value.clone()
}

#[ferment_macro::export]
pub fn ValueMap_clone(value_map: &ValueMap) -> ValueMap {
    value_map.clone()
}

#[ferment_macro::export]
pub fn std_collections_Map_keys_String_values_platform_value_Value_clone(map: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    map.clone()
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
pub fn DocumentV0_clone(document: &DocumentV0) -> DocumentV0 {
    document.clone()
}

#[ferment_macro::export]
pub fn Arr_u8_32_clone(slice: [u8; 32]) -> [u8; 32] {
    slice.clone()
}

#[ferment_macro::export]
pub fn Arr_u8_20_clone(slice: [u8; 20]) -> [u8; 20] {
    slice.clone()
}

#[ferment_macro::export]
pub fn Arr_u8_36_clone(slice: [u8; 36]) -> [u8; 36] {
    slice.clone()
}

#[ferment_macro::export]
pub fn WhereClause_clone(o: WhereClause) -> WhereClause {
    o.clone()
}

#[ferment_macro::export]
pub fn WhereOperator_clone(o: WhereOperator) -> WhereOperator {
    o.clone()
}
#[ferment_macro::export]
pub fn OrderClause_clone(o: OrderClause) -> OrderClause {
    o.clone()
}