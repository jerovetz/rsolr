use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use reqwest::StatusCode;

/// Errors that may occur in the Solr interaction.
pub struct RSolrError {
    pub source: Option<Box<dyn Error>>,
    pub status: Option<StatusCode>,
    pub message: Option<String>
}

impl Debug for RSolrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut builder = f.debug_struct("error:RSolrError");
        builder.field("source", &self.source());
        builder.field("status", &self.status());
        builder.field("message", &self.message());
        builder.finish()
    }
}

impl Display for RSolrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut builder = f.debug_struct("error:RSolrError");
        builder.field("source", &self.source());
        builder.finish()
    }
}

impl Error for RSolrError {
    /// Gets original error, which generally comes
    /// from JSON encoding/decoding or from the HTTP communication.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return match &self.source {
            Some(source) => Some(&**source),
            None => None
        };
    }
}

pub enum ErrorKind {
    Network,
    NotFound,
    SolrSyntax,
    Other
}

impl RSolrError {
    pub fn kind(&self) -> ErrorKind {
        if self.source.is_some() && self.status.is_none() {
           return ErrorKind::Network
        }

        if self.status.unwrap() == StatusCode::NOT_FOUND {
            return ErrorKind::NotFound
        }

        ErrorKind::Other
    }

    /// Gets the HTTP status code the client has.
    pub fn status(&self) -> Option<StatusCode> {
        self.status
    }

    /// Gets the error message, which could be a Solr server error message, a JSON processing error text
    /// or a raw text body, if client can't parse it as JSON.
    pub fn message(&self) -> Option<&str> {
        match self.message.clone() {
            Some(message) => Some(Box::leak(message.into_boxed_str())),
            _ => None
        }
    }
}