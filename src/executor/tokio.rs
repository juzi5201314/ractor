use tokio::runtime::{Runtime, Handle};
use crate::executor::{Executor, Show, JoinHandle, ExecutorHandle};
use utils_soeur::builder;
use crate::actor::Actor;
use std::time::Duration;
use std::future::Future;
use futures::FutureExt;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use std::ops::Deref;

builder!(pub TokioExecutorBuilder {
    async_workers: usize,
    sync_threads: usize,
    stack_size: usize
});

impl Default for TokioExecutorBuilder {
    fn default() -> Self {
        TokioExecutorBuilder {
            async_workers: num_cpus::get(),
            sync_threads: 1024,
            stack_size: 1024 * 1024 * 4
        }
    }
}

impl TokioExecutorBuilder {
    pub fn build(self) -> std::io::Result<TokioExecutor> {
        Ok(TokioExecutor::new(tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .max_blocking_threads(self.sync_threads)
            .thread_stack_size(self.stack_size)
            .enable_all()
            .build()?))
    }
}

pub struct TokioExecutor(Runtime, TokioExecutorHandle);

#[derive(Clone)]
pub struct TokioExecutorHandle(Handle);

impl ExecutorHandle for TokioExecutorHandle {
    fn spawn_async<F>(&self, task: F) -> JoinHandle<F::Output> where F: Future + Send + 'static, F::Output: Send + 'static {
        let handle = self.0.spawn(task);

        JoinHandle::new(Box::pin(async {
            handle.await.unwrap()
        }))
    }

    fn spawn_sync<F, O>(&self, task: F) -> JoinHandle<O> where F: FnOnce() -> O + Send + 'static, O: Send + 'static {
        let handle = self.0.spawn_blocking(task);

        JoinHandle::new(Box::pin(async {
            handle.await.unwrap()
        }))
    }
}

impl TokioExecutor {
    fn new(rt: Runtime) -> Self {
        let h = TokioExecutorHandle(rt.handle().clone());
        TokioExecutor(rt, h)
    }
}

impl Deref for TokioExecutor {
    type Target = TokioExecutorHandle;

    fn deref(&self) -> &Self::Target {
        self.handle()
    }
}

impl Executor<TokioExecutorHandle> for TokioExecutor {
    fn handle(&self) -> &TokioExecutorHandle {
        &self.1
    }

    fn block_on<F>(self, fut: F) -> F::Output where F: Future {
        self.0.block_on(fut)
    }

    fn shutdown(self) {
        //RT.take().unwrap().shutdown_timeout(Duration::from_secs(3))
    }
}
