use crossfire::mpmc::bounded_future_both;
use futures::channel::oneshot;

use crate::actor::{Actor, Address};
use crate::envelope::Envelope;
use crate::error::{ChannelSendError, ChannelTrySendError};
use crate::executor::{ExecutorHandle, Show};
use crate::message::{AsyncMessageHandler, Message, MessageHandler, ResponseHandle};
use crate::stage::Scenes;

pub struct Broker<A, H> {
    #[allow(unused)]
    scenes: Scenes<H>,
    addr: Address<A>,
}

impl<A, H> Broker<A, H>
where
    A: Actor,
    H: ExecutorHandle,
{
    pub fn spawn<F>(scenes: &Scenes<H>, actor_creator: F, quantity: usize) -> Broker<A, H>
    where
        F: Fn() -> A,
    {
        let (tx, rx) = bounded_future_both(A::MAIL_BOX_SIZE as usize);
        let addr = Address::new(tx);
        for _ in 0..quantity {
            scenes.show(Show::new(rx.clone(), actor_creator(), scenes.clone()));
        }

        Broker {
            scenes: scenes.clone(),
            addr,
        }
    }

    #[inline]
    pub async fn stop(&self) -> bool {
        self.addr.send(Envelope::Stop).await.is_ok()
    }

    #[inline]
    pub fn blocking_stop(&self) -> bool {
        self.addr.send_blocking(Envelope::Stop).is_ok()
    }

    #[inline]
    pub fn try_stop(&self) -> Result<(), ChannelTrySendError<Envelope<A>>> {
        self.addr.try_send(Envelope::Stop).map_err(Into::<ChannelTrySendError<_>>::into)
    }

    #[inline]
    pub async fn send_envelope<O>(
        &self,
        (envelope, rx): (Envelope<A>, oneshot::Receiver<O>),
    ) -> Result<ResponseHandle<O>, ChannelSendError<Envelope<A>>> {
        self.addr
            .send(envelope)
            .await
            .map_err::<ChannelSendError<Envelope<A>>, _>(Into::into)?;
        Ok(ResponseHandle(rx))
    }

    pub async fn send<M>(
        &self,
        msg: M,
    ) -> Result<ResponseHandle<<A as MessageHandler<M>>::Output>, ChannelSendError<Envelope<A>>>
    where
        M: Message + 'static,
        A: MessageHandler<M>,
    {
        self.send_envelope(Envelope::pack(msg)).await
    }

    pub async fn send_async<M>(
        &self,
        msg: M,
    ) -> Result<ResponseHandle<<A as AsyncMessageHandler<M>>::Output>, ChannelSendError<Envelope<A>>>
    where
        M: Message + 'static,
        A: AsyncMessageHandler<M>,
    {
        self.send_envelope(Envelope::pack_async(msg)).await
    }

    #[inline]
    pub fn blocking_send_envelope<O>(
        &self,
        (envelope, rx): (Envelope<A>, oneshot::Receiver<O>),
    ) -> Result<ResponseHandle<O>, ChannelSendError<Envelope<A>>> {
        self.addr
            .send_blocking(envelope)
            .map_err::<ChannelSendError<Envelope<A>>, _>(Into::into)?;
        Ok(ResponseHandle(rx))
    }

    pub fn blocking_send<M>(
        &self,
        msg: M,
    ) -> Result<ResponseHandle<<A as MessageHandler<M>>::Output>, ChannelSendError<Envelope<A>>>
    where
        M: Message + 'static,
        A: MessageHandler<M>,
    {
        self.blocking_send_envelope(Envelope::pack(msg))
    }

    pub fn blocking_send_async<M>(
        &self,
        msg: M,
    ) -> Result<ResponseHandle<<A as AsyncMessageHandler<M>>::Output>, ChannelSendError<Envelope<A>>>
    where
        M: Message + 'static,
        A: AsyncMessageHandler<M>,
    {
        self.blocking_send_envelope(Envelope::pack_async(msg))
    }

    #[inline]
    pub fn try_send_envelope<O>(
        &self,
        (envelope, rx): (Envelope<A>, oneshot::Receiver<O>),
    ) -> Result<ResponseHandle<O>, ChannelTrySendError<Envelope<A>>> {
        self.addr
            .try_send(envelope)
            .map_err::<ChannelTrySendError<Envelope<A>>, _>(Into::into)?;
        Ok(ResponseHandle(rx))
    }

    pub fn try_send<M>(
        &self,
        msg: M,
    ) -> Result<ResponseHandle<<A as MessageHandler<M>>::Output>, ChannelTrySendError<Envelope<A>>>
    where
        M: Message + 'static,
        A: MessageHandler<M>,
    {
        self.try_send_envelope(Envelope::pack(msg))
    }

    pub fn try_send_async<M>(
        &self,
        msg: M,
    ) -> Result<
        ResponseHandle<<A as AsyncMessageHandler<M>>::Output>,
        ChannelTrySendError<Envelope<A>>,
    >
    where
        M: Message + 'static,
        A: AsyncMessageHandler<M>,
    {
        self.try_send_envelope(Envelope::pack_async(msg))
    }
}

unsafe impl<A, H> Send for Broker<A, H> {}
unsafe impl<A, H> Sync for Broker<A, H> {}
