use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{ready, FutureExt};

use crate::actor::{Actor, Rx};
use crate::broker::Broker;
use crate::message::MessageHandler;

pub use self::tokio::*;
use crate::stage::Scenes;
use futures::future::BoxFuture;
use std::ops::Deref;

mod tokio;

pub trait Executor<H>: Deref<Target = H> + Send + 'static {
    fn handle(&self) -> &H
    where
        H: ExecutorHandle;

    fn block_on<F>(self, fut: F) -> F::Output
    where
        F: Future;

    fn shutdown(self)
    where
        Self: Sized,
    {
    }
}

pub trait ExecutorHandle: Send + Sync + Clone + 'static {
    fn spawn_async<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;

    fn spawn_sync<F, O>(&self, task: F) -> JoinHandle<O>
    where
        F: FnOnce() -> O + Send + 'static,
        O: Send + 'static;
}

pub struct JoinHandle<O>(BoxFuture<'static, O>);

impl<O> JoinHandle<O> {
    pub fn new(fut: BoxFuture<'static, O>) -> Self {
        JoinHandle(fut)
    }
}

impl<O> Future for JoinHandle<O> {
    type Output = O;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.poll_unpin(cx)
    }
}

pub struct Show<A, H> {
    rx: Rx<A>,
    broker: Broker<A>,
    scenes: Scenes<H>,
}

impl<A, H> Show<A, H>
where
    H: ExecutorHandle,
    A: Actor,
{
    pub fn new(rx: Rx<A>, broker: Broker<A>, scenes: Scenes<H>) -> Self {
        Show { rx, broker, scenes }
    }

    pub async fn into_future(mut self) -> () {
        let scenes = self.scenes.clone();
        while let Ok(handle) = self.rx.recv().await {
            self = scenes
                .spawn_sync(move || {
                    (handle)(&mut self.broker.actor);
                    self
                })
                .await;
        }
        //todo: err
    }
}
