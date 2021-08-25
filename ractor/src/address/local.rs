use crate::address::remote::RemoteAddressServer;
use crate::envelope::{Envelope, MailBoxTx, RespRx};
use crate::error::{ChannelSendError, ChannelTrySendError};
use crate::message::Message;
use crate::{Actor, MessageHandler, ResponseHandle};
use std::net::SocketAddr;

pub struct LocalAddress<A: ?Sized> {
    pub(crate) sender: MailBoxTx<A>,
}

impl<A> LocalAddress<A>
where
    A: Actor,
{
    #[inline]
    pub fn new(sender: MailBoxTx<A>) -> Self {
        LocalAddress { sender }
    }

    /// 升级到远程地址
    /// 默认监听`0.0.0.0:0`
    #[inline]
    pub async fn upgrade(self) -> Result<RemoteAddressServer, ractor_rpc::Error> {
        RemoteAddressServer::from_local(self, "0.0.0.0:0".parse().unwrap()).await
    }

    #[inline]
    pub async fn upgrade_to(
        self,
        addr: SocketAddr,
    ) -> Result<RemoteAddressServer, ractor_rpc::Error> {
        RemoteAddressServer::from_local(self, addr).await
    }

    // 不能广播消息, 这个功能有待商榷
    /*#[inline]
    pub async fn stop(&self) -> bool {
        self.sender.send(Envelope::Stop).await.is_ok()
    }

    #[inline]
    pub fn try_stop(&self) -> Result<(), ChannelTrySendError<Envelope<A>>> {
        self.sender.try_send(Envelope::Stop)?;
        Ok(())
    }*/

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

impl<A> Clone for LocalAddress<A> {
    fn clone(&self) -> Self {
        LocalAddress {
            sender: self.sender.clone(),
        }
    }
}
