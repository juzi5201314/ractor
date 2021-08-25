use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("websocket error: {0}")]
    WsError(#[from] crate::WsError),

    #[error("de/serialize error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("")]
    HandlerPanic,

    #[error("Error when forwarding to local")]
    ForwardToLocal,
}
