use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RSolrError {
    #[error("Connection problem occurred.")]
    Network {
        #[source]
        source: reqwest::Error,
    },
    #[error("Solr cannot find the requested resource.")]
    NotFound,
    #[error("Syntax error in Solr request: `{0}`")]
    Syntax(String),
    #[error("Generic Solr error.")]
    Other {
        #[source]
        source: Box<dyn Error>,
        status: http::StatusCode,
        body_text: String
    },
    #[error("JSON deserialization failed: `{0}`")]
    Serialization(String)
}