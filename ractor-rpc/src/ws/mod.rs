pub mod client;
pub mod server;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WsError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("websocket error: {0}")]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),
}
