#![allow(rustdoc::broken_intra_doc_links)]

pub use actor::Actor;
pub use actor_runner::StoppingPosition;
#[cfg(feature = "remote")]
pub use address::RemoteAddress;
pub use address::{Address, LocalAddress};
pub use broker::{Broker, SpawnHandle};
#[cfg(feature = "remote")]
pub use context::MessageRegister;
pub use context::{Context, GlobalContext, State};
pub use message::{Message, MessageHandler, ResponseHandle};
/*#[cfg(feature = "derive")]
pub use ractor_derive::*;
*/
mod actor;
mod actor_runner;
mod address;
mod broker;
mod context;
mod envelope;
pub mod error;
mod message;

#[cfg(test)]
mod tests {}
