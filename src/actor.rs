use std::marker::PhantomData;
use std::ops::Deref;

use crossfire::mpsc::{RxFuture, SharedFutureBoth, TxFuture};
use futures::channel::oneshot;
use futures::future::BoxFuture;
use futures::FutureExt;

use crate::broker::Broker;
use crate::executor::{Executor, ExecutorHandle, Show};
use crate::message::{Message, MessageHandler, MessageResponse, Response, ResponseHandle};
use crate::stage::{Scenes, Stage};

pub type Envelope<A> = Box<dyn FnOnce(&mut A) -> () + Send>;

fn pack<A, M, H>(
    scenes: Scenes<H>,
    msg: M,
) -> (
    Envelope<A>,
    oneshot::Receiver<Response<<<A as MessageHandler<M>>::Output as MessageResponse>::Output>>,
)
where
    M: Message + 'static,
    A: MessageHandler<M>,
    H: ExecutorHandle,
{
    let (tx, rx) = oneshot::channel();
    (
        (Box::new(move |actor: &mut A| {
            let resp = <A as MessageHandler<M>>::handle(actor, msg).into_response(scenes);
            tx.send(resp).map_err(|_| (/* Response is discarded */));
        })),
        rx,
    )
}

pub type Tx<A> = TxFuture<Envelope<A>, SharedFutureBoth>;
pub type Rx<A> = RxFuture<Envelope<A>, SharedFutureBoth>;

pub trait Actor: Send + Unpin + 'static {
    const MAIL_BOX_SIZE: u32 = 100;

    fn create() -> Self
    where
        Self: Sized;
}

#[derive(Clone)]
pub struct Address<A> {
    sender: Tx<A>,
}

impl<'a, A> Deref for Address<A> {
    type Target = Tx<A>;

    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

impl<A> Address<A> {
    pub fn new(sender: Tx<A>) -> Self {
        Address { sender }
    }
}

pub struct ActorHandler<A, H> {
    scenes: Scenes<H>,
    addr: Address<A>,
}

impl<A, H> ActorHandler<A, H>
where
    A: Actor,
    H: ExecutorHandle,
{
    pub async fn create(scenes: &Scenes<H>) -> ActorHandler<A, H> {
        let (tx, rx) = crossfire::mpsc::bounded_future_both(A::MAIL_BOX_SIZE as usize);
        let addr = Address::new(tx);
        let broker = Broker::new(A::create());
        scenes.show(Show::new(rx, broker, scenes.clone()));
        ActorHandler {
            scenes: scenes.clone(),
            addr,
        }
    }

    pub(crate) fn scenes(&self) -> &Scenes<H> {
        &self.scenes
    }

    pub async fn send<M>(
        &self,
        msg: M,
    ) -> Result<ResponseHandle<<<A as MessageHandler<M>>::Output as MessageResponse>::Output>, ()>
    where
        M: Message + 'static,
        A: MessageHandler<M>,
    {
        let (envelope, rx) = pack(self.scenes.clone(), msg);
        self.addr.send(envelope).await.map_err(|err| ())?;
        Ok(ResponseHandle(rx))
    }

    pub fn try_send<M>(&self, msg: M)
    where
        M: Message + 'static,
        A: MessageHandler<M>,
    {
        let (envelope, rx) = pack(self.scenes.clone(), msg);
        self.addr.try_send(envelope).unwrap();
    }
}
