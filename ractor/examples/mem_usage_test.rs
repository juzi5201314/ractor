use tokio::runtime::Handle;

use ractor::{Actor, Context, Message, MessageHandler, Stage};

#[derive(Debug, Message)]
struct Sum;

#[derive(Default)]
struct MyActor;

#[async_trait::async_trait]
impl Actor for MyActor {
    const MAIL_BOX_SIZE: u32 = 100;

    async fn create(_ctx: &Context<Self>) -> Self
    where
        Self: Sized,
    {
        MyActor
    }
}

#[async_trait::async_trait]
impl MessageHandler<Sum> for MyActor {
    type Output = ();
    type Error = ();

    async fn handle(&mut self, _msg: Sum, _ctx: &Context<Self>) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stage = Stage::from_handle(Handle::current());

    let _my_actor = stage.spawn::<MyActor>(1_000_000);

    std::io::stdin().read_line(&mut String::new());
}
