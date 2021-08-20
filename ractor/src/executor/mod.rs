use std::panic::AssertUnwindSafe;

use futures::FutureExt;

pub use self::tokio::Executor;

use crate::actor::Actor;
use crate::context::Context;
use crate::envelope::Envelope;

mod tokio;

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
                    match AssertUnwindSafe((handle)(&mut self.actor, &self.context))
                        .catch_unwind()
                        .await
                    {
                        Ok(_) => {}
                        Err(err) => {
                            self.actor.catch_unwind(err);
                        }
                    }
                }
                Envelope::Stop => break,
            }
        }
        self.actor.stopped(&self.context).await;
    }
}
