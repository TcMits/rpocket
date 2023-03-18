use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum RPocketError {
    // internal errors
    MutexError,
    SerdeError(serde_json::Error),
    RequestError(reqwest::Error),
    UrlError(url::ParseError),

    // api errors
    APIError(APIError),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct APIError {
    pub code: i64,
    pub message: String,
    pub data: serde_json::Value,
}
