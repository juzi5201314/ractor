use std::any::Any;

use ractor::{Actor, Broker, Context, MessageHandler, StoppingPosition};
use std::fmt::Debug;

#[derive(Debug)]
struct Hello;

#[derive(Default)]
struct MyActor {
    panic: bool,
}

#[async_trait::async_trait]
impl Actor for MyActor {
    const MAIL_BOX_SIZE: u32 = 10;

    async fn create(_ctx: &mut Context<Self>) -> Self
    where
        Self: Sized,
    {
        MyActor { panic: false }
    }

    async fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("started");
    }

    async fn stopped(&mut self, _ctx: &mut Context<Self>, _pos: StoppingPosition) {
        println!("stopped");
    }

    async fn reset(&mut self, _ctx: &mut Context<Self>)
    where
        Self: Sized,
    {
        println!("reset");
    }

    fn catch_unwind(&mut self, _err: Box<dyn Any + Send>, ctx: &mut Context<Self>) {
        ctx.reset();
        println!("catch_unwind");
    }
}

#[async_trait::async_trait]
impl MessageHandler<Hello> for MyActor {
    type Output = ();

    async fn handle(&mut self, _: Hello, _ctx: &mut Context<Self>) -> Self::Output {
        if !self.panic {
            self.panic = true;
            panic!("expected");
        } else {
            println!("no panic");
        }
    }
}

#[tokio::main]
async fn main() {
    let my_actor = Broker::<MyActor>::spawn_one().await;

    my_actor.send(Hello).await.unwrap();
    my_actor.send(Hello).await.unwrap();
}
