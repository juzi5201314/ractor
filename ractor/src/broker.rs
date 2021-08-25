use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;

use crossfire::mpmc::bounded_future_both;
use futures::future::join_all;
use futures::stream::FuturesUnordered;
use futures::Stream;
use tokio::task::JoinHandle;

use crate::actor::Actor;
use crate::context::{Context, Inner};
use crate::actor_runner::ActorRunner;
use crate::stage::Stage;
use crate::LocalAddress;

pub struct Broker<A> {
    addr: Arc<LocalAddress<A>>,
    actor_runner_handles: FuturesUnordered<JoinHandle<()>>,
}

impl<A> Broker<A>
where
    A: Actor,
{
    pub async fn spawn(stage: &Stage, quantity: usize) -> Broker<A> {
        let (tx, rx) = bounded_future_both(A::MAIL_BOX_SIZE as usize);
        let addr = Arc::new(LocalAddress::new(tx));

        let context = Context {
            inner: Arc::new(Inner {
                self_addr: Arc::downgrade(&addr),
                stage: stage.clone(),
                recipient: rx,
            }),
        };

        let join_handles = join_all((0..quantity).map(|_| A::create(&context)))
            .await
            .into_iter()
            .map(|actor| {
                stage.run(ActorRunner {
                    actor,
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
    #[inline]
    pub fn addr(&self) -> &LocalAddress<A> {
        &self.addr
    }

    pub async fn wait_for_actors(&mut self) -> WaitForActors<'_> {
        WaitForActors(&mut self.actor_runner_handles)
    }

    pub fn abort(&self) {
        for handle in &self.actor_runner_handles {
            handle.abort();
        }
    }
}

impl<A> Deref for Broker<A> {
    type Target = LocalAddress<A>;

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
