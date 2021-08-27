use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;

use crossfire::mpmc::bounded_future_both;
use futures::stream::FuturesUnordered;
use futures::Stream;
use tokio::task::JoinHandle;

use crate::actor::Actor;
use crate::actor_runner::ActorRunner;
use crate::context::{GlobalContext, Inner};
use crate::{Context, LocalAddress};
use std::marker::PhantomData;

pub struct Broker<A> {
    addr: Arc<LocalAddress<A>>,
    actor_runner_handles: FuturesUnordered<JoinHandle<()>>,
}

impl<A> Broker<A>
where
    A: Actor,
{
    #[inline]
    pub async fn spawn_one() -> Self {
        Broker::spawn(1).await
    }

    pub async fn spawn(quantity: usize) -> Self {
        let (tx, rx) = bounded_future_both(A::MAIL_BOX_SIZE as usize);
        let addr = Arc::new(LocalAddress::new(tx));

        let global_context = GlobalContext {
            inner: Arc::new(Inner {
                self_addr: Arc::downgrade(&addr),
                recipient: rx,
            }),
        };

        let join_handles = FuturesUnordered::new();

        for _ in 0..quantity {
            let context = Context {
                global_context: global_context.clone(),
            };
            let actor = A::create(&context).await;
            join_handles.push(tokio::spawn(ActorRunner { actor, context }.run()))
        }

        Broker {
            addr,
            actor_runner_handles: join_handles,
        }
    }

    /// 将`GlobalContext::spawn`产生的Actor绑定到Broker
    #[inline]
    pub fn bind(&self, handle: SpawnHandle<A>) {
        self.actor_runner_handles.push(handle.join_handle)
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
                Some(res) => {
                    if let Err(err) = res {
                        if err.is_panic() {
                            log::warn!("Panic appears during the running of the actor: {}", err)
                        }
                    }
                    Poll::Pending
                }
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct SpawnHandle<A> {
    join_handle: JoinHandle<()>,
    marker: PhantomData<A>
}

impl<A> From<JoinHandle<()>> for SpawnHandle<A> where A: Actor {
    #[inline]
    fn from(h: JoinHandle<()>) -> Self {
        SpawnHandle {
            join_handle: h,
            marker: PhantomData::default()
        }
    }
}
