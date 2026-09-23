//! Integration between backend and local system.
//!
//! Allows for downloading messages or sending messages.

use crate::system::System;

pub struct Puck {
    broker: puckfs::Broker<trng::Trng>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Xous error")]
    Xous,
    #[error("The broker emitted an error")]
    Broker(#[from] puckfs::BrokerError),
    #[error("Failed to receive message")]
    ReceiveError(#[from] puckfs::RecvError),
    #[error("Failed to send message")]
    SendError(#[from] puckfs::SendError),
    #[error("Failed to verify or decrypt a packet")]
    CryptoError,
    #[error("Failed to validate message")]
    ValidateError,
    #[error("Unexpected response")]
    UnexpectedResponse,
}

impl From<xous::Error> for Error {
    fn from(_value: xous::Error) -> Self {
        Error::Xous
    }
}

impl From<puckmsg::ValidateError> for Error {
    fn from(_value: puckmsg::ValidateError) -> Self {
        Error::ValidateError
    }
}

struct ReadOperation<'a> {
    puck: &'a mut Puck,
    dirname: &'a str,
    dirview: puckfs::DirView,
}

impl ReadOperation<'_> {
    pub fn iter(&mut self) -> impl Iterator<Item = Result<puckmsg::ValidatedMessage, Error>> {
        self.dirview.iter().map(|entry| {
            let name = String::from_utf8_lossy(entry.file_name);
            let path = format!("{}/{name}", self.dirname);
            self.puck
                .broker
                .send(puckfs::OutMessage::ReadFile(path.as_bytes()))?;

            let puckfs::InMessage::ReadFile(contents) = self.puck.recv()? else {
                return Err(Error::UnexpectedResponse);
            };

            let message = puckmsg::validate(&contents);
            Ok(message?)
        })
    }
}

impl Puck {
    pub fn new() -> Result<Self, Error> {
        let rng = trng::Trng::new(&xous_names::XousNames::new()?)?;
        let broker = puckfs::Broker::open("localhost:14400", rng)?;

        let mut res = Self { broker };
        res.handshake()?;
        Ok(res)
    }

    /// Perform a handshake "hello" operation, required when opening a new broker session.
    fn handshake(&mut self) -> Result<(), Error> {
        // Unwrap: we assume that we are beyond the UNIX EPOCH.
        self.broker.send(puckfs::OutMessage::hello().unwrap())?;
        let puckfs::InMessage::Hello(s, r) = self.recv()? else {
            return Err(Error::UnexpectedResponse);
        };
        self.broker.reposition(s, r + 1);
        Ok(())
    }

    /// Receive any package.
    fn recv(&mut self) -> Result<puckfs::InMessage, Error> {
        loop {
            match self.broker.read() {
                Ok(msg) => {
                    return Ok(msg);
                }
                Err(puckfs::RecvError::Incomplete) => {
                    self.broker.perform_io().map_err(|_| Error::CryptoError)?
                }
                Err(err) => return Err(err.into()),
            }
        }
    }

    /// List all the files in a directory.
    fn dir(&mut self, dir: &str) -> Result<puckfs::DirView, Error> {
        self.broker
            .send(puckfs::OutMessage::ReadDir(dir.as_bytes()))?;
        let puckfs::InMessage::ReadDir(entries) = self.recv()? else {
            return Err(Error::UnexpectedResponse);
        };
        Ok(entries)
    }

    /// Get all the files in a directory.
    fn get<'a>(&'a mut self, dir: &'a str) -> Result<ReadOperation<'a>, Error> {
        let dirview = self.dir(&format!("{dir}/"))?;
        Ok(ReadOperation {
            puck: self,
            dirname: dir,
            dirview,
        })
    }

    /// Receive messages from backend.
    pub fn sync(&mut self, system: &mut System) -> Result<usize, Error> {
        let mut count = 0usize;
        for entry in self.get("in")?.iter() {
            let Ok(entry) = entry else {
                log::warn!("Invalid message received, ignoring");
                continue;
            };

            let message = crate::data::Message {
                inner: entry,
                read: false,
            };

            let inserted = system.add_message(&message, false)?;

            if inserted {
                count += 1;
            }
        }
        Ok(count)
    }
}
