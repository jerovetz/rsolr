use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct SolrResult<T> {
    pub numFound: u32,
    pub start: u32,
    pub numFoundExact: bool,
    pub docs: Vec<T>
}

#[derive(Deserialize, Clone)]
pub struct Response<T> where T: Clone {
    #[serde(default = "empty_result")]
    pub response: Option<SolrResult<T>>,
}

fn empty_result<T>() -> Option<SolrResult<T>> {
    None
}