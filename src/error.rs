use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub struct RSCError {
    pub source: Box<dyn Error>,
}

impl Debug for RSCError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> FmtResult {
        todo!()
    }
}

impl Display for RSCError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> FmtResult {
        todo!()
    }
}

impl Error for RSCError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.source)
    }
}

impl RSCError {
    pub fn kind(&self) -> &str {
        "RSCError"
    }
}