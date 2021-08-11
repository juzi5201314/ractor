use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::channel::oneshot;
use futures::future::BoxFuture;
use futures::{ready, FutureExt};

use crate::actor::Actor;
use crate::executor::{Executor, ExecutorHandle, JoinHandle};
use crate::stage::{Scenes, Stage};

pub trait Message: Debug + Send {}

pub trait MessageHandler<M>: Sized + Send
where
    Self: Actor,
    M: Message,
{
    type Output: MessageResponse;

    fn handle(&mut self, msg: M) -> Self::Output;
}

pub trait MessageResponse: Sized {
    type Output;
    fn into_response<H>(self, stage: Scenes<H>) -> Response<Self::Output>
    where
        H: ExecutorHandle;
}

pub struct Response<O>(O);

pub struct ResponseHandle<O>(pub(crate) oneshot::Receiver<Response<O>>);

impl<O> ResponseHandle<O> {
    pub async fn recv(self) -> O {
        let resp = self.0.await.unwrap();
        resp.0
    }
}

unsafe impl<O> Send for Response<O> {}

macro_rules! impl_type_sync {
    ($_ty:ty) => {
        impl MessageResponse for $_ty {
            type Output = $_ty;

            fn into_response<H>(self, scenes: Scenes<H>) -> Response<Self::Output>
            where
                H: ExecutorHandle,
            {
                Response(self)
            }
        }
    };
}

impl_type_sync!(());
impl_type_sync!(String);
impl_type_sync!(&'static str);
impl_type_sync!(isize);
impl_type_sync!(i8);
impl_type_sync!(i16);
impl_type_sync!(i32);
impl_type_sync!(i64);
impl_type_sync!(i128);
impl_type_sync!(usize);
impl_type_sync!(u8);
impl_type_sync!(u16);
impl_type_sync!(u32);
impl_type_sync!(u64);
impl_type_sync!(u128);
impl_type_sync!(f32);
impl_type_sync!(f64);
impl_type_sync!(char);

impl<T> MessageResponse for Option<T>
where
    T: Send + 'static,
{
    type Output = Option<T>;

    fn into_response<H>(self, scenes: Scenes<H>) -> Response<Self::Output>
    where
        H: ExecutorHandle,
    {
        Response(self)
    }
}

impl<T, E> MessageResponse for Result<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    type Output = Result<T, E>;

    fn into_response<H>(self, scenes: Scenes<H>) -> Response<Self::Output>
    where
        H: ExecutorHandle,
    {
        Response(self)
    }
}

#[cfg(feature = "specialization")]
default impl<T: MessageResponse<Output = T>> MessageResponse for T {
    type Output = T;

    default fn into_response<H>(self, scenes: Scenes<H>) -> Response<Self::Output>
    where
        H: ExecutorHandle,
    {
        Response(self)
    }
}

/*impl<'a, T> MessageResponse for BoxFuture<'a, T> where T: Send + 'a {
    type Output = T;

    fn into_response<H>(self, scenes: Scenes<H>) -> Response<Self::Output> where H: ExecutorHandle {
        Response(self)
    }
}
*/
