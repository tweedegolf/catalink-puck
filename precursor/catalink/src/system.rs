use crate::data;
use std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

const CONTACTS_DICT_NAME: &'static str = "CATALINK_CONTACTS";
const MESSAGES_DICT_NAME: &'static str = "CATALINK_MESSAGES";
const BUFFER_SIZE: usize = 1024;

pub struct System {
    pddb: pddb::Pddb,
}

impl System {
    pub fn new() -> Self {
        let pddb = pddb::Pddb::new();

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
                res.add_message(
                    &data::Message {
                        inner: message,
                        read: false,
                    },
                    true,
                )
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

    /// Add an entry to the PDDB with serialized `item` as the contents.
    ///
    /// If `overwrite` is not set, will not overwrite the contents if the entry already exists.
    fn add_entry<T: serde::Serialize>(
        &mut self,
        dict_name: &str,
        key_name: &str,
        item: &T,
        overwrite: bool,
    ) -> Result<bool, xous::Error> {
        let mut entry_res =
            self.pddb
                .get(dict_name, key_name, None, false, false, None, None::<fn()>);

        let mut inserted = false;

        // If the entry does not yet exist, insert it.
        if let Err(e) = &entry_res
            && e.kind() == std::io::ErrorKind::NotFound
        {
            entry_res = self
                .pddb
                .get(dict_name, key_name, None, true, true, None, None::<fn()>);
            inserted = true;
        }

        let mut entry = entry_res.map_err(|_| xous::Error::StorageError)?;

        if inserted || overwrite {
            postcard::to_io(item, &mut entry).map_err(|_| xous::Error::SerializationError)?;
        }

        Ok(inserted)
    }

    /// Get all the messages stored on the device.
    pub fn get_messages(
        &mut self,
    ) -> Result<impl Iterator<Item = Result<data::Message, xous::Error>>, xous::Error> {
        self.read_entries(MESSAGES_DICT_NAME)
    }

    pub fn add_message(
        &mut self,
        message: &data::Message,
        overwrite: bool,
    ) -> Result<bool, xous::Error> {
        self.add_entry(MESSAGES_DICT_NAME, &message.key(), message, overwrite)
    }

    /// Get all the contacts stored on the device.
    pub fn get_contacts(
        &mut self,
    ) -> Result<impl Iterator<Item = Result<data::Contact, xous::Error>>, xous::Error> {
        self.read_entries(CONTACTS_DICT_NAME)
    }

    pub fn add_contact(&mut self, contact: &data::Contact) -> Result<bool, xous::Error> {
        self.add_entry(CONTACTS_DICT_NAME, &contact.name, contact, false)
    }

    pub fn our_contact(&self) -> data::Contact {
        data::stubs::our_contact()
    }
}

#[derive(Default, Debug)]
pub struct ContactState {
    pub last_message: Option<data::Message>,
    pub unread_count: usize,
}

impl System {
    pub fn derive_contact_state(
        &mut self,
    ) -> Result<BTreeMap<data::Contact, ContactState>, xous::Error> {
        let mut contact_state: BTreeMap<data::Contact, ContactState> = BTreeMap::new();

        for contact in self.get_contacts()? {
            if let Ok(contact) = contact {
                contact_state.insert(contact, ContactState::default());
            }
        }

        for message in self.get_messages()? {
            if let Ok(message) = message {
                for contact in message.relevant_contacts() {
                    if let Some(contact_state) = contact_state.get_mut(&contact) {
                        if !message.read {
                            contact_state.unread_count += 1;
                        }

                        // When current message is the last for this particular contact (or the only one)
                        if contact_state
                            .last_message
                            .as_ref()
                            .is_none_or(|last_message| last_message.inner.time < message.inner.time)
                        {
                            contact_state.last_message = Some(message.clone());
                        }
                    }
                }
            }
        }

        Ok(contact_state)
    }
}
