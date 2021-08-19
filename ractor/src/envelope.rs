use crossfire::mpmc::{RxFuture, SharedFutureBoth, TxFuture};
use crossfire::mpsc;
use crossfire::mpsc::SharedSenderBRecvF;
use futures::future::BoxFuture;

use crate::actor::Actor;
use crate::message::{Message, MessageHandler};

pub enum Envelope<A :?Sized> {
    Task(Box<dyn FnOnce(&mut A) -> BoxFuture<()> + Send>),
    Stop,
}

impl<A> Envelope<A>
where
    A: Actor,
{
    pub(crate) fn pack<M>(
        msg: M,
    ) -> (Envelope<A>, RespRx<<A as MessageHandler<M>>::Output>)
    where
        M: Message + 'static,
        A: MessageHandler<M>,
    {
        let (tx, rx) = mpsc::bounded_tx_blocking_rx_future(1);
        (
            Envelope::Task(Box::new(move |actor: &mut A| {
                Box::pin(async move {
                    let resp = <A as MessageHandler<M>>::handle(actor, msg).await;
                    tx.try_send(resp)
                        .map_err(|_| (/* Response is discarded */))
                        .ok();
                })
            })),
            rx,
        )
    }
}

pub type MailBoxTx<A> = TxFuture<Envelope<A>, SharedFutureBoth>;
pub type MailBoxRx<A> = RxFuture<Envelope<A>, SharedFutureBoth>;
#[allow(unused)]
pub type RespTx<O> = mpsc::TxBlocking<O, SharedSenderBRecvF>;
pub type RespRx<O> = mpsc::RxFuture<O, SharedSenderBRecvF>;
