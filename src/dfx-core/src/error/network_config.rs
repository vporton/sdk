use crate::error::io::IoError;
use std::num::ParseIntError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkConfigError {
    #[error("Failed to parse : {0}")]
    ParseProviderUrlFailed(Box<String>, url::ParseError),

    #[error("Did not find any providers for network {0}")]
    NoProvidersForNetwork(String),

    #[error("Failed to parse contents of {0} as a port value")]
    ParsePortValueFailed(Box<PathBuf>, Box<ParseIntError>),

    #[error("Failed to read webserver port: {0}")]
    ReadWebserverPortFailed(IoError),
}
