use crossfire::mpmc::{RxFuture, SharedFutureBoth, TxFuture};
use futures::channel::oneshot;
use futures::future::BoxFuture;

use crate::actor::Actor;
use crate::message::{AsyncMessageHandler, Message, MessageHandler};

pub enum Envelope<A> {
    Sync(Box<dyn FnOnce(&mut A) -> () + Send>),
    Async(Box<dyn FnOnce(&mut A) -> BoxFuture<()> + Send>),
    Stop,
}

impl<A> Envelope<A>
where
    A: Actor,
{
    pub(crate) fn pack_async<M>(
        msg: M,
    ) -> (
        Envelope<A>,
        oneshot::Receiver<<A as AsyncMessageHandler<M>>::Output>,
    )
    where
        M: Message + 'static,
        A: AsyncMessageHandler<M>,
    {
        let (tx, rx) = oneshot::channel();
        (
            Envelope::Async(Box::new(move |actor: &mut A| {
                Box::pin(async move {
                    let resp = <A as AsyncMessageHandler<M>>::handle(actor, msg).await;
                    tx.send(resp).map_err(|_| (/* Response is discarded */)).ok();
                })
            })),
            rx,
        )
    }

    pub(crate) fn pack<M>(
        msg: M,
    ) -> (
        Envelope<A>,
        oneshot::Receiver<<A as MessageHandler<M>>::Output>,
    )
    where
        M: Message + 'static,
        A: MessageHandler<M>,
    {
        let (tx, rx) = oneshot::channel();
        (
            Envelope::Sync(Box::new(move |actor: &mut A| {
                let resp = <A as MessageHandler<M>>::handle(actor, msg);
                tx.send(resp).map_err(|_| (/* Response is discarded */)).ok();
            })),
            rx,
        )
    }
}

pub type MailBoxTx<A> = TxFuture<Envelope<A>, SharedFutureBoth>;
pub type MailBoxRx<A> = RxFuture<Envelope<A>, SharedFutureBoth>;
