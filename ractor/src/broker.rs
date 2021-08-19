use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::task::Poll;

use crossfire::mpmc::bounded_future_both;
use futures::Stream;
use futures::stream::FuturesUnordered;

use crate::actor::Actor;
use crate::address::Address;
use crate::context::Context;
use crate::executor::ActorRunner;
use crate::JoinHandle;
use crate::stage::Stage;

pub struct Broker<A> {
    addr: Arc<Address<A>>,
    actor_runner_handles: FuturesUnordered<JoinHandle<()>>,
}

impl<A> Broker<A>
where
    A: Actor,
{
    pub fn spawn(stage: &Stage, quantity: usize) -> Broker<A> {
        let (tx, rx) = bounded_future_both(A::MAIL_BOX_SIZE as usize);
        let addr = Arc::new(Address::new(tx));

        let context = Arc::new(Context {
            self_addr: Arc::downgrade(&addr),
            stage: stage.clone(),
            alive_count: AtomicUsize::new(0),
            recipient: rx
        });

        let join_handles = (0..quantity)
            .map(|_| {
                stage.run(ActorRunner {
                    actor: A::create(&context),
                    context: context.clone(),
                })
            })
            .collect::<FuturesUnordered<JoinHandle<()>>>();

        Broker {
            addr,
            actor_runner_handles: join_handles,
        }
    }
}

impl<A> Broker<A>
where
    A: Actor,
{
    #[cfg(feature = "use_tokio")]
    pub async fn wait_for_actors(&mut self) -> WaitForActors<'_> {
        WaitForActors(&mut self.actor_runner_handles)
    }

    #[cfg(feature = "use_async-std")]
    pub async fn wait_for_actors(&self) {
        futures::future::join_all(&self.actor_runner_handles).await
    }

    pub fn abort(&self) {
        for handle in &self.actor_runner_handles {
            #[cfg(feature = "use_async-std")]
            handle.cancel();
            #[cfg(feature = "use_tokio")]
            handle.abort();
        }
    }
}

impl<A> Deref for Broker<A> {
    type Target = Address<A>;

    fn deref(&self) -> &Self::Target {
        &self.addr
    }
}

pub struct WaitForActors<'a>(&'a mut FuturesUnordered<JoinHandle<()>>);

impl<'a> Future for WaitForActors<'a> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll_next(cx) {
            Poll::Ready(opt) => match opt {
                None => Poll::Ready(()),
                Some(res) => Poll::Pending, // todo: error handling
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
