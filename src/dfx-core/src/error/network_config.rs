use crate::error::io::IoError;
use crate::error::socket_addr_conversion::SocketAddrConversionError;
use std::num::ParseIntError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkConfigError {
    #[error("Network '{0}' does not specify any network providers.")]
    NetworkHasNoProviders(String),

    #[error("The '{0}' network must be a local network.")]
    NetworkMustBeLocal(String),

    #[error("Cannot find network context.")]
    NoNetworkContext(),

    #[error("Did not find any providers for network {0}")]
    NoProvidersForNetwork(String),

    #[error("Failed to parse bind address: {0}")]
    ParseBindAddressFailed(SocketAddrConversionError),

    #[error("Failed to parse : {0}")]
    ParseProviderUrlFailed(Box<String>, url::ParseError),

    #[error("Failed to parse contents of {0} as a port value")]
    ParsePortValueFailed(Box<PathBuf>, Box<ParseIntError>),

    #[error("Failed to read webserver port: {0}")]
    ReadWebserverPortFailed(IoError),
}
