use ractor::{Actor, Context, Message, MessageHandler, Broker};

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
    let _my_actor = Broker::<MyActor>::spawn(1_000_000).await;

    // 查看任务管理器, 估算占用内存大小
    // 404 byte
    println!("done");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}
