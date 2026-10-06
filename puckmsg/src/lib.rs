use std::time::Duration;

pub mod async_api;

/// A newtype representing the ID of a Principal
#[derive(Debug, Copy, Clone, zeroize::Zeroize, Eq, PartialEq)]
pub struct ID(pub u32);

/// A newtype representing the ID of a symmetric cryptographic key.
#[derive(Debug, Copy, Clone, zeroize::Zeroize, Eq, PartialEq)]
pub struct KeyID(pub u32);

/// A "Principal" as contained in the database.
/// TODO: In  mature code this should be better shielded against
/// programming mistakes (e.g. less easy to get key material out)
pub struct Principal {
    pub id: ID,
    pub our: Key,
    pub their: Key,
}

impl std::fmt::Debug for Principal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Principal {{ id: {:?}, our: <HIDDEN>, their: <HIDDEN> }}",
            self.id
        )
    }
}

#[derive(zeroize::Zeroize)]
pub struct Key {
    pub id: KeyID,
    pub secret: Vec<u8>,
}

pub type PrincipalsDb<'a> = std::collections::HashMap<&'a str, Principal>;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum MsgType {
    Plain,
    Ack1,
    Qack,
}

impl From<MsgType> for [u8; 4] {
    fn from(inp: MsgType) -> [u8; 4] {
        match inp {
            MsgType::Plain => *b"UTF8",
            MsgType::Ack1 => *b"ACK1",
            MsgType::Qack => *b"QACK",
        }
    }
}

impl TryFrom<[u8; 4]> for MsgType {
    type Error = ();
    fn try_from(inp: [u8; 4]) -> Result<MsgType, ()> {
        Ok(match &inp {
            b"UTF8" => MsgType::Plain,
            b"ACK1" => MsgType::Ack1,
            b"QACK" => MsgType::Qack,
            _ => return Err(()),
        })
    }
}

#[derive(Debug)]
pub enum EnvelopeError {
    TooManyRecipients(usize),
    TooLarge,
    UnknownNick(String),
    Crypto(aead::Error),
    KeyError(KeyID),
}

#[derive(Debug)]
pub enum ValidateError {
    IncompleteData,
    MalformedAuthData,
    SpoofAttempt { actual: ID },
    Crypto(aead::Error),
    KeyError(KeyID),
    DuplicateKeysInDatabase(KeyID),
}

const VERSION: [u8; 4] = *b"HOT3";

/// Prepare a message for sending over puckfs.
/// This expects: `sender` to contain the nickname of the sender, and `recipients` to
/// be a collection of nicknames of recipients.
pub fn envelope<Iter>(
    db: &PrincipalsDb,
    sender: &str,
    recipients: Iter,
    content: &str,
    rng: &mut impl rand_core::RngCore,
) -> Result<Vec<Vec<u8>>, EnvelopeError>
where
    Iter: IntoIterator<Item: AsRef<str>>,
{
    generic_envelope(db, sender, recipients, content, MsgType::Plain, rng)
}

fn generic_envelope<Iter>(
    db: &PrincipalsDb,
    sender: &str,
    recipients: Iter,
    content: &str,
    msg_type: MsgType,
    rng: &mut impl rand_core::RngCore,
) -> Result<Vec<Vec<u8>>, EnvelopeError>
where
    Iter: IntoIterator<Item: AsRef<str>>,
{
    let now: u64 = Utc::now().timestamp() as u64;

    let Some(sender): Option<&Principal> = db.get(sender) else {
        return Err(EnvelopeError::UnknownNick(sender.to_string()));
    };

    let recipients: Vec<&Principal> = match recipients
        .into_iter()
        .map(|nick| {
            let nick = nick.as_ref();
            db.get(nick).ok_or_else(|| nick.to_string())
        })
        .collect()
    {
        Ok(ids) => ids,
        Err(unknown) => return Err(EnvelopeError::UnknownNick(unknown.to_string())),
    };

    if recipients.len() > 100 {
        return Err(EnvelopeError::TooManyRecipients(recipients.len()));
    }

    let msg_type: [u8; 4] = msg_type.into();

    let mut buf = Vec::new();
    buf.extend_from_slice(&VERSION);
    buf.extend_from_slice(&msg_type);
    buf.extend_from_slice(&now.to_be_bytes());
    buf.extend_from_slice(&sender.id.0.to_be_bytes());
    let len = recipients.len() as u16;
    buf.extend_from_slice(&len.to_be_bytes());
    for rcpt in recipients.iter() {
        buf.extend_from_slice(&rcpt.id.0.to_be_bytes());
    }
    buf.extend_from_slice(content.as_bytes());

    let ciphertexts = recipients.into_iter().map(move |rcpt| {
        use chacha20poly1305::{
            KeyInit, XChaCha20Poly1305, XNonce,
            aead::{Aead, Payload},
        };

        let cipher = XChaCha20Poly1305::new(
            (&*rcpt.our.secret)
                .try_into()
                .map_err(|_| EnvelopeError::KeyError(rcpt.our.id))?,
        );

        let mut nonce = XNonce::default();
        rng.try_fill_bytes(&mut nonce)
            .expect("to have ample random bytes");

        let aad = &rcpt.our.id.0.to_be_bytes();
        let mut ciphertext = cipher
            .encrypt(&nonce, Payload { msg: &buf, aad })
            .map_err(EnvelopeError::Crypto)?;

        //TODO: this can be improved
        ciphertext.splice(0..0, nonce);
        ciphertext.splice(0..0, *aad);
        Ok::<_, EnvelopeError>(ciphertext)
    });

    ciphertexts.collect()
}

