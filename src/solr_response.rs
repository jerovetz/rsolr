use std::collections::HashMap;
use serde::{Deserialize};
use serde_json::Value;

/// The wrapper of the successful response. It holds the response of the [JSON Response Writer](https://solr.apache.org/guide/8_1/response-writers.html#json-response-writer).
#[derive(Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Response<T> {
    pub numFound: u32,
    pub start: u32,
    pub numFoundExact: bool,
    pub docs: Vec<T>
}

#[derive(Deserialize, Clone, Debug)]
pub struct Facet {
    pub facet_queries: HashMap<String, u64>,
    pub facet_fields: Value
}

#[derive(Deserialize, Clone, Debug)]
pub struct SolrResponse<T> where T: Clone {
    #[serde(default = "empty_response")]
    pub response: Option<Response<T>>,
    #[serde(default = "empty_facet_counts")]
    pub facet_counts: Option<Facet>
}

fn empty_response<T>() -> Option<Response<T>> {
    None
}
fn empty_facet_counts() -> Option<Facet> { None }