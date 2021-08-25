use quinn::Endpoint as QEndpoint;
use std::net::{SocketAddr, IpAddr};
use super::QuicError;

pub struct Endpoint {
    endpoint: QEndpoint,
}

impl Endpoint {
    pub fn connect(config: EndpointConfig, peer_addr: SocketAddr) -> Result<Self, QuicError> {
        // todo: config
        let (endpoint, incoming) = QEndpoint::builder().bind(&config.bind_addr)?;

        todo!()
    }

    fn bind_addr(ip_addr: IpAddr) -> SocketAddr {
        // random bind port
        SocketAddr::new(ip_addr, 0)
    }
}

pub struct EndpointConfig {
    bind_addr: SocketAddr
}
