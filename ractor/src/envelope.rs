use crossfire::mpmc::{RxFuture, SharedFutureBoth, TxFuture};
use crossfire::mpsc;
use crossfire::mpsc::SharedSenderBRecvF;
use futures::future::BoxFuture;

use crate::actor::Actor;
use crate::Context;
use crate::message::{Message, MessageHandler};

pub enum Envelope<A: ?Sized> {
    Task(Box<dyn for<'a> FnOnce(&'a mut A, &'a Context<A>) -> BoxFuture<'a, ()> + Send>),
    Stop,
}

impl<A> Envelope<A>
where
    A: Actor,
{
    pub(crate) fn pack<M>(msg: M) -> (Envelope<A>, RespRx<<A as MessageHandler<M>>::Output>)
    where
        M: Message + 'static,
        A: MessageHandler<M>,
    {
        let (tx, rx) = mpsc::bounded_tx_blocking_rx_future(1);
        (
            Envelope::Task(Box::new(move |actor: &mut A, ctx: &Context<A>| {
                Box::pin(async move {
                    let resp_res = <A as MessageHandler<M>>::handle(actor, msg, ctx).await;
                    match resp_res {
                        Err(err) => {
                            <A as MessageHandler<M>>::handle_error(actor, err, ctx).await;
                        }
                        Ok(resp) => {
                            tx.try_send(resp)
                                .map_err(|_| (/* Response is discarded */))
                                .ok();
                        }
                    }
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
