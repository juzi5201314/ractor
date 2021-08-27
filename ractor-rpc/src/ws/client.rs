use std::collections::HashMap;
use std::sync::Arc;

use crossfire::mpsc::{RxFuture, SharedSenderBRecvF};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::Message as WMessage;
use tokio_tungstenite::{connect_async_with_config, MaybeTlsStream, WebSocketStream};
use url::Url;

use crate::ws::WsError;
use crate::{deserialize, serialize, Message, RemoteType};

pub struct Client {
    sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WMessage>,
    recv_handle: JoinHandle<()>,

    // todo: timeout delete
    // todo: optimize mutex
    // 等待响应的消息表
    msg_table:
        Arc<Mutex<HashMap<u128, Box<dyn FnOnce(&[u8]) -> Result<(), crate::Error> + Send + Sync>>>>,
}

impl Client {
    pub async fn connect(url: Url, config: Option<WebSocketConfig>) -> Result<Self, WsError> {
        let (stream, _) = connect_async_with_config(url, config).await?;
        let (sink, stream) = stream.split();

        let msg_table = Arc::new(Mutex::new(HashMap::new()));

        let handle = tokio::spawn(Client::listen_response_message(
            stream,
            Arc::clone(&msg_table),
        ));

        Ok(Client {
            sink,
            recv_handle: handle,
            msg_table,
        })
    }

    async fn listen_response_message(
        mut stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        msg_table: Arc<
            Mutex<HashMap<u128, Box<dyn FnOnce(&[u8]) -> Result<(), crate::Error> + Send + Sync>>>,
        >,
    ) {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                WMessage::Text(_) => {}
                WMessage::Binary(msg) => {
                    let msg: Message = match deserialize(&msg) {
                        Ok(msg) => msg,
                        Err(err) => {
                            log::warn!("the client received a message that could not be deserialized. Discarded");
                            log::debug!("{}", err);
                            continue;
                        }
                    };

                    match msg_table.lock().await.remove(&msg.unique_id) {
                        None => {
                            log::warn!("a message that does not exist in the message table was received. It may be that the response has expired or is a forged message.");
                            continue;
                        }
                        Some(resp) => {
                            resp(&msg.payload).unwrap();
                        }
                    }
                }
                WMessage::Ping(_) => {}
                WMessage::Pong(_) => {}
                WMessage::Close(_) => {}
            }
        }
    }

    #[allow(unused)]
    async fn ping(&mut self) -> Result<(), WsError> {
        self.sink.send(WMessage::Ping(Vec::new())).await?;
        Ok(())
    }

    pub async fn send<O>(
        &mut self,
        msg: Message,
    ) -> Result<RxFuture<O, SharedSenderBRecvF>, crate::Error>
    where
        O: RemoteType + 'static,
    {
        let a = serialize(&msg)?;
        self.sink
            .send(WMessage::Binary(a))
            .await
            .map_err(Into::<WsError>::into)?;

        let (tx, rx) = crossfire::mpsc::bounded_tx_blocking_rx_future(1);

        self.msg_table.lock().await.insert(
            msg.unique_id,
            Box::new(move |bytes: &[u8]| {
                tx.try_send(deserialize::<O>(bytes)?)
                    .map_err(|_| (/* Response is discarded */))
                    .ok();
                Ok(())
            }),
        );
        Ok(rx)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.recv_handle.abort()
    }
}
