use std::panic::AssertUnwindSafe;

use futures::FutureExt;

use crate::actor::Actor;
use crate::context::Context;
use crate::State;

pub struct ActorRunner<A> {
    pub actor: A,
    pub context: Context<A>,
}

impl<A> ActorRunner<A>
where
    A: Actor,
{
    pub async fn run(mut self) -> () {
        let mut restart_count = 0;

        // 无副作用的状态:
        //
        // `Status::Continue`为默认状态, 不需要重置状态
        // `Status::Stop`之后停止, 不需要重置状态
        // `Status::Reset`会在'life_cycle之后的第一次状态中重置
        // `Status::Abort`为只读状态

        'main_loop: loop {
            match AssertUnwindSafe(async {
                'life_cycle: loop {
                    // 进入生命周期后的状态
                    match &mut self.context.state {
                        State::Continue => {}
                        state @ State::Reset => {
                            // 不能在start之前就reset
                            state.clear();
                        }
                        State::Stop => break 'life_cycle,
                        state => state.reach().await,
                    }
                    self.actor.started(&mut self.context).await;

                    let pos = 'started: loop {
                        // 开始之后的状态
                        match &mut self.context.state {
                            State::Continue => {}
                            State::Stop => break 'started StoppingPosition::Starting,
                            State::Reset => {
                                self.actor.reset(&mut self.context).await;
                                continue 'life_cycle;
                            }
                            state => state.reach().await,
                        }

                        #[allow(unused_labels)]
                        'message_loop: while let Ok(envelope) = self.context.recipient.recv().await
                        {
                            (envelope)(&mut self.actor, &mut self.context).await;

                            // 处理完消息之后的状态
                            match &mut self.context.state {
                                State::Continue => {}
                                State::Reset => {
                                    self.actor.reset(&mut self.context).await;
                                    continue 'life_cycle;
                                }
                                State::Stop => {
                                    break 'started StoppingPosition::Message;
                                }
                                state => state.reach().await,
                            }
                        }
                        break 'started StoppingPosition::End;
                    };
                    self.actor.stopped(&mut self.context, pos).await;
                    // 停止之后的状态
                    match &mut self.context.state {
                        State::Continue | State::Stop => {}
                        State::Reset => {
                            // 如果消息通道关闭了, 那么就不可能再重启
                            if !matches!(pos, StoppingPosition::End) {
                                self.actor.reset(&mut self.context).await;
                                continue 'life_cycle;
                            }
                        }
                        state => state.reach().await,
                    }
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
                    if matches!(self.context.state, State::Reset)
                        && restart_count < A::MAX_RESTARTS
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
    End
}
