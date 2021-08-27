#[cfg(feature = "remote")]
use crate::address::remote::RemoteAddressServer;
use crate::envelope::{self, Envelope, MailBoxTx};
use crate::error::{ChannelSendError, ChannelTrySendError};
use crate::message::Message;
use crate::{Actor, MessageHandler, ResponseHandle};

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
    #[cfg(feature = "remote")]
    #[inline]
    pub async fn upgrade(self) -> Result<RemoteAddressServer, ractor_rpc::Error> {
        RemoteAddressServer::from_local(self, "0.0.0.0:0".parse().unwrap()).await
    }

    #[cfg(feature = "remote")]
    #[inline]
    pub async fn upgrade_to(
        self,
        addr: std::net::SocketAddr,
    ) -> Result<RemoteAddressServer, ractor_rpc::Error> {
        RemoteAddressServer::from_local(self, addr).await
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
        let (envelope, rx) = envelope::pack(msg);
        self.sender
            .send(envelope)
            .await
            .map_err::<ChannelSendError<Envelope<A>>, _>(Into::into)?;
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
        let (envelope, rx) = envelope::pack(msg);
        self.sender
            .try_send(envelope)
            .map_err::<ChannelTrySendError<Envelope<A>>, _>(Into::into)?;
        Ok(ResponseHandle(rx))
    }
}

impl<A> Clone for LocalAddress<A> {
    fn clone(&self) -> Self {
        LocalAddress {
            sender: self.sender.clone(),
        }
    }
}
