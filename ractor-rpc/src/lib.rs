use std::hash::{Hash, Hasher};

use bincode::Options;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use uuid::Uuid;

pub use error::Error;
pub use ws::WsError;

//mod quic;
mod error;
mod ws;

pub type RpcServer = ws::server::Server;
pub type RpcClient = ws::client::Client;

/// 实现了`RemoteType`表示该类型可以被远程发送或者接收
/// 为基础类型实现了`RemoteType`
pub trait RemoteType: Serialize + DeserializeOwned + Send {
    type Identity: Hash;

    /// 唯一标识符
    /// 应该保证: A.identity() == B.identity() && deserialize::<B>(serialize(A)) && deserialize::<A>(serialize(B))
    fn identity() -> Self::Identity;

    // todo: optimize
    #[inline]
    fn identity_id() -> u64 {
        let mut hasher = ahash::AHasher::default();
        Self::identity().hash(&mut hasher);
        hasher.finish()
    }
}

macro_rules! builtin {
    ($ty:ty, $name:expr) => {
        impl RemoteType for $ty {
            type Identity = &'static str;

            fn identity() -> Self::Identity {
                $name
            }
        }
    };
}

builtin!((), "builtin::unit");
builtin!(String, "builtin::string");
builtin!(isize, "builtin::isize");
builtin!(usize, "builtin::usize");
builtin!(i8, "builtin::i8");
builtin!(i16, "builtin::i16");
builtin!(i32, "builtin::i32");
builtin!(i64, "builtin::i64");
builtin!(u8, "builtin::u8");
builtin!(u16, "builtin::u16");
builtin!(u32, "builtin::u32");
builtin!(u64, "builtin::u64");
builtin!(f32, "builtin::f32");
builtin!(f64, "builtin::f64");
builtin!(char, "builtin::char");

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub identity_id: u64,
    pub unique_id: u128,
    pub payload: Vec<u8>,
}

impl Message {
    pub fn new<T>(ty: T) -> Self
    where
        T: RemoteType,
    {
        Message {
            identity_id: T::identity_id(),
            payload: serialize(&ty).unwrap(),
            unique_id: Uuid::new_v4().as_u128(),
        }
    }
}

#[test]
fn test_message() {
    let msg = Message::new(String::from("hello world"));

    let msg_header: MessageHeader = deserialize(&serialize(&msg).unwrap()).unwrap();

    assert_eq!(msg_header.identity_id, msg.identity_id);
    assert_eq!(msg_header.unique_id, msg.unique_id);
    assert_eq!(&msg_header.payload, msg.payload.as_bytes());
}

#[doc(hidden)]
#[inline]
pub fn serialize<T>(data: &T) -> Result<Vec<u8>, bincode::Error>
where
    T: Serialize,
{
    bincode::DefaultOptions::default()
        .with_little_endian()
        .serialize(data)
}

#[doc(hidden)]
#[inline]
pub fn deserialize<T>(data: &[u8]) -> Result<T, bincode::Error>
where
    T: DeserializeOwned,
{
    bincode::DefaultOptions::default()
        .with_little_endian()
        .deserialize(data)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
