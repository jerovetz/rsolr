use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use reqwest::StatusCode;

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
    SolrSyntax
}

impl RSCError {
    pub fn kind(&self) -> ErrorKind {
        if self.source.is_some() {
           return ErrorKind::Network
        }
        ErrorKind::NotFound
    }

    pub fn status(&self) -> Option<StatusCode> {
        self.status
    }

    pub fn message(&self) -> Option<&str> {
        let static_message = Box::leak(self.message.as_ref().unwrap().clone().into_boxed_str());
        Some(static_message)
    }


}