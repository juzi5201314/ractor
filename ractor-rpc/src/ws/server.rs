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
use crate::error::Error;

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
        // 注意: 在连接数量增多时会占用过多内存, 是否有更好方法不需要保存全部JoinHandle？
        // 收集所有connection的JoinHandle, 并在结束时终止. 避免server已经关闭但`handle_connection`future还在运行.
        let mut handles = Vec::new();
        while let Ok((stream, _peer_addr)) = self.tcp_listener.accept().await {
            let stream = match tokio_tungstenite::accept_async_with_config(
                stream,
                Some(self.config.clone()),
            )
            .await
            {
                Ok(s) => s,
                Err(err) => {
                    log::warn!("receive a tcp connection but cannot upgrade to a websocket connection.");
                    log::debug!(err);
                    continue;
                }
            };

            handles.push(tokio::spawn(Self::handle_connection(stream, self.register.clone())));
        }
        handles.into_iter().for_each(|handle| handle.abort());
    }

    async fn handle_connection(stream: WebSocketStream<TcpStream>, register: Arc<MessageRegister>) {
        let (mut outgoing, mut incoming) = stream.split();

        while let Some(Ok(msg)) = incoming.next().await {
            match msg {
                Message::Binary(msg) => {
                    let msg: crate::Message = match deserialize(&msg) {
                        Ok(msg) => msg,
                        Err(err) => {
                            log::warn!("the client received a message that could not be deserialized. Discarded");
                            log::debug!(err);
                            continue;
                        }
                    };
                    match register.get(&msg.identity_id) {
                        None => {
                            log::warn!("received a message that is not in the message registry. id: {}", &msg.identity_id);
                            continue
                        },
                        Some(f) => match outgoing
                            .send(Message::Binary(
                                serialize(&crate::Message {
                                    identity_id: 0,
                                    unique_id: msg.unique_id,
                                    payload: match f(&msg.payload).await {
                                        Ok(t) => t,
                                        Err(err) => {
                                            log::warn!("failed to forward the message to the local.");
                                            log::debug!(err);
                                            continue
                                        }
                                    },
                                })
                                .expect("Unable to deserialize message"),
                            ))
                            .await
                            .map_err(Into::<crate::WsError>::into)
                            .map_err(Into::<crate::Error>::into) {
                            Ok(_) => {}
                            Err(err) => {
                                log::warn!("unable to respond to the message.");
                                log::debug!(err);
                                break
                            }
                        }
                        ,
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
