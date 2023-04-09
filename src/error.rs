use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum RPocketError {
    MutexError,
    SerdeError(serde_json::Error),
    RequestError(reqwest::Error),
    UrlError(url::ParseError),
    APIError(APIError),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct APIError {
    pub code: i64,
    pub message: String,
    pub data: serde_json::Value,
}
