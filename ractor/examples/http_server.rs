use std::convert::Infallible;

use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Request, Response};
use tokio::net::{TcpListener, TcpStream};

use ractor::{Actor, Context, Message, MessageHandler, Broker};

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[tokio::main]
async fn main() {
    let my_actor = Broker::<MyActor>::spawn(10000, true).await;

    tokio::spawn(async move {
        let addr = "127.0.0.1:7070";
        let tcp_listener = TcpListener::bind(addr).await.unwrap();

        println!("Listening on: {}", addr);

        loop {
            let (tcp_stream, _) = tcp_listener.accept().await.unwrap();

            my_actor.send(Req(tcp_stream)).await.unwrap();
        }
    })
    .await
    .unwrap();
}

#[derive(Default)]
struct MyActor;

#[async_trait::async_trait]
impl Actor for MyActor {
    const MAIL_BOX_SIZE: u32 = 10;

    async fn create(_ctx: &mut Context<Self>) -> Self
    where
        Self: Sized,
    {
        MyActor
    }
}

#[derive(Debug, Message)]
struct Req(TcpStream);

#[async_trait::async_trait]
impl MessageHandler<Req> for MyActor {
    type Output = ();

    #[inline]
    async fn handle(&mut self, Req(stream): Req, _ctx: &mut Context<Self>) -> Self::Output {
        if let Err(http_err) = Http::new()
            .serve_connection(stream, service_fn(hello))
            .await
        {
            eprintln!("Error while serving HTTP connection: {}", http_err);
        }
    }
}

#[inline]
async fn hello(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World!")))
}
