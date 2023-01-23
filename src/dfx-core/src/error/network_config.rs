use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkConfigError {
    #[error("Failed to parse : {0}")]
    ParseProviderUrlFailed(Box<String>, url::ParseError),

    #[error("Did not find any providers for network {0}")]
    NoProvidersForNetwork(String),
}
