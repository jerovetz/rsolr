use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use reqwest::StatusCode;

/// Errors that may occur in the Solr interaction.
pub struct RSCError {
    pub source: Option<Box<dyn Error>>,
    pub status: Option<StatusCode>,
    pub message: Option<String>
}

impl Debug for RSCError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut builder = f.debug_struct("error:RSCError");
        builder.field("source", &self.source());
        builder.finish()
    }
}

impl Display for RSCError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut builder = f.debug_struct("error:RSCError");
        builder.field("source", &self.source());
        builder.finish()
    }
}

impl Error for RSCError {
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

impl RSCError {
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
        let static_message = Box::leak(self.message.as_ref().unwrap().clone().into_boxed_str());
        Some(static_message)
    }


}