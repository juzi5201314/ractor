use ractor::{Actor, Context, Message, MessageHandler, RemoteAddress, Broker};
use ractor::{LocalAddress, MessageRegister};
use ractor_rpc::RemoteType;
use url::Url;

#[derive(Debug, Message, serde::Deserialize, serde::Serialize)]
struct Sum(isize, isize);

impl RemoteType for Sum {
    type Identity = &'static str;

    fn identity() -> Self::Identity {
        "example::remote::sum"
    }
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

    fn register(register: &mut MessageRegister, local_address: LocalAddress<Self>) {
        register.register::<Sum, Self>(local_address);
    }
}

#[async_trait::async_trait]
impl MessageHandler<Sum> for MyActor {
    type Output = isize;

    async fn handle(
        &mut self,
        Sum(a, b): Sum,
        _ctx: &mut Context<Self>,
    ) -> Self::Output {
        a + b
    }
}

#[tokio::main]
async fn main() {
    let my_actor = Broker::<MyActor>::spawn(100).await;
    let local_addr = my_actor.addr();
    let server_guard = local_addr.clone().upgrade().await.unwrap();

    let url = Url::parse(&format!(
        "ws://127.0.0.1:{}",
        server_guard.local_addr().port()
    ))
    .unwrap();

    // 以下代码只知道服务器地址
    // --------------------------------

    let mut remote_addr = RemoteAddress::connect(url).await.unwrap();

    let resp = remote_addr.send::<_, MyActor>(Sum(1, 2)).await.unwrap();
    assert_eq!(resp.recv().await.unwrap(), 3);
}
