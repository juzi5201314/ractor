use std::time::Duration;

use futures::future::join_all;
use futures::FutureExt;
use tokio::time::timeout;

use ractor::{Actor, Broker, Context, Message, MessageHandler};

#[derive(Debug, Message)]
struct Sleep(u64);

#[derive(Default)]
struct MyActor;

#[async_trait::async_trait]
impl Actor for MyActor {
    const MAIL_BOX_SIZE: u32 = 2;

    async fn create(_ctx: &Context<Self>) -> Self
    where
        Self: Sized,
    {
        MyActor
    }
}

#[async_trait::async_trait]
impl MessageHandler<Sleep> for MyActor {
    type Output = ();

    async fn handle(
        &mut self,
        msg: Sleep,
        _ctx: &Context<Self>,
    ) -> Self::Output {
        tokio::time::sleep(Duration::from_secs(msg.0)).await
    }
}

// We have 100 actors and process 200 messages, and each message takes 1 second.
// Since they are parallel, they should be completed in about 2 seconds
#[tokio::main]
async fn main() {
    let my_actor = Broker::<MyActor>::spawn(100).await;

    // Taking into account other costs, it should be completed within 2.2 seconds
    let res = timeout(Duration::from_secs_f64(2.2), async {
        // send 200 sleep message.
        let resp_handles = join_all(
            (0..200).map(|_| my_actor.send(Sleep(1)).map(|res| res.expect("send failed"))),
        )
        .await;

        join_all(resp_handles.into_iter().map(|handle| handle.recv())).await;
    })
    .await;

    assert!(res.is_ok());
}
