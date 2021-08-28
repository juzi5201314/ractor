use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use crossfire::mpmc::bounded_future_both;
use futures::future::join_all;
use tokio::task::JoinHandle;

use crate::actor::Actor;
use crate::actor_runner::ActorRunner;
use crate::context::{GlobalContext, Inner, State};
use crate::{Context, LocalAddress};

pub struct Broker<A> {
    addr: Arc<LocalAddress<A>>,
    actor_runner_handles: Vec<JoinHandle<()>>,
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
            join_all(
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
            .collect::<Vec<JoinHandle<()>>>()
        } else {
            let mut join_handles = Vec::with_capacity(quantity);
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
    pub fn bind(&mut self, handle: SpawnHandle<A>) {
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

    pub async fn wait_for_actors(self) {
        drop(self.addr);
        join_all(self.actor_runner_handles).await;
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
