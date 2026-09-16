//! Descriptions of persistently stored data.

pub mod stubs;

use puckmsg::ValidatedMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub inner: ValidatedMessage,
    pub read: bool,
}

impl Message {
    /// Derive a key used to store the entry.
    ///
    /// The key is also used to alphanumerically sort the entire set of messages.
    pub fn key(&self) -> String {
        format!("{:?}-{}", self.inner.time, self.inner.sender)
    }
}
