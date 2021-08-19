pub use executor::{Executor, JoinHandle};

use crate::actor::Actor;
use crate::context::Context;
use crate::envelope::Envelope;

#[cfg(all(feature = "use_tokio", feature = "use_async-std"))]
compile_error!("Only one of the asynchronous executors can be selected.");

#[cfg_attr(feature = "use_tokio", path = "tokio.rs")]
#[cfg_attr(feature = "use_async-std", path = "async_std.rs")]
mod executor;

pub struct ActorRunner<A> {
    pub actor: A,
    pub context: Context<A>,
}

impl<A> ActorRunner<A>
where
    A: Actor,
{
    pub async fn run(mut self) -> () {
        self.actor.started(&self.context).await;

        while let Ok(envelope) = self.context.recipient.recv().await {
            match envelope {
                Envelope::Task(handle) => {
                    (handle)(&mut self.actor, &self.context).await;
                }
                Envelope::Stop => break,
            }
        }
        self.actor.stopped(&self.context).await;
    }
}
