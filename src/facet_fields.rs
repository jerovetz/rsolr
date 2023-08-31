use std::fmt::Debug;
use serde::{Deserialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize)]
pub struct FacetFields {
    #[serde(flatten)]
    pub fields: Value
}
