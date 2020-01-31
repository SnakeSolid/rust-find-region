use postgres::Error as PgError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub type DatabaseResult<T> = Result<T, DatabaseError>;

#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError { message: String },
    QueryExecutionError { message: String },
    ValueError { message: String },
}

impl DatabaseError {
    pub fn connection_error(error: PgError) -> DatabaseError {
        debug!("Connection error - {}", error);

        DatabaseError::ConnectionError {
            message: format!("{}", error),
        }
    }

    pub fn query_execution_error(error: PgError) -> DatabaseError {
        debug!("Query execution error - {}", error);

        DatabaseError::QueryExecutionError {
            message: format!("{}", error),
        }
    }

    pub fn value_error(error: PgError) -> DatabaseError {
        debug!("Value error - {}", error);

        DatabaseError::ValueError {
            message: format!("{}", error),
        }
    }
}

impl Error for DatabaseError {}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            DatabaseError::ConnectionError { message } => write!(f, "{}", message),
            DatabaseError::QueryExecutionError { message } => write!(f, "{}", message),
            DatabaseError::ValueError { message } => write!(f, "{}", message),
        }
    }
}
