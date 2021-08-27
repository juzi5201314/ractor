use std::fmt::{Debug, Formatter, Display};

use async_trait::async_trait;

use crate::actor::Actor;
use crate::envelope::RespRx;
use crate::Context;

pub trait Message: Send {}

#[async_trait]
pub trait MessageHandler<M>: Sized + Send
where
    Self: Actor,
    M: Message,
{
    type Output: Send + 'static;
    /// 处理消息
    ///
    /// # Error
    /// `Self::Error`表示在处理消息时发生的错误, 并在返回`Error`时将`Error`发送到`handle_error`处理.
    ///
    /// ### 为什么我们有时候需要`Output = Result<..., Error1>`?
    /// 在返回`Self::Error`的时候会使用`handle_error`处理这个错误.
    /// 而将`Output`设置为`Result`的时候, `Result<..., Error1>`将会原封不动发送到`ResponseHandle`, 使用`recv`接收.
    ///
    /// 也就是说`Self::Error`适用于你发送了消息但不需要接收响应的时候处理错误,
    /// 而`Output = Result<..., Error1>`适用于在等待接收响应之后处理错误.
    async fn handle(&mut self, msg: M, ctx: &mut Context<Self>) -> Self::Output;
}

pub struct ResponseHandle<O>(pub(crate) RespRx<O>);

impl<O> ResponseHandle<O> {
    #[inline]
    pub async fn recv(self) -> Result<O, HandlerPanic> {
        self.0.await.map_err(|_| HandlerPanic)
    }

    #[inline]
    pub fn try_recv(&mut self) -> Result<O, HandlerPanic> {
        self.0.try_recv().map_err(|_| HandlerPanic)
    }
}

pub struct HandlerPanic;

impl Debug for HandlerPanic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "HandlerPanic")
    }
}

impl Display for HandlerPanic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for HandlerPanic {}
