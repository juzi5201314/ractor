use std::future::Future;
use std::ops::Deref;
use std::time::Duration;

use tokio::runtime::{Handle, Runtime};
use utils_soeur::builder;

use crate::executor::{Executor, ExecutorHandle, JoinError, JoinHandle};

builder!(pub TokioExecutorBuilder {
    async_workers: usize,
    sync_threads: usize,
    stack_size: usize,
    shutdown_timeout: Duration
});

impl Default for TokioExecutorBuilder {
    fn default() -> Self {
        TokioExecutorBuilder {
            async_workers: num_cpus::get(),
            sync_threads: 1024,
            stack_size: 1024 * 1024 * 4,
            shutdown_timeout: Duration::from_secs(5),
        }
    }
}

impl TokioExecutorBuilder {
    pub fn build(self) -> std::io::Result<TokioExecutor> {
        Ok(TokioExecutor::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(self.async_workers)
                .max_blocking_threads(self.sync_threads)
                .thread_stack_size(self.stack_size)
                .enable_all()
                .build()?,
            self.shutdown_timeout,
        ))
    }
}

pub struct TokioExecutor {
    rt: Runtime,
    handle: TokioExecutorHandle,

    shutdown_timeout: Duration,
}

#[derive(Clone)]
pub struct TokioExecutorHandle(Handle);

impl ExecutorHandle for TokioExecutorHandle {
    #[inline]
    fn block_on<F>(&self, fut: F) -> F::Output
        where
            F: Future,
    {
        self.0.block_on(fut)
    }

    #[inline]
    fn spawn_async<F>(&self, task: F) -> Box<dyn JoinHandle<F::Output>>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        Box::new(self.0.spawn(task))
    }

    #[inline]
    fn spawn_blocking<F, O>(&self, task: F) -> O where F: FnOnce() -> O {
        let _guard = self.0.enter();
        tokio::task::block_in_place(task)
    }
}

#[async_trait::async_trait]
impl<O> JoinHandle<O> for tokio::task::JoinHandle<O> where O: Send {
    async fn join(self) -> Result<O, JoinError> {
        self.await.map_err(Into::into)
    }

    fn abort(&self) {
        self.abort()
    }
}

impl From<tokio::task::JoinError> for JoinError {
    fn from(err: tokio::task::JoinError) -> Self {
        JoinError {
            msg: err.to_string(),
        }
    }
}

impl TokioExecutor {
    fn new(rt: Runtime, shutdown_timeout: Duration) -> Self {
        let handle = TokioExecutorHandle(rt.handle().clone());
        TokioExecutor {
            rt,
            handle,
            shutdown_timeout,
        }
    }
}

impl Deref for TokioExecutor {
    type Target = TokioExecutorHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.handle()
    }
}

impl Executor<TokioExecutorHandle> for TokioExecutor {
    #[inline]
    fn handle(&self) -> &TokioExecutorHandle {
        &self.handle
    }

    fn shutdown(self) {
        self.rt.shutdown_timeout(self.shutdown_timeout)
    }
}
