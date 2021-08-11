#![cfg_attr(feature = "specialization", feature(specialization))]
#![feature(generic_associated_types)]
mod actor;
mod message;
mod stage;
mod executor;
mod broker;
mod director;


#[cfg(test)]
mod tests {
    use crate::actor::Actor;
    use crate::message::{Message, MessageHandler};
    use crate::stage::StageBuilder;
    use std::thread::sleep;
    use std::time::Duration;
    use std::future::Future;
    use futures::FutureExt;
    use futures::future::BoxFuture;
    use std::pin::Pin;

    #[test]
    fn add_count() {
        struct MyActor {
            count: u32
        }

        #[derive(Debug)]
        struct Add(u32);
        #[derive(Debug)]
        struct QueryCount;

        impl Actor for MyActor {
            const MAIL_BOX_SIZE: u32 = 24;

            fn create() -> Self {
                MyActor {
                    count: 0
                }
            }
        }

        impl MessageHandler<Add> for MyActor {
            type Output = u32;

            fn handle(&mut self, msg: Add) -> Self::Output {
                let old = self.count;
                self.count += msg.0;
                old
            }
        }

        impl MessageHandler<QueryCount> for MyActor {
            type Output = u32;

            fn handle(&mut self, _msg: QueryCount) -> u32 {
                let count = self.count;
                count
            }
        }

        impl Message for Add {}
        impl Message for QueryCount {}

        let stage = StageBuilder::new().build();

        stage.block_on(|s| async move {
            let my_actor = s.spawn::<MyActor>().await;

            let resp = my_actor.send(Add(3)).await.unwrap();
            let old = resp.recv().await;

            let now = my_actor.send(QueryCount).await.unwrap().recv().await;

            assert_eq!(now, old + 3);
        })
    }

    #[test]
    fn fib() {
        #[derive(Debug)]
        struct Fibonacci(u32);

        impl Message for Fibonacci {}

        struct MyActor {
            count: u128
        }

        impl MyActor {
            fn fib(&self, n: u32) -> u128 {
                // Pretending to be calculating
                sleep(Duration::from_secs(1));
                1
            }
        }

        impl Actor for MyActor {
            const MAIL_BOX_SIZE: u32 = 100;

            fn create() -> Self {
                MyActor {
                    count: 0
                }
            }
        }

        impl MessageHandler<Fibonacci> for MyActor {

            type Output = u128;

            fn handle(&mut self, msg: Fibonacci) -> Self::Output {
                self.count += self.fib(msg.0);
                self.count
            }
        }


        let stage = StageBuilder::new().build();

        stage.block_on(|s| async move {
            let my_actor = s.spawn::<MyActor>().await;
            let my_actor2 = s.spawn::<MyActor>().await;
            let mut v = Vec::new();
            for _ in 0..10 {
                v.push(my_actor.send(Fibonacci(1)).await.unwrap().recv());
                v.push(my_actor2.send(Fibonacci(1)).await.unwrap().recv());
            }
            // Each message needs 1s to process, 20 message synchronous calculation takes 20s.
            // But the two actors are parallel, so it only takes 10s
            let out = futures::future::join_all(v).await.iter().sum::<u128>();
            assert_eq!(out, 110);
        })
    }
}