/// A struct representing a valid message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedMessage {
    pub sender: String,
    pub recipients: Vec<String>,
    pub body: Vec<u8>,
    pub time: DateTime<Utc>,
}

/// Validate and parse a received message. If succesful, this return the nickname of the
/// sender, the nicknames of the recipients, and the raw message contents.
pub fn validate(db: &PrincipalsDb, blob: &[u8]) -> Result<ValidatedMessage, ValidateError> {
    let (key_id, blob) = blob
        .split_first_chunk::<4>()
        .ok_or(ValidateError::IncompleteData)?;

    let (nonce, msg) = blob
        .split_first_chunk()
        .ok_or(ValidateError::IncompleteData)?;

    let key_id = KeyID(u32::from_be_bytes(*key_id));
    let nonce = (*nonce).into();

    // Figure out which keys and their principals might be used for this message.
    let mut encrypted_for_parties = db.values().filter_map(|x| {
        if x.their.id == key_id {
            Some((x, &x.their))
        } else if x.our.id == key_id {
            Some((x, &x.our))
        } else {
            None
        }
    });

    // Only one principal should be valid though, so enforce that.
    let Some((encrypted_for, key)) = encrypted_for_parties.next() else {
        return Err(ValidateError::KeyError(key_id));
    };
    if encrypted_for_parties.count() > 0 {
        return Err(ValidateError::DuplicateKeysInDatabase(key_id));
    }

    use chacha20poly1305::{
        KeyInit, XChaCha20Poly1305,
        aead::{Aead, Payload},
    };

    let cipher = XChaCha20Poly1305::new(
        (&*key.secret)
            .try_into()
            .map_err(|_| ValidateError::KeyError(key.id))?,
    );

    let aad = &key.id.0.to_be_bytes();
    let plaintext = cipher
        .decrypt(&nonce, Payload { msg, aad })
        .map_err(ValidateError::Crypto)?;

    let (version, plaintext) = plaintext
        .split_first_chunk()
        .ok_or(ValidateError::MalformedAuthData)?;
    let (msgtype, plaintext) = plaintext
        .split_first_chunk()
        .ok_or(ValidateError::MalformedAuthData)?;
    let (timestamp, plaintext) = plaintext
        .split_first_chunk()
        .ok_or(ValidateError::MalformedAuthData)?;
    let (sender, plaintext) = plaintext
        .split_first_chunk()
        .ok_or(ValidateError::MalformedAuthData)?;
    let (num, mut plaintext) = plaintext
        .split_first_chunk()
        .ok_or(ValidateError::MalformedAuthData)?;

    //TODO: do something with the unused parts
    let _version: [u8; 4] = *version;
    let _msgtype: Option<MsgType> = (*msgtype).try_into().ok();
    let timestamp = u64::from_be_bytes(*timestamp);

    let time = chrono::DateTime::UNIX_EPOCH + Duration::from_secs(timestamp);
    let sender_id = ID(u32::from_be_bytes(*sender));
    let encrypted_by_sender = sender_id == encrypted_for.id;

    //TODO: we can simply plug the correct value in if we stop using HashMap
    let sender = reverse_lookup(db, sender_id).expect("true origin to be present");

    let num = u16::from_be_bytes(*num);
    if num > 100 {
        return Err(ValidateError::MalformedAuthData);
    }

    let mut encrypted_for_cc = false;
    let mut cc_list = Vec::new();
    for _ in 0..num {
        let cc;
        (cc, plaintext) = plaintext
            .split_first_chunk()
            .ok_or(ValidateError::MalformedAuthData)?;
        let cc_id = ID(u32::from_be_bytes(*cc));

        if encrypted_for.id == cc_id {
            encrypted_for_cc = true;
        }

        let nick = reverse_lookup(db, cc_id)
            .map(ToOwned::to_owned)
            .unwrap_or(format!("unknown-{}", cc_id.0));

        cc_list.push(nick);
    }

    // Message was not encrypted+signed for either the sender or any of the receivers of the message.
    if !encrypted_by_sender && !encrypted_for_cc {
        return Err(ValidateError::SpoofAttempt {
            actual: encrypted_for.id,
        });
    }

    //TODO: this can be improved
    let message = ValidatedMessage {
        sender: sender.to_owned(),
        recipients: cc_list,
        body: plaintext.to_vec(),
        time,
    };

    Ok(message)
}

fn reverse_lookup<'a>(db: &PrincipalsDb<'a>, id: ID) -> Option<&'a str> {
    db.iter().find(|(_, p)| p.id == id).map(|(nick, _)| *nick)
}
