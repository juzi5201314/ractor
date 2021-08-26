use std::panic::AssertUnwindSafe;

use futures::FutureExt;

use crate::actor::Actor;
use crate::context::Context;
use crate::envelope::Envelope;

pub struct ActorRunner<A> {
    pub actor: A,
    pub context: Context<A>,
}

impl<A> ActorRunner<A>
where
    A: Actor,
{
    pub async fn run(mut self) -> () {
        #[inline(always)]
        #[doc(hidden)]
        async fn _run<A>(this: &mut ActorRunner<A>)
        where
            A: Actor,
        {
            this.actor.started(&this.context).await;

            while let Ok(envelope) = this.context.recipient.recv().await {
                match envelope {
                    Envelope::Task(handle) => {
                        (handle)(&mut this.actor, &this.context).await;
                    }
                    Envelope::Stop => break,
                }
            }

            this.actor.stopped(&this.context).await;
        }
        let mut restart_count = 0;
        loop {
            match AssertUnwindSafe(_run(&mut self)).catch_unwind().await {
                Ok(_) => break,
                Err(err) => {
                    if self.actor.catch_unwind(err) && restart_count < A::MAX_RESTARTS {
                        restart_count += 1;
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }

        /*self.actor.started(&self.context).await;

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
        self.actor.stopped(&self.context).await;*/
    }
}
