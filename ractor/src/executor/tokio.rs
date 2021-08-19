use std::future::Future;

use tokio::runtime::Handle;

pub type JoinHandle<T> = tokio::task::JoinHandle<T>;

#[derive(Clone)]
pub struct Executor {
    handle: Handle,
}

impl Executor {
    pub fn from_handle(handle: Handle) -> Self {
        Executor { handle }
    }
}

impl Executor {
    #[inline]
    pub fn spawn_async<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.handle.spawn(task)
    }

    #[inline]
    pub fn spawn_blocking<F, O>(&self, task: F) -> JoinHandle<O>
    where
        F: FnOnce() -> O + Send + 'static,
        O: Send + 'static
    {
        self.handle.spawn_blocking(task)
    }
}
