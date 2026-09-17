//! Descriptions of persistently stored data.

pub mod stubs;

use chrono::DateTime;
use puckmsg::ValidatedMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Contact {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Get the message body, assuming it is a valid UTF-8 string.
    pub fn body_opt(&self) -> Option<&str> {
        str::from_utf8(&self.inner.body).ok()
    }

    /// Get the message body, or the string "<<CORRUPTED MESSAGE>>" when it is not a valid UTF-8 string.
    pub fn body(&self) -> &str {
        self.body_opt().unwrap_or("<<CORRUPTED MESSAGE>>")
    }

    pub fn time_str(&self) -> String {
        self.inner.time.format("%d/%m/%y %H:%M").to_string()
    }
}
