use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Weak;

use crate::{Actor, Address, Stage};
use crate::envelope::MailBoxRx;

pub struct Context<A: ?Sized> {
    pub self_addr: Weak<Address<A>>,
    pub recipient: MailBoxRx<A>,
    pub stage: Stage,
    pub alive_count: AtomicUsize
}

impl<A> Context<A> where A: Actor {
    pub fn pending_message_count(&self) -> Option<usize> {
        self.self_addr.upgrade().map(|addr| addr.sender.len())
    }

    pub fn addr_holders_count(&self) -> usize {
        self.self_addr.strong_count()
    }

    pub fn alive_count(&self) -> usize {
        self.alive_count.load(Ordering::SeqCst)
    }
}
