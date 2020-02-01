use crate::manager::DynamicConnectionsError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;

pub type UpdateConnectionsResult<T> = Result<T, UpdateConnectionsError>;

#[derive(Debug)]
pub enum UpdateConnectionsError {
    StartThreadError { error: IoError },
    SpawnCommandError { error: IoError },
    ReadOutputError { error: IoError },
    WaitCommandError { error: IoError },
    UpdateConnectionsError { error: DynamicConnectionsError },
}

macro_rules! warn_error {
    ($error:expr) => {{
        let error = $error;

        warn!("{}", error);

        error
    }};
}

impl UpdateConnectionsError {
    pub fn start_thread_error(error: IoError) -> UpdateConnectionsError {
        warn_error!(UpdateConnectionsError::StartThreadError { error })
    }

    pub fn spawn_command_error(error: IoError) -> UpdateConnectionsError {
        warn_error!(UpdateConnectionsError::SpawnCommandError { error })
    }

    pub fn read_output_error(error: IoError) -> UpdateConnectionsError {
        warn_error!(UpdateConnectionsError::ReadOutputError { error })
    }

    pub fn wait_command_error(error: IoError) -> UpdateConnectionsError {
        warn_error!(UpdateConnectionsError::WaitCommandError { error })
    }

    pub fn update_connections_error(error: DynamicConnectionsError) -> UpdateConnectionsError {
        warn_error!(UpdateConnectionsError::UpdateConnectionsError { error })
    }
}

impl Error for UpdateConnectionsError {}

impl Display for UpdateConnectionsError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            UpdateConnectionsError::StartThreadError { error } => {
                write!(f, "Failed to start thread - {}", error)
            }
            UpdateConnectionsError::SpawnCommandError { error } => {
                write!(f, "Failed to spawn command - {}", error)
            }
            UpdateConnectionsError::ReadOutputError { error } => {
                write!(f, "Failed to read command output - {}", error)
            }
            UpdateConnectionsError::WaitCommandError { error } => {
                write!(f, "Failed to wait command - {}", error)
            }
            UpdateConnectionsError::UpdateConnectionsError { error } => {
                write!(f, "Failed to update dynamic connections - {}", error)
            }
        }
    }
}
