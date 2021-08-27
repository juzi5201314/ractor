use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::{Arc, Weak};

#[cfg(feature = "remote")]
use futures::{Future, FutureExt};

#[cfg(feature = "remote")]
use ractor_rpc::{deserialize, serialize, RemoteType};

use crate::actor_runner::ActorRunner;
use crate::broker::SpawnHandle;
use crate::envelope::MailBoxRx;
use crate::{Actor, LocalAddress};

/// 单个actor的上下文
pub struct Context<A: ?Sized> {
    pub(crate) global_context: GlobalContext<A>,
}

impl<A> Context<A>
where
    A: Actor,
{
    #[inline]
    pub fn global(&self) -> &GlobalContext<A> {
        &self.global_context
    }
}

impl<A> Deref for Context<A>
where
    A: Actor,
{
    type Target = GlobalContext<A>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.global_context
    }
}

/// 全局(同一个地址的全部actor)共享的上下文
pub struct GlobalContext<A: ?Sized> {
    pub(crate) inner: Arc<Inner<A>>,
}

pub struct Inner<A: ?Sized> {
    pub self_addr: Weak<LocalAddress<A>>,
    pub(crate) recipient: MailBoxRx<A>,
}

impl<A> Inner<A>
where
    A: Actor,
{
    pub fn addr_holders_count(&self) -> usize {
        self.self_addr.strong_count()
    }

    pub fn pending_message_count(&self) -> usize {
        self.recipient.len()
    }
}

impl<A> GlobalContext<A>
where
    A: Actor,
{
    pub fn alive_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// 在上下文中产生一个新的Actor
    /// 可以`Broker::bind`将`JoinHandle`绑定到一个Broker上, 以便统一管理.
    pub async fn spawn(&self) -> SpawnHandle<A> {
        let context = Context {
            global_context: self.clone(),
        };
        let actor = A::create(&context).await;
        tokio::spawn(ActorRunner { actor, context }.run()).into()
    }
}

impl<A> Debug for GlobalContext<A>
where
    A: Actor,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Actor GlobalContext")?;
        writeln!(
            f,
            "There are {} actors alive in the context",
            self.alive_count()
        )?;
        writeln!(
            f,
            "There are {} messages pending in the mailbox",
            self.pending_message_count()
        )?;
        writeln!(
            f,
            "There are {} holders with email addresses",
            self.addr_holders_count()
        )
    }
}

impl<A> Deref for GlobalContext<A> {
    type Target = Inner<A>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<A> Clone for GlobalContext<A> {
    fn clone(&self) -> Self {
        GlobalContext {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(feature = "remote")]
#[derive(Default)]
pub struct MessageRegister(
    pub  std::collections::HashMap<
        u64,
        Box<
            dyn Fn(
                    &[u8],
                ) -> std::pin::Pin<
                    Box<dyn Future<Output = Result<Vec<u8>, ractor_rpc::Error>> + Send>,
                > + Send
                + Sync,
        >,
    >,
);

#[cfg(feature = "remote")]
impl MessageRegister {
    pub fn register<M, A>(&mut self, addr: LocalAddress<A>)
    where
        M: ?Sized + RemoteType + crate::message::Message + 'static,
        A: crate::MessageHandler<M>,
        A::Output: RemoteType,
    {
        self.0.insert(
            M::identity_id(),
            Box::new(move |bytes| {
                let addr = addr.clone();
                let msg = deserialize::<M>(&bytes);
                async move {
                    Ok(serialize(
                        &addr
                            .send(msg?)
                            .await
                            .map_err(|_| ractor_rpc::Error::ForwardToLocal)?
                            .recv()
                            .await
                            .map_err(|_| ractor_rpc::Error::HandlerPanic)?,
                    )?)
                }
                .boxed()
            }),
        );
    }
}
