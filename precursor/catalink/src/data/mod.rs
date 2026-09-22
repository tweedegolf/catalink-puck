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

    /// All the [Contact]s that are relevant to this [Message].
    pub fn relevant_contacts(&self) -> impl Iterator<Item = Contact> {
        self.inner
            .recipients
            .iter()
            .chain(std::iter::once(&self.inner.sender))
            .map(|name| Contact { name: name.clone() })
    }

    /// If the [Contact] either sent or received the [Message].
    pub fn is_relevant_to(&self, contact: &Contact) -> bool {
        self.relevant_contacts()
            .find(|other| other == contact)
            .is_some()
    }
}
