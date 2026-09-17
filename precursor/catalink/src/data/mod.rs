//! Descriptions of persistently stored data.

pub mod stubs;

use std::time::SystemTime;

use chrono::DateTime;
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
        let timestamp = self
            .inner
            .time
            .signed_duration_since(DateTime::UNIX_EPOCH)
            .num_seconds();
        format!("{}-{}", timestamp, self.inner.sender)
    }
}
