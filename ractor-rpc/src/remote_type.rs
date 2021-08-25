use std::hash::{Hash, Hasher};

use serde::de::DeserializeOwned;
use serde::Serialize;

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
