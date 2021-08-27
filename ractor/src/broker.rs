use std::future::Future;
use std::marker::PhantomData;
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
use crate::context::{GlobalContext, Inner, State};
use crate::{Context, LocalAddress};

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
        Broker::spawn(1, false).await
    }

    /// `并发生成`在[`Actor::create`]有异步阻塞操作的时候效率会更高
    ///
    /// 但在普通情况下关闭`并发生成`效率更好
    pub async fn spawn(quantity: usize, concurrent_spawn: bool) -> Self {
        let (tx, rx) = bounded_future_both(A::MAIL_BOX_SIZE as usize);
        let addr = Arc::new(LocalAddress::new(tx));

        let global_context = GlobalContext {
            inner: Arc::new(Inner {
                self_addr: Arc::downgrade(&addr),
                recipient: rx,
            }),
        };

        let join_handles = if concurrent_spawn {
            futures::future::join_all(
                (0..quantity)
                    .map(|_| Context {
                        global_context: global_context.clone(),
                        state: State::Continue,
                    })
                    .map(|mut ctx| async move {
                        let actor = A::create(&mut ctx).await;
                        (actor, ctx)
                    }),
            )
            .await
            .into_iter()
            .map(|(actor, context)| tokio::spawn(ActorRunner { actor, context }.run()))
            .collect::<FuturesUnordered<JoinHandle<()>>>()
        } else {
            let join_handles = FuturesUnordered::new();
            for _ in 0..quantity {
                let mut context = Context {
                    global_context: global_context.clone(),
                    state: State::Continue,
                };
                let actor = A::create(&mut context).await;
                join_handles.push(tokio::spawn(ActorRunner { actor, context }.run()))
            }
            join_handles
        };

        Broker {
            addr,
            actor_runner_handles: join_handles,
        }
    }

    /// 将[`GlobalContext::spawn`]产生的Actor绑定到Broker
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

/// 实际上等同于[`JoinHandle<()>`]
///
/// 为了保证`<A>`是相同的
pub struct SpawnHandle<A> {
    join_handle: JoinHandle<()>,
    marker: PhantomData<A>,
}

impl<A> From<JoinHandle<()>> for SpawnHandle<A>
where
    A: Actor,
{
    #[inline]
    fn from(h: JoinHandle<()>) -> Self {
        SpawnHandle {
            join_handle: h,
            marker: PhantomData::default(),
        }
    }
}
