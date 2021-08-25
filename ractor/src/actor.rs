use crate::address::LocalAddress;
use crate::context::Context;
use crate::MessageRegister;
use std::any::Any;

#[async_trait::async_trait]
pub trait Actor: Send + 'static {
    const MAIL_BOX_SIZE: u32;

    async fn create(_ctx: &Context<Self>) -> Self
    where
        Self: Sized;

    async fn started(&mut self, _ctx: &Context<Self>) {}

    async fn stopped(&mut self, _ctx: &Context<Self>) {}

    fn register(register: &mut MessageRegister, local_address: LocalAddress<Self>) {}

    /// 捕获panic
    /// 用于发生意外的时候处理actor
    /// 不建议也不应该用于错误处理
    ///
    /// 目前只会在handle message时捕获
    fn catch_unwind(&mut self, _err: Box<dyn Any + Send>) {}

    #[inline]
    fn msg_register(local_address: LocalAddress<Self>) -> MessageRegister {
        let mut register = MessageRegister::default();
        Self::register(&mut register, local_address);
        register
    }
}
