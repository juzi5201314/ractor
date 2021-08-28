use ractor::{Actor, Broker, Context, MessageHandler};

#[derive(Debug)]
struct Reset;

#[derive(Default)]
struct MyActor {
    counter: usize,
}

#[async_trait::async_trait]
impl Actor for MyActor {
    const MAIL_BOX_SIZE: u32 = 10;
    type Args = ();

    async fn create(_ctx: &mut Context<Self>, _args: Self::Args) -> Self
    where
        Self: Sized,
    {
        MyActor { counter: 0 }
    }

    async fn reset(&mut self, _ctx: &mut Context<Self>)
    where
        Self: Sized,
    {
        self.counter += 1;
    }
}

#[async_trait::async_trait]
impl MessageHandler<Reset> for MyActor {
    type Output = usize;

    async fn handle(&mut self, _: Reset, ctx: &mut Context<Self>) -> Self::Output {
        if self.counter < 3 {
            ctx.reset();
        }

        self.counter
    }
}

#[tokio::main]
async fn main() {
    let my_actor = Broker::<MyActor>::spawn_one().await;

    assert_eq!(my_actor.call(Reset).await.unwrap(), 0);
    assert_eq!(my_actor.call(Reset).await.unwrap(), 1);
    assert_eq!(my_actor.call(Reset).await.unwrap(), 2);
    assert_eq!(my_actor.call(Reset).await.unwrap(), 3);
    assert_eq!(my_actor.call(Reset).await.unwrap(), 3);
}
