
use serde::{Deserialize};
use serde_json::{json, Value};
use crate::facet_fields::FacetFields;

/// The response part of the server response body.
#[derive(Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Response<T> {
    pub numFound: u32,
    pub start: u32,
    pub numFoundExact: bool,
    pub docs: Vec<T>
}

/// The facet part of the response. Facet_fields is parsed, see there.
#[derive(Deserialize, Clone, Debug)]
pub struct Facet {
    pub facet_queries: Value,
    pub facet_fields: FacetFields
}

/// The rendered response body. It uses the default writer: [JSON Response Writer](https://solr.apache.org/guide/8_1/response-writers.html#json-response-writer).
#[derive(Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct SolrResponse<T> where T: Clone {
    #[serde(default = "empty_response")]
    pub response: Option<Response<T>>,
    #[serde(default = "empty_facet_counts")]
    pub facet_counts: Option<Facet>,
    pub nextCursorMark: Option<String>,

    /// Container for remaining fields.
    #[serde(flatten)]
    pub raw: Value
}

impl<T> Default for SolrResponse<T> where T: Clone {
    fn default() -> Self {
        SolrResponse { response: None, facet_counts: None, nextCursorMark: None, raw: json!("{}") }
    }
}

fn empty_response<T>() -> Option<Response<T>> {
    None
}
fn empty_facet_counts() -> Option<Facet> { None }