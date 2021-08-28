use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::{Arc, Weak};
use std::time::Duration;

#[cfg(feature = "remote")]
use futures::{Future, FutureExt};
use tokio::sync::Notify;

#[cfg(feature = "remote")]
use ractor_rpc::{deserialize, serialize, RemoteType};

use crate::actor_runner::ActorRunner;
use crate::broker::SpawnHandle;
use crate::envelope::MailBoxRx;
use crate::{Actor, LocalAddress};

/// 指示Actor之后的状态
///
///
/// 状态跟Actor的生命周期息息相关
///
/// 指定本周期结束的状态, 在周期结束之前执行某种行为
///
///
/// 在不合适的地方设置不适合的状态会被忽略,
///
/// 每一次周期结束之后, 状态会重置为[`Status::Continue`].
#[derive(Clone)]
pub enum State {
    /// Actor创建之后的默认状态, 什么都不做.
    Continue,
    /// 停止Actor
    ///
    ///
    /// 在生命周期中的[`Stop`]状态才会触发[`Actor::stopped`]. 因为还没有开始, 自然就没有停止.
    ///
    /// 所以如果在[`Actor::create`]中设置状态为[`Status::Stop`], 那么Actor什么都不会做就停止了.
    Stop,
    /// 收到`Notify`的通知后恢复
    Pause(Arc<Notify>),
    /// 调用[`tokio::task::yield_now`]
    Yield,
    /// 重置Actor状态并重启, 重新开始Actor生命周期
    ///
    /// 仅在生命周期内有效.
    ///
    /// 例如, 你无法在start之前就reset, 这样会无视掉[`Status::Reset`]并在[`Actor::started`]之后执行reset操作.
    ///
    ///
    /// 会调用[`Actor::reset`]来重置状态
    Reset,
    /// 表示终止
    ///
    ///
    /// 该状态只会出现在[`Actor::catch_unwind`]中
    ///
    /// 手动设置此状态无效, 等同于[`Status::Continue`].
    Abort,
    /// 休眠指定时间,
    /// .ms
    Sleep(u64),
}

impl State {
    #[inline]
    pub(crate) fn clear(&mut self) {
        *self = State::Continue
    }
}

/// 单个actor的上下文
pub struct Context<A: ?Sized> {
    pub(crate) global_context: GlobalContext<A>,
    /// 指定本周期结束的状态
    pub state: State,
}

impl<A> Context<A>
where
    A: Actor,
{
    #[inline]
    pub fn global(&self) -> &GlobalContext<A> {
        &self.global_context
    }

    #[inline]
    pub fn stop(&mut self) {
        self.state = State::Stop;
    }

    #[inline]
    pub fn pause(&mut self) -> Arc<Notify> {
        let notify = Arc::new(Notify::const_new());
        self.state = State::Pause(notify.clone());
        notify
    }

    #[inline]
    pub fn r#yield(&mut self) {
        self.state = State::Yield;
    }

    #[inline]
    pub fn reset(&mut self) {
        self.state = State::Reset;
    }

    #[inline]
    pub fn sleep(&mut self, dur: Duration) {
        self.state = State::Sleep(dur.as_millis() as u64);
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
    ///
    /// 可以[`Broker::bind`]将[`SpawnHandle`]绑定到一个Broker上, 以便统一管理.
    pub async fn spawn(&self) -> SpawnHandle<A> {
        let mut context = Context {
            global_context: self.clone(),
            state: State::Continue,
        };
        let actor = A::create(&mut context).await;
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
