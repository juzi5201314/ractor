use std::any::Any;
use std::fmt::Debug;
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::time::sleep;

use ractor::{Actor, Broker, Context, MessageHandler, StoppingPosition};

#[derive(Debug)]
struct Hello;

#[derive(Default)]
struct MyActor {
    panic: bool,
    id: usize,
}

#[async_trait::async_trait]
impl Actor for MyActor {
    const MAIL_BOX_SIZE: u32 = 10;

    async fn create(_ctx: &mut Context<Self>) -> Self
    where
        Self: Sized,
    {
        static C: AtomicUsize = AtomicUsize::new(0);
        MyActor {
            panic: false,
            id: C.fetch_add(1, Ordering::SeqCst),
        }
    }

    async fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("no.{} started", self.id);
    }

    async fn stopped(&mut self, _ctx: &mut Context<Self>, _pos: StoppingPosition) {
        println!("no.{} stopped", self.id);
    }

    async fn reset(&mut self, _ctx: &mut Context<Self>)
    where
        Self: Sized,
    {
        println!("no.{} reset", self.id);
    }

    fn catch_unwind(&mut self, _err: Box<dyn Any + Send>, ctx: &mut Context<Self>) {
        ctx.reset();
        println!("no.{} catch_unwind", self.id)
    }
}

#[async_trait::async_trait]
impl MessageHandler<Hello> for MyActor {
    type Output = ();

    async fn handle(&mut self, _: Hello, ctx: &mut Context<Self>) -> Self::Output {
        // 避免一个actor把全部消息抢占了
        ctx.r#yield();
        if !self.panic {
            self.panic = true;
            panic!("no.{} expected", self.id);
        } else {
            println!("no.{} no panic", self.id);
        }
    }
}

#[tokio::main]
async fn main() {
    let my_actor = Broker::<MyActor>::spawn(2, true).await;

    my_actor.send(Hello).await.unwrap();
    my_actor.send(Hello).await.unwrap();
    my_actor.send(Hello).await.unwrap();
    my_actor.send(Hello).await.unwrap();

    sleep(Duration::from_secs(1)).await;
    stdout().flush().unwrap()
}
