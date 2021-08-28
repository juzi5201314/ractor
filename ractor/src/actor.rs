use std::any::Any;

use crate::actor_runner::StoppingPosition;
use crate::context::Context;
use crate::State;

#[async_trait::async_trait]
pub trait Actor: Send + 'static {
    /// 信箱大小
    const MAIL_BOX_SIZE: u32;

    /// 最大重试次数
    const MAX_RESTARTS: u16 = 3;

    type Args: Send + Clone;

    async fn create(_ctx: &mut Context<Self>) -> Self
    where
        Self: Sized;

    async fn started(&mut self, _ctx: &mut Context<Self>) {}

    async fn stopped(&mut self, _ctx: &mut Context<Self>, _pos: StoppingPosition) {}

    /// 清理并重置actor的状态
    ///
    ///
    /// 默认行为为调用[`Actor::create`]创建一个新的Actor替换当前Self.
    async fn reset(&mut self, _ctx: &mut Context<Self>)
    where
        Self: Sized,
    {
        let new = Self::create(_ctx).await;
        *self = new;
    }

    #[cfg(feature = "remote")]
    fn register(_register: &mut crate::MessageRegister, _local_address: crate::LocalAddress<Self>) {
    }

    /// 用于发生意外的时候处理actor
    ///
    /// 不建议也绝不应该用于错误处理
    ///
    ///
    /// 捕获在生命周期期间发生的panic ([`Actor::started`], [`Actor::stopped`], [`MessageHandler::handle`])
    ///
    ///
    /// 此时状态为[`State::Abort`], 之后Actor将会结束(没有[`Actor::stopped`]).
    ///
    /// 如状态设置为[`State::Reset`], actor会重新开始Actor的生命周期, 并调用[`Actor::reset`]清理actor状态.
    ///
    /// 忽略其他状态
    ///
    ///
    /// 最大重启次数视乎[`Actor::MAX_RESTARTS`], 到达限制之后无论如何都会结束, 避免意外错误导致的无限重启和重启带来的性能损耗.
    fn catch_unwind(&mut self, _err: Box<dyn Any + Send>, ctx: &mut Context<Self>) {
        ctx.state = State::Reset
    }

    #[cfg(feature = "remote")]
    #[inline]
    fn msg_register(local_address: crate::address::LocalAddress<Self>) -> crate::MessageRegister {
        let mut register = crate::MessageRegister::default();
        Self::register(&mut register, local_address);
        register
    }
}
