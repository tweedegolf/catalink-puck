use super::{EnvelopeError, ValidateError};
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

#[derive(Debug)]
pub enum CmdError {
    Send(SendError),
    Recv(RecvError),
    InvalidResponse,
    Envelope(EnvelopeError),
    Validate(ValidateError),
    Remote(String),
}

impl From<SendError> for CmdError {
    fn from(e: SendError) -> CmdError {
        CmdError::Send(e)
    }
}

impl From<RecvError> for CmdError {
    fn from(e: RecvError) -> CmdError {
        CmdError::Recv(e)
    }
}

impl From<EnvelopeError> for CmdError {
    fn from(e: EnvelopeError) -> CmdError {
        CmdError::Envelope(e)
    }
}

impl From<ValidateError> for CmdError {
    fn from(e: ValidateError) -> CmdError {
        CmdError::Validate(e)
    }
}

pub async fn handshake(broker: &mut Broker) -> Result<(), CmdError> {
    expect! { broker.send(OutMessage::hello().map_err(SendError::TimeError)?)? };
    expect! { let InMessage::Hello(s, r) = broker.read()? };
    broker.reposition(s, r + 1);
    Ok(())
}

pub async fn dir(broker: &mut Broker, dir: &str) -> Result<DirView, CmdError> {
    expect! { broker.send(OutMessage::ReadDir(dir.as_bytes()))? };
    expect! { let InMessage::ReadDir(entries) = broker.read()? };

    Ok(entries)
}

pub async fn post(
    broker: &mut Broker,
    from: &str,
    to: &[&str],
    contents: &str,
) -> Result<(), CmdError> {
    for (who, data) in to.iter().zip(super::envelope(from, to, contents)?) {
        expect! {
            broker.send(OutMessage::WriteFile(
                format!("{who}/test").as_bytes(),
                &data,
            ))?
        };

        expect! { let InMessage::WriteFile = broker.read()? };
    }

    Ok(())
}

pub async fn get(
    broker: &mut Broker,
    inbox: &str,
) -> Result<Vec<Result<(String, Vec<String>, Vec<u8>), ValidateError>>, CmdError> {
    let inbox = format!("{inbox}/");

    let entries = dir(broker, &inbox).await?;

    let mut results = Vec::with_capacity(entries.iter().count());

    for entry in entries.iter() {
        //FIXME: we should ideally have a "Path" type
        let name = String::from_utf8_lossy(entry.file_name);
        expect! { broker.send(OutMessage::ReadFile(format!("{inbox}{name}").as_bytes()))? };
        expect! { let InMessage::ReadFile(contents) = broker.read()? };

        let result = super::validate(&contents);

        if result.is_ok() {
            delete(broker, &format!("{inbox}{name}")).await?;
        }

        results.push(result);
    }

    Ok(results)
}

pub async fn delete(broker: &mut Broker, path: &str) -> Result<(), CmdError> {
    expect! { broker.send(OutMessage::Remove(path.as_bytes()))? };
    expect! { let InMessage::Remove = broker.read()? };

    Ok(())
}
