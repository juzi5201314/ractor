use crate::actor::Actor;
use crate::message::{Message, MessageHandler};
use std::ops::Deref;
use std::sync::Arc;

pub struct Broker<A> {
    pub(crate) actor: A
}

impl<A> Broker<A> where A: Actor {
    pub fn new(actor: A) -> Self {
        Broker {
            actor
        }
    }
}
