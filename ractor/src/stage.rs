use tokio::runtime::Handle;
use tokio::task::JoinHandle;

use crate::actor::Actor;
use crate::actor_runner::ActorRunner;
use crate::broker::Broker;

#[derive(Clone)]
pub struct Stage;

impl Stage {
    /// 不再需要传入执行器, `Handle`只是确保在tokio上下文中
    pub fn from_handle(_handle: Handle) -> Self {
        Stage
    }

    #[inline]
    pub async fn spawn<A>(&self, quantity: usize) -> Broker<A>
    where
        A: Actor,
    {
        Broker::spawn(self, quantity).await
    }

    #[inline]
    pub fn run<A>(&self, runner: ActorRunner<A>) -> JoinHandle<()>
    where
        A: Actor,
    {
        tokio::spawn(runner.run())
    }
}
