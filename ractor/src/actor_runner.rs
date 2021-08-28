use std::panic::AssertUnwindSafe;

use futures::FutureExt;

use crate::actor::Actor;
use crate::context::Context;
use crate::State;

pub struct ActorRunner<A> where A: Actor {
    pub actor: A,
    pub context: Context<A>,
}
macro_rules! reach_state {
    ($state:expr, { $($p:pat_param => $c:expr),* }) => {
        match $state {
            $($p => $c),*,
            State::Pause(notify) => {
                notify.notified().await;
            }
            State::Yield => {
                tokio::task::yield_now().await;
            }
            State::Sleep(t) => tokio::time::sleep(std::time::Duration::from_millis(*t)).boxed().await,
            State::Abort => {}
        }
        $state.clear();
    };
}

impl<A> ActorRunner<A>
where
    A: Actor,
{
    #[inline]
    pub async fn run(mut self) -> () {
        let mut restart_count = 0;

        'main_loop: loop {
            match AssertUnwindSafe(async {
                'life_cycle: loop {
                    // 进入生命周期后的状态
                    reach_state!(&mut self.context.state, {
                        State::Continue => {},
                        State::Reset => {
                            // 不能在start之前就reset
                        },
                        State::Stop => break 'life_cycle
                    });
                    self.actor.started(&mut self.context).await;

                    let pos = 'started: loop {
                        // 开始之后的状态
                        reach_state!(&mut self.context.state, {
                            State::Continue => {},
                            State::Stop => break 'started StoppingPosition::Starting,
                            State::Reset => {
                                self.actor.reset(&mut self.context).await;
                                continue 'life_cycle;
                            }
                        });

                        #[allow(unused_labels)]
                        'message_loop: while let Ok(envelope) = self.context.recipient.recv().await
                        {
                            (envelope)(&mut self.actor, &mut self.context).await;

                            // 处理完消息之后的状态
                            reach_state!(&mut self.context.state, {
                                State::Continue => {},
                                State::Reset => {
                                    self.actor.reset(&mut self.context).await;
                                    continue 'life_cycle;
                                },
                                State::Stop => {
                                    break 'started StoppingPosition::Message;
                                }
                            });
                        }
                        break 'started StoppingPosition::End;
                    };
                    self.actor.stopped(&mut self.context, pos).await;
                    // 停止之后的状态
                    reach_state!(&mut self.context.state, {
                        State::Continue => {},
                        State::Stop => {},
                        State::Reset => {
                            // 如果消息通道关闭了, 那么就不可能再重启
                            if !matches!(pos, StoppingPosition::End) {
                                self.actor.reset(&mut self.context).await;
                                continue 'life_cycle;
                            }
                        }
                    });
                    break 'life_cycle;
                }
            })
            .catch_unwind()
            .await
            {
                Ok(_) => break 'main_loop,
                Err(err) => {
                    self.context.state = State::Abort;
                    self.actor.catch_unwind(err, &mut self.context);
                    if matches!(self.context.state, State::Reset) && restart_count < A::MAX_RESTARTS
                    {
                        restart_count += 1;
                        self.actor.reset(&mut self.context).await;
                        continue 'main_loop;
                    } else {
                        break 'main_loop;
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum StoppingPosition {
    Starting,
    Message,
    End,
}
