use thiserror::Error;

#[derive(Error, Debug, serde::Serialize)]
pub enum ConnError {
    #[error("connect error: {0}")]
    ConnectError(String),
    #[error("send error: {0}")]
    SendError(String),
    #[error("lock error: {0}")]
    LockError(String),
}