use std::sync::Arc;
use std::sync::atomic::Ordering;

pub use executor::{Executor, JoinHandle};

use crate::actor::Actor;
use crate::context::Context;
use crate::envelope::{Envelope, MailBoxRx};

#[cfg(all(feature = "use_tokio", feature = "use_async-std"))]
compile_error!("Only one of the asynchronous executors can be selected.");

#[cfg_attr(feature = "use_tokio", path = "tokio.rs")]
#[cfg_attr(feature = "use_async-std", path = "async_std.rs")]
mod executor;

pub struct ActorRunner<A> {
    pub rx: MailBoxRx<A>,
    pub actor: A,
    pub context: Arc<Context<A>>,
}

impl<A> ActorRunner<A>
where
    A: Actor,
{
    pub async fn run(mut self) -> () {
        self.actor.started().await;
        self.context.alive_count.fetch_add(1, Ordering::SeqCst);

        while let Ok(envelope) = self.rx.recv().await {
            match envelope {
                Envelope::Task(handle) => {
                    (handle)(&mut self.actor).await;
                }
                Envelope::Stop => break,
            }
        }
        self.context.alive_count.fetch_sub(1, Ordering::SeqCst);
        self.actor.stopped().await;
    }
}
