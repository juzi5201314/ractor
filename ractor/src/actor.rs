use std::any::Any;

use crate::context::Context;

#[async_trait::async_trait]
pub trait Actor: Send + 'static {
    /// 信箱大小
    const MAIL_BOX_SIZE: u32;

    /// 最大重试次数
    const MAX_RESTARTS: u16 = 5;

    async fn create(_ctx: &Context<Self>) -> Self
    where
        Self: Sized;

    async fn started(&mut self, _ctx: &Context<Self>) {}

    async fn stopped(&mut self, _ctx: &Context<Self>) {}

    #[cfg(feature = "remote")]
    fn register(_register: &mut crate::MessageRegister, _local_address: crate::LocalAddress<Self>) {}

    /// 用于发生意外的时候处理actor
    /// 不建议也不应该用于错误处理
    ///
    /// # Return
    /// 如果返回值为`true`, actor会重新开始运行`start -> handle message -> stopped`的流程,
    /// 这个时候使用者应当清理并重置actor的状态.
    /// 最大重启次数视乎`Actor::MAX_RESTARTS`, 到达限制之后无论如何都会结束, 避免无限重启.
    ///
    /// 如果返回值为`false`, actor会正常结束.
    fn catch_unwind(&mut self, _err: Box<dyn Any + Send>) -> bool {
        false
    }

    #[cfg(feature = "remote")]
    #[inline]
    fn msg_register(local_address: crate::address::LocalAddress<Self>) -> crate::MessageRegister {
        let mut register = crate::MessageRegister::default();
        Self::register(&mut register, local_address);
        register
    }
}
