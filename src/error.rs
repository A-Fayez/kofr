use thiserror::Error;

pub type Result<T> = std::result::Result<T, KofrError>;

#[derive(Error, Debug)]
pub enum KofrError {
    #[error("No connector with name: {0} was found")]
    ConnectorNotFound(String),
    #[error("could not deserialize the json returned")]
    DeserializeConnectorError(#[from] serde_json::Error),
    #[error("Response returned from {0} is not a json array")]
    NotAJsonArrayError(String),
    #[error("error while requesting uri, {0}")]
    ApiError(#[from] ureq::Error),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
}
