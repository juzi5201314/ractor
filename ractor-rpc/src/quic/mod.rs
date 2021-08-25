pub mod endpoint;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum QuicError {
    #[error("endpoint error: {0}")]
    Endpoint(quinn::EndpointError)
}
