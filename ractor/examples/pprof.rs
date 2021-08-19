use futures::future::{join_all, FutureExt};
use tokio::runtime::Handle;

use ractor::{Actor, Context, Message, MessageHandler, Stage};

#[derive(Debug, Message)]
struct Sum(u64, u64);

#[derive(Default)]
struct MyActor;

#[async_trait::async_trait]
impl Actor for MyActor {
    const MAIL_BOX_SIZE: u32 = 100;

    fn create(_ctx: &Context<Self>) -> Self
    where
        Self: Sized,
    {
        MyActor
    }
}

#[async_trait::async_trait]
impl MessageHandler<Sum> for MyActor {
    type Output = u64;

    async fn handle(&mut self, msg: Sum, _ctx: &Context<Self>) -> Self::Output {
        msg.0 + msg.1
    }
}

#[tokio::main]
async fn main() {
    let guard = pprof::ProfilerGuard::new(100).unwrap();

    let stage = Stage::from_handle(Handle::current());

    let my_actor = stage.spawn::<MyActor>(10);

    let resp_handles = join_all((0..200).map(|_| {
        my_actor
            .send(Sum(10, 5))
            .map(|res| res.expect("send failed"))
    }))
    .await;

    join_all(resp_handles.into_iter().map(|handle| handle.recv())).await;

    if let Ok(report) = guard.report().build() {
        let file = std::fs::File::create("flamegraph.svg").unwrap();
        report.flamegraph(file).unwrap();
    };
}
