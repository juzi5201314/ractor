use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::{deserialize, serialize};

type MessageRegister = HashMap<
    u64,
    Box<
        dyn Fn(&[u8]) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, crate::Error>> + Send>>
            + Send
            + Sync,
    >,
>;

pub struct Server {
    tcp_listener: TcpListener,
    config: WebSocketConfig,
    register: Arc<MessageRegister>,
}

impl Server {
    pub async fn listen(
        addr: SocketAddr,
        config: Option<WebSocketConfig>,
        register: MessageRegister,
    ) -> Result<Self, super::WsError> {
        let tcp_listener = TcpListener::bind(addr).await?;
        Ok(Server {
            tcp_listener,
            config: config.unwrap_or_default(),
            register: Arc::new(register),
        })
    }

    pub async fn run(self) {
        while let Ok((stream, _peer_addr)) = self.tcp_listener.accept().await {
            let stream = match tokio_tungstenite::accept_async_with_config(
                stream,
                Some(self.config.clone()),
            )
            .await
            {
                Ok(s) => s,
                Err(err) => {
                    dbg!("failed upgrade to websocket request");
                    dbg!(err);
                    continue;
                }
            };

            tokio::spawn(Self::handle_connection(stream, self.register.clone()));
        }
    }

    async fn handle_connection(stream: WebSocketStream<TcpStream>, register: Arc<MessageRegister>) {
        let (mut outgoing, mut incoming) = stream.split();

        while let Some(Ok(msg)) = incoming.next().await {
            match msg {
                Message::Binary(msg) => {
                    let msg: crate::Message = match deserialize(&msg) {
                        Ok(msg) => msg,
                        Err(err) => {
                            dbg!(err);
                            dbg!("Unexpected message header. deserialize failed");
                            continue;
                        }
                    };
                    match register.get(&msg.identity_id) {
                        None => continue,
                        Some(f) => outgoing
                            .send(Message::Binary(
                                serialize(&crate::Message {
                                    identity_id: 0,
                                    unique_id: msg.unique_id,
                                    payload: f(&msg.payload).await.unwrap(),
                                })
                                .unwrap(),
                            ))
                            .await
                            .map_err(Into::<crate::WsError>::into)
                            .map_err(Into::<crate::Error>::into)
                            .unwrap(),
                    };
                }
                _ => {}
            }
        }
    }

    #[inline]
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.tcp_listener.local_addr()
    }
}
