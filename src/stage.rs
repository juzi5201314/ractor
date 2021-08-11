use crate::executor::{Executor, TokioExecutorBuilder, TokioExecutor, Show, JoinHandle, ExecutorHandle, TokioExecutorHandle};
use crate::actor::{Actor, ActorHandler, Address};
use crate::director::Director;
use crate::broker::Broker;
use once_cell::sync::OnceCell;
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

    pub fn executor(mut self, executor: E) -> Self {
        self.executor = executor;
        self
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

impl Default for StageBuilder<TokioExecutor, TokioExecutorHandle> {
    fn default() -> Self {
        StageBuilder {
            executor: TokioExecutorBuilder::new().build().unwrap(),
            _marker: Default::default()
        }
    }
}

#[derive(Clone)]
pub struct Scenes<H> {
    handle: H
}

impl<H> Scenes<H> where H: ExecutorHandle {
    pub fn show<A>(&self, show: Show<A, H>) -> JoinHandle<()> where A: Actor {
        self.spawn_async(show.into_future())
    }

    pub async fn spawn<A>(&self) -> ActorHandler<A, H> where A: Actor {
        ActorHandler::create(self).await
    }
}

impl<H> Deref for Scenes<H> where H: ExecutorHandle {
    type Target = H;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

pub struct Stage<E, H> {
    executor: E,
    scene: Scenes<H>
}

impl<E, H> Stage<E, H> where E: Executor<H>, H: ExecutorHandle {
    pub fn scene(&self) -> &Scenes<H> {
        &self.scene
    }

    pub fn block_on<F>(self, func: fn(Scenes<H>) -> F) -> F::Output where F: Future {
        self.executor.block_on(func(self.scene))
    }

    pub fn shutdown(self) {
        self.executor.shutdown()
    }
}
