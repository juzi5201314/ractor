use criterion::{black_box, Criterion};
use criterion::{criterion_group, criterion_main};
use tokio::runtime::{Builder, Runtime};

use ractor::{Actor, Broker, Context};
use criterion::async_executor::AsyncExecutor;
use std::future::Future;

fn multi_thread(c: &mut Criterion) {
    let rt = TokioRt(Builder::new_multi_thread().enable_all().build().unwrap());
    c.bench_function("spawn 1", |b| {
        b.to_async(&rt).iter(|| async {
            let my_actor = Broker::<MyActor>::spawn(1, false).await;
            black_box(my_actor)
        });
    });
    c.bench_function("sync spawn 100", |b| {
        b.to_async(&rt).iter(|| async {
            let my_actor = Broker::<MyActor>::spawn(10, false).await;
            black_box(my_actor)
        });
    });
    c.bench_function("concurrent spawn 100", |b| {
        b.to_async(&rt).iter(|| async {
            let my_actor = Broker::<MyActor>::spawn(10, true).await;
            black_box(my_actor)
        });
    });
}

criterion_group!(benches, multi_thread);
criterion_main!(benches);

#[derive(Default)]
struct MyActor;

#[async_trait::async_trait]
impl Actor for MyActor {
    const MAIL_BOX_SIZE: u32 = 100;

    async fn create(_ctx: &mut Context<Self>) -> Self
    where
        Self: Sized,
    {
        MyActor
    }
}

struct TokioRt(Runtime);

impl AsyncExecutor for &TokioRt {
    #[inline]
    fn block_on<T>(&self, future: impl Future<Output=T>) -> T {
        self.0.block_on(future)
    }
}
