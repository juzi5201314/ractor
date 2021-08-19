use crate::envelope::{Envelope, MailBoxTx, RespRx};
use crate::error::{ChannelSendError, ChannelTrySendError};
use crate::message::Message;
use crate::{Actor, MessageHandler, ResponseHandle};

pub struct Address<A> {
    pub(crate) sender: MailBoxTx<A>,
}

impl<A> Address<A>
where
    A: Actor,
{
    #[inline]
    pub fn new(sender: MailBoxTx<A>) -> Self {
        Address { sender }
    }

    #[inline]
    pub async fn stop(&self) -> bool {
        self.sender.send(Envelope::Stop).await.is_ok()
    }

    #[inline]
    pub fn try_stop(&self) -> Result<(), ChannelTrySendError<Envelope<A>>> {
        self.sender.try_send(Envelope::Stop)?;
        Ok(())
    }

    #[inline]
    pub async fn send_envelope<O>(
        &self,
        (envelope, rx): (Envelope<A>, RespRx<O>),
    ) -> Result<ResponseHandle<O>, ChannelSendError<Envelope<A>>> {
        self.sender
            .send(envelope)
            .await
            .map_err::<ChannelSendError<Envelope<A>>, _>(Into::into)?;
        Ok(ResponseHandle(rx))
    }

    #[inline]
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

    // blocking_send有点问题, 还需调试

    #[inline]
    pub fn try_send_envelope<O>(
        &self,
        (envelope, rx): (Envelope<A>, RespRx<O>),
    ) -> Result<ResponseHandle<O>, ChannelTrySendError<Envelope<A>>> {
        self.sender
            .try_send(envelope)
            .map_err::<ChannelTrySendError<Envelope<A>>, _>(Into::into)?;
        Ok(ResponseHandle(rx))
    }

    #[inline]
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
}

impl<A> Clone for Address<A> {
    fn clone(&self) -> Self {
        Address {
            sender: self.sender.clone(),
        }
    }
}
