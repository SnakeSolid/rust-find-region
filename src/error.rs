use crate::config::ConfigError;
use crate::worker::UpdateConnectionsError;
use iron::error::HttpError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub type ApplicationResult = Result<(), ApplicationError>;

#[derive(Debug)]
pub enum ApplicationError {
    LoadConfigError { message: String },
    ConfigError { message: String },
    UpdateConnectionsError { message: String },
    ServerError { message: String },
}

impl ApplicationError {
    pub fn read_config_error(error: ConfigError) -> ApplicationError {
        error!("Failed to read configuration - {}", error);

        ApplicationError::LoadConfigError {
            message: format!("{}", error),
        }
    }

    pub fn config_error(error: ConfigError) -> ApplicationError {
        error!("Invalid configuration - {}", error);

        ApplicationError::ConfigError {
            message: format!("{}", error),
        }
    }

    pub fn update_connections_error(error: UpdateConnectionsError) -> ApplicationError {
        error!("Update connections error - {}", error);

        ApplicationError::UpdateConnectionsError {
            message: format!("{}", error),
        }
    }

    pub fn server_error(error: HttpError) -> ApplicationError {
        error!("Server error - {}", error);

        ApplicationError::ServerError {
            message: format!("{}", error),
        }
    }
}

impl Error for ApplicationError {}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ApplicationError::LoadConfigError { message } => write!(f, "{}", message),
            ApplicationError::ConfigError { message } => write!(f, "{}", message),
            ApplicationError::UpdateConnectionsError { message } => write!(f, "{}", message),
            ApplicationError::ServerError { message } => write!(f, "{}", message),
        }
    }
}
