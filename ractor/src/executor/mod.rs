use std::future::Future;
use std::ops::Deref;

use crate::actor::Actor;
use crate::envelope::{Envelope, MailBoxRx};
use crate::stage::Scenes;

#[cfg(feature = "tokio-runtime")]
pub use self::tokio::*;

#[cfg(feature = "tokio-runtime")]
mod tokio;

pub trait Executor<H>: Deref<Target = H> + Send + 'static {
    fn handle(&self) -> &H
    where
        H: ExecutorHandle;

    fn block_on<F>(&self, fut: F) -> F::Output
    where
        F: Future;

    fn shutdown(self)
    where
        Self: Sized,
    {}
}

pub trait ExecutorHandle: Send + Sync + Clone + 'static {
    fn spawn_async<F>(&self, task: F) -> Box<dyn JoinHandle<F::Output>>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;

    fn spawn_blocking<F, O>(&self, task: F) -> O
        where
            F: FnOnce() -> O;
}

#[derive(Debug)]
pub struct JoinError {
    msg: String,
}

#[async_trait::async_trait]
pub trait JoinHandle<O> {
    async fn join(self) -> Result<O, JoinError>;

    fn abort(&self);
}

pub struct Show<A, H> {
    rx: MailBoxRx<A>,
    actor: A,
    scenes: Scenes<H>,
}

impl<A, H> Show<A, H>
where
    H: ExecutorHandle,
    A: Actor,
{
    pub fn new(rx: MailBoxRx<A>, actor: A, scenes: Scenes<H>) -> Self {
        Show { rx, actor, scenes }
    }

    pub async fn into_future(mut self) -> () {
        let scenes = self.scenes.clone();

        self.actor.started(&scenes).await;

        while let Ok(envelope) = self.rx.recv().await {
            match envelope {
                Envelope::Sync(handle) => {
                    scenes
                        .spawn_blocking(|| {
                            (handle)(&mut self.actor);
                        });
                }
                Envelope::Async(handle) => {
                    (handle)(&mut self.actor).await;
                }
                Envelope::Stop => break
            }
        }
        self.actor.stopped(&scenes).await;
    }
}
