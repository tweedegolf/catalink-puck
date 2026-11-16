use std::collections::BTreeMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::data::{Contact, Message};
use crate::system::{ContactState, System};

#[derive(thiserror::Error, Debug)]
pub enum SyncError {
    #[error("puck error: {0}")]
    Puck(
        #[source]
        #[from]
        crate::puck::Error,
    ),
    #[error("xous error: {0:?}")]
    Xous(xous::Error),
}

enum Command {
    Sync,
    SendMessage {
        recipients: Vec<String>,
        content: String,
    },
    AddMessage {
        message: Message,
    },
    AddContact {
        name: String,
    },
}

pub struct SystemState {
    pub contacts: BTreeMap<Contact, ContactState>,
    pub messages: Vec<Message>,
    pub last_error: Option<SyncError>,
    pub our_contact: Contact,
    system: System,
}

impl SystemState {
    pub fn new() -> Self {
        let system = System::new();
        Self {
            contacts: BTreeMap::new(),
            messages: Vec::new(),
            last_error: None,
            our_contact: system.our_contact(),
            system,
        }
    }
}

pub struct Sync {
    sender: Sender<Command>,
    pub state: Arc<Mutex<SystemState>>,
}

impl Sync {
    pub fn our_contact(&self) -> Contact {
        self.state.lock().unwrap().our_contact.clone()
    }

    pub fn add_message(&self, message: Message) {
        _ = self.sender.send(Command::AddMessage { message });
    }

    pub fn sync(&self) {
        _ = self.sender.send(Command::Sync);
    }

    pub fn send_message(&self, recipients: Vec<String>, content: String) {
        _ = self.sender.send(Command::SendMessage {
            recipients,
            content,
        });
    }

    pub fn add_contact(&self, name: String) {
        _ = self.sender.send(Command::AddContact { name });
    }
}

pub struct SyncMessages {
    pub load_state: u32,
    pub sync: u32,
    pub send_succesful: u32,
    pub add_contact_succesful: u32,
    pub error: u32,
}

pub fn start_sync_thread(
    listener: xous::SID,
    sync_messages: SyncMessages,
) -> Result<Sync, xous::Error> {
    let listener = xous::connect(listener)?;
    let state = Arc::new(Mutex::new(SystemState::new()));
    let state_thread = state.clone();
    let (sender, receiver) = channel();
    thread::spawn(move || run(receiver, listener, sync_messages, state_thread));
    Ok(Sync { sender, state })
}

fn send_xous_message(listener: xous::CID, message: u32, number: u32) {
    xous::send_message(
        listener,
        xous::Message::new_scalar(message as usize, number as usize, 0, 0, 0),
    )
    .expect("Failed to send xous message");
}

fn sync(system: &mut crate::system::System) -> Result<usize, crate::puck::Error> {
    let mut puck = crate::puck::Puck::new()?;
    Ok(puck.sync(system)?)
}

fn send(
    our_contact_name: &str,
    recipients: &Vec<String>,
    body: &str,
) -> Result<Vec<puckmsg::ValidatedMessage>, crate::puck::Error> {
    let mut puck = crate::puck::Puck::new()?;
    puck.send(our_contact_name, recipients, body)
}

fn run(
    receiver: Receiver<Command>,
    listener: xous::CID,
    sync_messages: SyncMessages,
    state: Arc<Mutex<SystemState>>,
) {
    let mut system = System::new();
    let contact_state = system
        .derive_contact_state()
        .expect("Failed to initialize contact state");
    {
        let mut state = state.lock().unwrap();
        state.contacts = contact_state;
        if let Ok(messages) = system.get_messages() {
            state.messages = messages.filter_map(|message| message.ok()).collect()
        }
    }
    send_xous_message(listener, sync_messages.load_state, 0);
    loop {
        match receiver.recv() {
            Ok(message) => match message {
                Command::Sync => match sync(&mut system) {
                    Ok(count) => match system.get_messages() {
                        Ok(messages) => {
                            state.lock().unwrap().messages =
                                messages.filter_map(|message| message.ok()).collect();

                            send_xous_message(listener, sync_messages.sync, count as u32);
                        }
                        Err(err) => {
                            state.lock().unwrap().last_error = Some(SyncError::Xous(err));
                            send_xous_message(listener, sync_messages.error, 0);
                        }
                    },
                    Err(err) => {
                        state.lock().unwrap().last_error = Some(err.into());
                        send_xous_message(listener, sync_messages.error, 0);
                    }
                },
                Command::SendMessage {
                    recipients,
                    content,
                } => match send(
                    &state.lock().unwrap().our_contact.name,
                    &recipients,
                    &content,
                ) {
                    Ok(_) => {
                        send_xous_message(listener, sync_messages.send_succesful, 0);
                    }
                    Err(err) => {
                        state.lock().unwrap().last_error = Some(SyncError::Puck(err));
                        send_xous_message(listener, sync_messages.error, 0);
                    }
                },
                Command::AddContact { name } => match system.add_contact(&Contact { name }) {
                    Ok(_) => {
                        send_xous_message(listener, sync_messages.add_contact_succesful, 0);
                    }
                    Err(err) => {
                        state.lock().unwrap().last_error = Some(SyncError::Xous(err));
                    }
                },
                Command::AddMessage { message } => {
                    if let Err(err) = system.add_message(&message, true) {
                        log::error!("Failed to add message: {message:?} due to: {err:?}");
                    }
                }
            },
            Err(_) => return,
        }
    }
}
