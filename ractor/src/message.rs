use std::fmt::{Debug, Formatter};

use async_trait::async_trait;

use crate::actor::Actor;
use crate::envelope::RespRx;
use crate::Context;

pub trait Message: Debug + Send {}

#[async_trait]
pub trait MessageHandler<M>: Sized + Send
where
    Self: Actor,
    M: Message,
{
    type Output: Send + 'static;

    async fn handle(&mut self, msg: M, ctx: &Context<Self>) -> Self::Output;
}

pub struct ResponseHandle<O>(pub(crate) RespRx<O>);

impl<O> ResponseHandle<O> {
    #[inline]
    pub async fn recv(self) -> Result<O, HandlerPanic> {
        self.0.recv().await.map_err(|_| HandlerPanic)
    }

    #[inline]
    pub fn try_recv(&self) -> Result<O, HandlerPanic> {
        self.0.try_recv().map_err(|_| HandlerPanic)
    }
}

pub struct HandlerPanic;

impl Debug for HandlerPanic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "HandlerPanic")
    }
}
