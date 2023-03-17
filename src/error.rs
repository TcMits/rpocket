#[derive(Debug)]
pub enum RPocketError {
    MutexError,
    SerdeError(serde_json::Error),
}
