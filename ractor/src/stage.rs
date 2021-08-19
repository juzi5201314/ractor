use crate::actor::Actor;
use crate::broker::Broker;
use crate::executor::{Executor, JoinHandle, ActorRunner};

#[derive(Clone)]
pub struct Stage {
    pub executor: Executor,
}

impl Stage {
    #[cfg(feature = "use_tokio")]
    pub fn from_handle(handle: tokio::runtime::Handle) -> Self {
        Stage::with_executor(Executor::from_handle(handle))
    }

    pub fn with_executor(executor: Executor) -> Self {
        Stage {
            executor,
        }
    }

    #[inline]
    pub fn spawn<A>(&self, quantity: usize) -> Broker<A>
        where
            A: Actor,
    {
        Broker::spawn(self, quantity)
    }

    #[inline]
    pub fn run<A>(&self, runner: ActorRunner<A>) -> JoinHandle<()> where A: Actor {
        self.executor.spawn_async(runner.run())
    }
}
