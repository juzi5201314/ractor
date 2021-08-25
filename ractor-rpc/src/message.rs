use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{serialize, RemoteType};

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

    let msg_header: MessageHeader = crate::deserialize(&serialize(&msg).unwrap()).unwrap();

    assert_eq!(msg_header.identity_id, msg.identity_id);
    assert_eq!(msg_header.unique_id, msg.unique_id);
    assert_eq!(&msg_header.payload, msg.payload.as_bytes());
}
