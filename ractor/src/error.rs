use crossfire::mpmc::TrySendError;
use std::fmt::{Debug, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {}

pub struct ChannelSendError<T>(T);

impl<T> From<crossfire::mpmc::SendError<T>> for ChannelSendError<T> {
    fn from(err: crossfire::mpmc::SendError<T>) -> Self {
        ChannelSendError(err.0)
    }
}

impl<T> Debug for ChannelSendError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The message could not be sent because the channel is disconnected."
        )
    }
}

impl<T> ChannelSendError<T> {
    pub fn recover(self) -> T {
        self.0
    }
}

pub enum ChannelTrySendError<T> {
    Full(T),
    Disconnected(T),
}

impl<T> From<crossfire::mpmc::TrySendError<T>> for ChannelTrySendError<T> {
    fn from(err: crossfire::mpmc::TrySendError<T>) -> Self {
        match err {
            TrySendError::Full(t) => ChannelTrySendError::Full(t),
            TrySendError::Disconnected(t) => ChannelTrySendError::Disconnected(t),
        }
    }
}

impl<T> Debug for ChannelTrySendError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The message could not be sent because the channel is {}.",
            if matches!(self, ChannelTrySendError::Full(_)) {
                "full"
            } else {
                "disconnected"
            }
        )
    }
}

impl<T> ChannelTrySendError<T> {
    pub fn recover(self) -> T {
        match self {
            ChannelTrySendError::Full(t) => t,
            ChannelTrySendError::Disconnected(t) => t
        }
    }
}
