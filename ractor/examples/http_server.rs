use std::convert::Infallible;

use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Request, Response};
use mimalloc::MiMalloc;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Handle;

use ractor::{Actor, Context, Message, MessageHandler, Stage};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    let stage = Stage::from_handle(Handle::current());

    let my_actor = stage.spawn::<MyActor>(10000);

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

impl Actor for MyActor {
    const MAIL_BOX_SIZE: u32 = 10;

    fn create(_ctx: &Context<Self>) -> Self
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
    async fn handle(&mut self, Req(stream): Req) -> Self::Output {
        if let Err(http_err) = Http::new()
            .serve_connection(stream, service_fn(hello))
            .await
        {
            eprintln!("Error while serving HTTP connection: {}", http_err);
        }
    }
}

async fn hello(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World!")))
}
