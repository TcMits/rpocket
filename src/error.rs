use serde::{Deserialize, Serialize};

/// error struct returned by the Pocket API.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct APIError {
    pub code: i64,
    pub message: String,
    pub data: serde_json::Value,
}

/// error type for the RPocket library.
#[derive(Debug)]
pub enum RPocketError {
    MutexError,
    SerdeError(serde_json::Error),
    RequestError(reqwest::Error),
    UrlError(url::ParseError),
    APIError(APIError),
    Error(Box<dyn std::error::Error + Send + Sync>),
}

impl From<serde_json::Error> for RPocketError {
    fn from(error: serde_json::Error) -> Self {
        RPocketError::SerdeError(error)
    }
}

impl From<reqwest::Error> for RPocketError {
    fn from(error: reqwest::Error) -> Self {
        RPocketError::RequestError(error)
    }
}

impl From<url::ParseError> for RPocketError {
    fn from(error: url::ParseError) -> Self {
        RPocketError::UrlError(error)
    }
}

impl From<APIError> for RPocketError {
    fn from(error: APIError) -> Self {
        RPocketError::APIError(error)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for RPocketError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        RPocketError::Error(error)
    }
}

impl std::fmt::Display for RPocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RPocketError::MutexError => write!(f, "mutex error"),
            RPocketError::SerdeError(error) => write!(f, "serde error: {}", error),
            RPocketError::RequestError(error) => write!(f, "request error: {}", error),
            RPocketError::UrlError(error) => write!(f, "url error: {}", error),
            RPocketError::APIError(error) => write!(f, "API error: {}", error.message),
            RPocketError::Error(error) => write!(f, "error: {}", error),
        }
    }
}

impl std::error::Error for RPocketError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RPocketError::MutexError => None,
            RPocketError::SerdeError(error) => Some(error),
            RPocketError::RequestError(error) => Some(error),
            RPocketError::UrlError(error) => Some(error),
            RPocketError::APIError(..) => None,
            RPocketError::Error(error) => Some(error.as_ref()),
        }
    }
}
