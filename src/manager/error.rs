use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub type DynamicConnectionsResult<T> = Result<T, DynamicConnectionsError>;

#[derive(Debug)]
pub struct DynamicConnectionsError {
    message: String,
}

impl DynamicConnectionsError {
    pub fn new(message: &str) -> DynamicConnectionsError {
        DynamicConnectionsError {
            message: message.into(),
        }
    }
}

impl Error for DynamicConnectionsError {}

impl Display for DynamicConnectionsError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message)
    }
}
