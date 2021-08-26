pub use local::LocalAddress;
#[cfg(feature = "remote")]
pub use remote::RemoteAddress;

use crate::envelope::MailBoxTx;
use crate::Actor;

mod local;
#[cfg(feature = "remote")]
mod remote;

pub enum Address<A: ?Sized> {
    Local(LocalAddress<A>),
    #[cfg(feature = "remote")]
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

    #[cfg(feature = "remote")]
    #[inline]
    pub fn is_remote(&self) -> bool {
        matches!(self, Address::Remote(_))
    }
}
