//! Convenience high-level API that integrates puckmsg and puckfs layers.
//! This is not a fully async API (in the normal sense) since some of the operations
//! might temporarily block.
//! But we don't expect a full async runtime to be used on the Precursor.

use super::{EnvelopeError, ValidateError, ValidatedMessage};
use puckfs::{Broker, DirView, InMessage, OutMessage, RecvError, SendError};

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct Listen<'a>(&'a mut Broker);
struct Sender<'a>(&'a mut Broker, OutMessage<'a>);

impl Future for Listen<'_> {
    type Output = Result<InMessage, RecvError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let broker = &mut self.get_mut().0;

        let value = match broker.read() {
            Err(RecvError::Incomplete) => {
                if let Err(err) = broker.perform_io() {
                    Err(RecvError::Crypto(err))
                } else {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
            }
            result => result,
        };

        Poll::Ready(value)
    }
}

impl Future for Sender<'_> {
    type Output = Result<(), SendError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self(broker, msg) = self.get_mut();

        let value = match broker.send(*msg) {
            Err(SendError::Busy) => {
                if let Err(err) = broker.perform_io() {
                    Err(SendError::Crypto(err))
                } else {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
            }
            result => result,
        };

        Poll::Ready(value)
    }
}

macro_rules! expect {
    (let $what: pat = $name:ident . read() $($q: tt)?) => {
        let result = Listen($name).await $($q)?;
        let $what = result else {
            if let InMessage::Error(e) = result {
                return Err(CmdError::Remote(String::from_utf8_lossy(&e).to_string()))
            } else {
                return Err(CmdError::InvalidResponse)
            }
        };
    };
    ($name:ident . send($what: expr) $($q: tt)?) => {
        Sender($name, $what).await $($q)?;
    };
}

#[derive(Debug, thiserror::Error)]
pub enum CmdError {
    #[error(transparent)]
    Send(#[from] SendError),
    #[error(transparent)]
    Recv(#[from] RecvError),
    #[error("invalid response received from broker")]
    InvalidResponse,
    #[error("problem encoding message: {0:?}")]
    Envelope(EnvelopeError),
    #[error(transparent)]
    Time(#[from] std::time::SystemTimeError),
    #[error("remote server returned error: {0}")]
    Remote(String),
}

impl From<EnvelopeError> for CmdError {
    fn from(err: EnvelopeError) -> Self {
        Self::Envelope(err)
    }
}

/// Perform a handshake with a broker (that is already opened)
pub async fn handshake(broker: &mut Broker) -> Result<(), CmdError> {
    expect! { broker.send(OutMessage::hello()?)? };
    expect! { let InMessage::Hello(s, r) = broker.read()? };
    broker.reposition(s, r + 1);
    Ok(())
}

/// Retrieve a directory listing from the puckfs path given by `dir`. The directory should
/// end in a forward slash.
pub async fn dir(broker: &mut Broker, dir: &str) -> Result<DirView, CmdError> {
    expect! { broker.send(OutMessage::ReadDir(dir.as_bytes()))? };
    expect! { let InMessage::ReadDir(entries) = broker.read()? };

    Ok(entries)
}

/// Post a message with sender (`from`) and receivers (`to`) indicated by their nicknames.
pub async fn post(
    db: &super::PrincipalsDb<'_>,
    broker: &mut Broker,
    from: &str,
    to: &[&str],
    contents: &str,
) -> Result<(), CmdError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = chrono::Utc::now();

    for (who, data) in to
        .iter()
        .zip(super::envelope(db, from, to, contents, now, broker.rng())?)
    {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        expect! {
            broker.send(OutMessage::WriteFile(
                format!("{who}/{ts}").as_bytes(),
                &data,
            ))?
        };

        expect! { let InMessage::WriteFile = broker.read()? };
    }

    Ok(())
}

/// Retrieve messages from a named inbox (this would typically be a nickname), and
/// delete them upon succesful reception.
pub async fn get(
    db: &super::PrincipalsDb<'_>,
    broker: &mut Broker,
    inbox: &str,
) -> Result<Vec<Result<ValidatedMessage, ValidateError>>, CmdError> {
    let inbox = format!("{inbox}/");

    let entries = dir(broker, &inbox).await?;

    let mut results = Vec::with_capacity(entries.iter().count());

    for entry in entries.iter() {
        //FIXME: we should ideally have a "Path" type
        let name = String::from_utf8_lossy(entry.file_name);
        expect! { broker.send(OutMessage::ReadFile(format!("{inbox}{name}").as_bytes()))? };
        expect! { let InMessage::ReadFile(contents) = broker.read()? };

        let result = super::validate(db, &contents);

        if result.is_ok() {
            delete(broker, &format!("{inbox}{name}")).await?;
        }

        results.push(result);
    }

    Ok(results)
}

/// Delete a file on the remote puckfs broker.
pub async fn delete(broker: &mut Broker, path: &str) -> Result<(), CmdError> {
    expect! { broker.send(OutMessage::Remove(path.as_bytes()))? };
    expect! { let InMessage::Remove = broker.read()? };

    Ok(())
}
