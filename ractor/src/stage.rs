use crate::executor::{Show, JoinHandle, ExecutorHandle, Executor};
use crate::actor::{Actor, ActorCreator};

use crate::broker::Broker;

use std::future::Future;
use std::ops::Deref;
use std::marker::PhantomData;

pub struct StageBuilder<E, H> {
    executor: E,
    _marker: PhantomData<H>
}

impl<E, H> StageBuilder<E, H> where E: Executor<H>, StageBuilder<E, H>: Default, H: ExecutorHandle {
    pub fn new() -> StageBuilder<E, H> {
        Default::default()
    }

    pub fn build(self) -> Stage<E, H> {
        Stage {
            scene: Scenes {
                handle: self.executor.handle().clone()
            },
            executor: self.executor,
        }
    }
}

#[cfg(feature = "tokio-runtime")]
impl Default for StageBuilder<crate::executor::TokioExecutor, crate::executor::TokioExecutorHandle> {
    fn default() -> Self {
        StageBuilder {
            executor: crate::executor::TokioExecutorBuilder::new().build().unwrap(),
            _marker: Default::default()
        }
    }
}

#[derive(Clone)]
pub struct Scenes<H> {
    handle: H
}

impl<H> Scenes<H> where H: ExecutorHandle {
    #[inline]
    pub fn show<A>(&self, show: Show<A, H>) -> Box<dyn JoinHandle<()>> where A: Actor {
        self.spawn_async(show.into_future())
    }

    #[inline]
    pub fn spawn<A>(&self, quantity: usize) -> Broker<A, H> where A: Actor + ActorCreator {
        Broker::spawn(self, || A::create(), quantity)
    }
}

impl<H> Deref for Scenes<H> where H: ExecutorHandle {
    type Target = H;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

pub struct Stage<E, H> {
    executor: E,
    scene: Scenes<H>
}

impl<E, H> Stage<E, H> where E: Executor<H>, H: ExecutorHandle {
    #[inline]
    pub fn scene(&self) -> &Scenes<H> {
        &self.scene
    }

    pub fn block_on<F>(&self, func: fn(Scenes<H>) -> F) -> F::Output where F: Future {
        self.executor.block_on(func(self.scene.clone()))
    }

    pub fn shutdown(self) {
        self.executor.shutdown()
    }
}
