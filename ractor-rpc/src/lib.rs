use bincode::Options;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub use error::Error;
pub use message::Message;
pub use remote_type::RemoteType;
pub use ws::WsError;

//mod quic;
mod error;
mod message;
mod remote_type;
mod ws;

pub type RpcServer = ws::server::Server;
pub type RpcClient = ws::client::Client;

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
