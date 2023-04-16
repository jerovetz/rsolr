use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub struct RSCError {
    pub source: Box<dyn Error>,
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
        Some(&*self.source)
    }
}

impl RSCError {
    pub fn kind(&self) -> &str {
        "RSCError"
    }
}