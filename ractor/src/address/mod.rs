pub use local::LocalAddress;
pub use remote::RemoteAddress;

use crate::envelope::MailBoxTx;
use crate::Actor;

mod local;
mod remote;

pub enum Address<A: ?Sized> {
    Local(LocalAddress<A>),
    Remote(RemoteAddress),
}

impl<A> Address<A>
where
    A: Actor,
{
    #[inline]
    pub fn local(sender: MailBoxTx<A>) -> Self {
        Address::Local(LocalAddress::new(sender))
    }

    #[inline]
    pub fn is_local(&self) -> bool {
        matches!(self, Address::Local(_))
    }

    #[inline]
    pub fn is_remote(&self) -> bool {
        matches!(self, Address::Remote(_))
    }
}
