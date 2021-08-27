use crossfire::mpmc::{RxFuture, SharedFutureBoth, TxFuture};
use futures::future::BoxFuture;
use tokio::sync::oneshot;

use crate::message::{Message, MessageHandler};
use crate::{Actor, Context};

pub type Envelope<A> =
    Box<dyn for<'a> FnOnce(&'a mut A, &'a mut Context<A>) -> BoxFuture<'a, ()> + Send>;

pub(crate) fn pack<A, M>(msg: M) -> (Envelope<A>, RespRx<<A as MessageHandler<M>>::Output>)
where
    M: Message + 'static,
    A: Actor + MessageHandler<M>,
{
    let (tx, rx) = oneshot::channel();
    (
        Box::new(move |actor: &mut A, ctx: &mut Context<A>| {
            Box::pin(async move {
                let resp = <A as MessageHandler<M>>::handle(actor, msg, ctx).await;
                tx.send(resp)
                    .map_err(|_| (/* Response is discarded */))
                    .ok();
            })
        }),
        rx,
    )
}

pub type MailBoxTx<A> = TxFuture<Envelope<A>, SharedFutureBoth>;
pub type MailBoxRx<A> = RxFuture<Envelope<A>, SharedFutureBoth>;
pub type RespRx<O> = oneshot::Receiver<O>;
