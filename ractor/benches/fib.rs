#![feature(test)]

extern crate test;

use std::thread;
use test::{black_box, Bencher};

use futures::future::{join_all, FutureExt};

use ractor::{Actor, Context, Message, MessageHandler, Stage};
use tokio::runtime::Handle;

const FIB_TH: u32 = 47;
const PRE: u32 = 100;

#[bench]
fn bench_sync(b: &mut Bencher) {
    b.iter(|| {
        black_box((0..PRE).for_each(|_| {
            assert_eq!(black_box(fib(FIB_TH)), 2971215073);
        }));
    })
}

#[bench]
fn bench_os_thread(b: &mut Bencher) {
    b.iter(|| {
        let hs = (0..PRE)
            .map(|_| {
                thread::spawn(move || {
                    assert_eq!(fib(FIB_TH), 2971215073);
                })
            })
            .collect::<Vec<thread::JoinHandle<()>>>();

        hs.into_iter().for_each(|h| h.join().unwrap());
    })
}

#[bench]
fn bench_rayon(b: &mut Bencher) {
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    b.iter(|| {
        let hs = (0..PRE)
            .into_par_iter()
            .map(|_| {
                assert_eq!(fib(FIB_TH), 2971215073);
            })
            .collect::<Vec<_>>();
        black_box(hs);
    })
}

#[bench]
fn bench_tokio(b: &mut Bencher) {
    tokio::runtime::Builder::new_multi_thread()
        .build()
        .unwrap()
        .block_on(async {
            let stage = Stage::from_handle(Handle::current());
            let my_actor = stage.spawn::<MyActor>(PRE as usize);
            b.iter(|| {
                futures::executor::block_on(black_box(join_all(
                    (0..PRE)
                        .map(|_| my_actor.send(Fib(FIB_TH)).map(|r| r.expect("send failed")))
                        .map(|fut| fut.map(|h| h.recv().map(|r| r.expect("recv failed")))),
                )));
            });
        });
}

#[inline(always)]
fn fib(n: u32) -> u32 {
    let mut f = (1, 1);
    for step in 0..n {
        if step > 1 {
            f = (f.1, f.0 + f.1);
        }
    }
    f.1
}

#[derive(Debug, Message)]
struct Fib(u32);

#[derive(Default)]
struct MyActor;

impl Actor for MyActor {
    const MAIL_BOX_SIZE: u32 = 101;

    fn create(_ctx: &Context<Self>) -> Self
    where
        Self: Sized,
    {
        MyActor
    }
}

#[async_trait::async_trait]
impl MessageHandler<Fib> for MyActor {
    type Output = ();

    #[inline]
    async fn handle(&mut self, msg: Fib) -> Self::Output {
        assert_eq!(fib(msg.0), 2971215073)
    }
}
