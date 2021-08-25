use std::net::SocketAddr;

use thiserror::Error;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use url::Url;

use ractor_rpc::{RemoteType, RpcClient, RpcServer};

use crate::{Actor, MessageHandler, ResponseHandle};
use crate::address::LocalAddress;
use crate::message::Message;
use tokio::task::JoinHandle;

#[derive(Debug, Error)]
pub enum RemoteAddressError {
    #[error("rpc error: {0}")]
    Rpc(#[from] ractor_rpc::Error),
}

pub struct RemoteAddress {
    client: RpcClient,
}

impl RemoteAddress {
    pub async fn connect(url: Url) -> Result<Self, ractor_rpc::Error> {
        Ok(RemoteAddress {
            client: RpcClient::connect(url, None).await?,
        })
    }

    pub async fn connect_with_config(
        url: Url,
        config: WebSocketConfig,
    ) -> Result<Self, ractor_rpc::Error> {
        Ok(RemoteAddress {
            client: RpcClient::connect(url, Some(config)).await?,
        })
    }

    #[inline]
    pub async fn send<M, H>(
        &mut self,
        msg: M,
    ) -> Result<ResponseHandle<H::Output>, RemoteAddressError>
    where
        M: Message + RemoteType,
        H: MessageHandler<M>,
        H::Output: RemoteType,
    {
        let rx = self.client.send(ractor_rpc::Message::new(msg)).await?;

        Ok(ResponseHandle(rx))
    }
}

/// drop后会停止监听rpc服务器
pub struct RemoteAddressServer {
    local_addr: SocketAddr,
    handle: JoinHandle<()>
}

impl RemoteAddressServer {
    pub async fn from_local<A>(
        local_address: LocalAddress<A>,
        addr: SocketAddr,
    ) -> Result<Self, ractor_rpc::Error>
    where
        A: Actor,
    {
        let register = A::msg_register(local_address);

        let server = RpcServer::listen(addr, None, register.0).await?;
        let addr = server.local_addr()?;

        Ok(RemoteAddressServer {
            local_addr: addr,
            handle: tokio::spawn(server.run())
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}


impl Drop for RemoteAddressServer {
    fn drop(&mut self) {
        self.handle.abort()
    }
}
