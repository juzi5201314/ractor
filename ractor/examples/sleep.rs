use std::time::Duration;

use futures::future::{join_all, FutureExt};
use tokio::time::timeout;

use ractor::{Actor, AsyncMessageHandler, Message, MessageHandler, StageBuilder};

#[derive(Debug, Message)]
struct Sleep(u64);

#[derive(Default)]
struct MyActor;

impl Actor for MyActor {
    const MAIL_BOX_SIZE: u32 = 101;
}

impl MessageHandler<Sleep> for MyActor {
    type Output = ();

    fn handle(&mut self, msg: Sleep) -> Self::Output {
        std::thread::sleep(Duration::from_secs(msg.0))
    }
}

#[async_trait::async_trait]
impl AsyncMessageHandler<Sleep> for MyActor {
    type Output = ();

    async fn handle(&mut self, msg: Sleep) -> Self::Output {
        tokio::time::sleep(Duration::from_secs(msg.0)).await
    }
}
// We have 100 actors and process 200 messages, and each message takes 1 second.
// Since they are parallel, they should be completed in about 2 seconds
fn main() {
    let stage = StageBuilder::new().build();

    stage.block_on(|scenes| async move {
        let my_actor = scenes.spawn::<MyActor>(100);
        // send 100 sync sleep message.
        let mut resp_handles = join_all(
            (0..100).map(|_| my_actor.send(Sleep(1)).map(|res| res.expect("send failed"))),
        )
        .await;
        // send 100 async sleep message.
        resp_handles.extend(
            join_all((0..100).map(|_| {
                my_actor
                    .send_async(Sleep(1))
                    .map(|res| res.expect("send failed"))
            }))
            .await,
        );

        // Taking into account other costs, it should be completed within 2.2 seconds
        let res = timeout(
            Duration::from_secs_f64(2.2),
            join_all(resp_handles.into_iter().map(|handle| handle.recv())),
        )
        .await;

        assert!(res.is_ok());
    })
}
