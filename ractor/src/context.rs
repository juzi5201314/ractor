use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::{Arc, Weak};

use crate::envelope::MailBoxRx;
use crate::{Actor, Address, Stage};

pub struct Context<A: ?Sized> {
    pub(crate) inner: Arc<Inner<A>>,
}

pub struct Inner<A: ?Sized> {
    pub self_addr: Weak<Address<A>>,
    pub(crate) recipient: MailBoxRx<A>,
    pub stage: Stage,
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

impl<A> Context<A>
where
    A: Actor,
{
    pub fn alive_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl<A> Debug for Context<A>
where
    A: Actor,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Actor Context")?;
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

impl<A> Deref for Context<A> {
    type Target = Inner<A>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<A> Clone for Context<A> {
    fn clone(&self) -> Self {
        Context {
            inner: Arc::clone(&self.inner)
        }
    }
}
