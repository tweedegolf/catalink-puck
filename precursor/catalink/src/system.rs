use std::{collections::btree_set::BTreeSet, io::Write};

use crate::data;

const CONTACTS_DICT_NAME: &'static str = "CATALINK_CONTACTS";
const MESSAGES_DICT_NAME: &'static str = "CATALINK_MESSAGES";
const BUFFER_SIZE: usize = 1024;

pub struct System {
    pddb: pddb::Pddb,
}

impl System {
    pub fn new() -> Self {
        let mut pddb = pddb::Pddb::new();

        let (res, _) = pddb.try_mount();
        if !res {
            panic!("User directed to not mount the persistent storage");
        }

        let mut res = Self { pddb };

        // If message dictionary does not exist, pre-load it with some dummy messages.
        let dicts = res.pddb.list_dict(None).unwrap();
        let dicts: BTreeSet<String> = dicts.into_iter().collect();
        if !dicts.contains(CONTACTS_DICT_NAME) {
            log::info!("Contacts dict not initialized, loading dummy data");
            for contact in data::stubs::contacts() {
                res.add_contact(&contact).unwrap();
            }
        }
        if !dicts.contains(MESSAGES_DICT_NAME) {
            log::info!("Messages dict not initialized, loading dummy data");
            for message in data::stubs::messages() {
                res.add_message(&data::Message {
                    inner: message,
                    read: false,
                })
                .unwrap();
            }
        }

        res
    }

    /// Get generic entries from a dictionary.
    ///
    /// Will use O(n) memory for a listing of all the entries without the message body.
    /// Each individual message body is loaded only when the element is iterated.
    ///
    /// Sorted by key (or name) of entry.
    fn read_entries<T: serde::de::DeserializeOwned>(
        &mut self,
        dict_name: &str,
    ) -> Result<impl Iterator<Item = Result<T, xous::Error>>, xous::Error> {
        let mut entries = match self.pddb.read_dict(dict_name, None, None) {
            Ok(entries) => entries,

            // Dict was not defined, hence we have no messages stored.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),

            // When something else occurs, that means that there was some issue with the underlying storage.
            Err(_) => return Err(xous::Error::StorageError),
        };

        entries.sort_by_cached_key(|entry| entry.name.clone());

        Ok(entries.into_iter().map(move |entry| {
            log::info!("Found entry: {}:{}", dict_name, entry.name);
            let mut dict_key = self
                .pddb
                .get(
                    dict_name,
                    &entry.name,
                    Some(&entry.basis),
                    false,
                    false,
                    Some(64),
                    None::<fn()>,
                )
                .map_err(|_| xous::Error::StorageError)?;

            let mut buf = [0u8; BUFFER_SIZE];
            let (message, _) = postcard::from_io((&mut dict_key, &mut buf))
                .map_err(|_| xous::Error::SerializationError)?;

            Ok(message)
        }))
    }

    fn add_entry<T: serde::Serialize>(
        &mut self,
        dict_name: &str,
        key_name: &str,
        item: &T,
    ) -> Result<(), xous::Error> {
        let mut entry = self
            .pddb
            .get(dict_name, key_name, None, true, true, None, None::<fn()>)
            .map_err(|_| xous::Error::StorageError)?;

        postcard::to_io(item, &mut entry).map_err(|_| xous::Error::SerializationError)?;
        log::info!("Inserted {}:{}", dict_name, key_name);
        Ok(())
    }

    /// Get all the messages stored on the device.
    pub fn get_messages(
        &mut self,
    ) -> Result<impl Iterator<Item = Result<data::Message, xous::Error>>, xous::Error> {
        self.read_entries(MESSAGES_DICT_NAME)
    }

    pub fn add_message(&mut self, message: &data::Message) -> Result<(), xous::Error> {
        self.add_entry(MESSAGES_DICT_NAME, &message.key(), message)
    }

    /// Get all the contacts stored on the device.
    pub fn get_contacts(
        &mut self,
    ) -> Result<impl Iterator<Item = Result<data::Contact, xous::Error>>, xous::Error> {
        self.read_entries(CONTACTS_DICT_NAME)
    }

    pub fn add_contact(&mut self, contact: &data::Contact) -> Result<(), xous::Error> {
        self.add_entry(CONTACTS_DICT_NAME, &contact.name, contact)
    }
}
