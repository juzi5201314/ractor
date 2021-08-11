use std::ops::Deref;

use crate::envelope::MailBoxTx;
use crate::{Scenes, ExecutorHandle};

#[async_trait::async_trait]
pub trait Actor: Send + Unpin + 'static {
    const MAIL_BOX_SIZE: u32;

    async fn started<H>(&mut self, _scenes: &Scenes<H>) where H: ExecutorHandle {}

    async fn stopped<H>(&mut self, _scenes: &Scenes<H>) where H: ExecutorHandle {}

    fn error_handle() {}
}

pub trait ActorCreator {
    fn create() -> Self
        where
            Self: Sized;
}

impl<T> ActorCreator for T where T: Default {
    #[inline]
    fn create() -> Self where Self: Sized {
        Default::default()
    }
}

#[derive(Clone)]
pub struct Address<A> {
    sender: MailBoxTx<A>,
}

impl<'a, A> Deref for Address<A> {
    type Target = MailBoxTx<A>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

impl<A> Address<A> {
    #[inline]
    pub fn new(sender: MailBoxTx<A>) -> Self {
        Address { sender }
    }
}
