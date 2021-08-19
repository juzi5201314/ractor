use std::future::Future;

use async_std::task;

pub type JoinHandle<T> = async_std::task::JoinHandle<T>;

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Executor
    }
}

impl Executor {
    #[inline]
    pub fn spawn_async<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        task::spawn(task)
    }

    #[inline]
    pub fn spawn_blocking<F, O>(&self, task: F) -> JoinHandle<O>
    where
        F: FnOnce() -> O,
    {
        task::spawn_blocking(task)
    }

    #[inline]
    pub fn block_on<F>(&self, fut: F) -> F::Output
    where
        F: Future,
    {
        task::block_on(fut)
    }
}
