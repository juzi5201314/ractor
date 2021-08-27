use std::panic::AssertUnwindSafe;

use futures::FutureExt;

use crate::actor::Actor;
use crate::context::Context;

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
                (envelope)(&mut this.actor, &this.context).await;
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
    }
}
