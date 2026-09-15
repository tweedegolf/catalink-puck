use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

pub mod async_api;

/// A newtype representing the ID of a Principal
#[derive(Debug, Copy, Clone, zeroize::Zeroize, Eq, PartialEq)]
pub struct ID(pub u32);

/// A newtype representing the ID of a symmetric cryptographic key.
#[derive(Debug, Copy, Clone, zeroize::Zeroize, Eq, PartialEq)]
pub struct KeyID(pub u32);

/// A "Principal" as contained in the database.
pub struct Principal {
    id: ID,
    our: Key,
    their: Key,
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
struct Key {
    id: KeyID,
    secret: Vec<u8>,
}

mod db;
use db::PRINCIPALS;

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
    Time(SystemTimeError),
    TooLarge,
    UnknownNick(String),
    Crypto(aead::Error),
    KeyError(KeyID),
}

#[derive(Debug)]
pub enum ValidateError {
    IncompleteData,
    MalformedAuthData,
    SpoofAttempt { actual: ID, pretended: ID },
    Crypto(aead::Error),
    KeyError(KeyID),
}

const VERSION: [u8; 4] = *b"HOT3";

/// Prepare a message for sending over puckfs.
/// This expects: `sender` to contain the nickname of the sender, and `recipients` to
/// be a collection of nicknames of recipients.
pub fn envelope<Iter>(
    sender: &str,
    recipients: Iter,
    content: &str,
) -> Result<Vec<Vec<u8>>, EnvelopeError>
where
    Iter: IntoIterator<Item: AsRef<str>>,
{
    generic_envelope(sender, recipients, content, MsgType::Plain)
}

fn generic_envelope<Iter>(
    sender: &str,
    recipients: Iter,
    content: &str,
    msg_type: MsgType,
) -> Result<Vec<Vec<u8>>, EnvelopeError>
where
    Iter: IntoIterator<Item: AsRef<str>>,
{
    let now: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(EnvelopeError::Time)?
        .as_secs();

    let Some(sender): Option<&Principal> = PRINCIPALS.get(sender) else {
        return Err(EnvelopeError::UnknownNick(sender.to_string()));
    };

    let recipients: Vec<&Principal> = match recipients
        .into_iter()
        .map(|nick| {
            let nick = nick.as_ref();
            PRINCIPALS.get(nick).ok_or_else(|| nick.to_string())
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

        #[cfg(not(target_os = "xous"))]
        use rand::TryRng;
        #[cfg(target_os = "xous")]
        use rand_core::RngCore;

        #[cfg(target_os = "xous")]
        let mut rng = trng::Trng::new(&xous_names::XousNames::new().expect("to get a xous handle"))
            .expect("to have a rng");
        #[cfg(not(target_os = "xous"))]
        let mut rng = rand::rngs::SysRng;

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
pub struct ValidatedMessage {
    pub sender: String,
    pub recipients: Vec<String>,
    pub body: Vec<u8>,
    pub time: SystemTime,
}

/// Validate and parse a received message. If succesful, this return the nickname of the
/// sender, the nicknames of the recipients, and the raw message contents.
pub fn validate(blob: &[u8]) -> Result<ValidatedMessage, ValidateError> {
    let (rcpt_key_id, blob) = blob
        .split_first_chunk::<4>()
        .ok_or(ValidateError::IncompleteData)?;

    let (nonce, msg) = blob
        .split_first_chunk()
        .ok_or(ValidateError::IncompleteData)?;

    let origin_key_id = KeyID(u32::from_be_bytes(*rcpt_key_id));
    let nonce = (*nonce).into();

    let Some(origin) = PRINCIPALS.values().find(|x| x.their.id == origin_key_id) else {
        return Err(ValidateError::KeyError(origin_key_id));
    };

    use chacha20poly1305::{
        KeyInit, XChaCha20Poly1305,
        aead::{Aead, Payload},
    };

    let cipher = XChaCha20Poly1305::new(
        (&*origin.their.secret)
            .try_into()
            .map_err(|_| ValidateError::KeyError(origin.their.id))?,
    );

    let aad = &origin.their.id.0.to_be_bytes();
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

    let time = UNIX_EPOCH + Duration::from_secs(timestamp);
    let sender = ID(u32::from_be_bytes(*sender));

    let true_origin = origin.id;
    if sender != true_origin {
        return Err(ValidateError::SpoofAttempt {
            actual: true_origin,
            pretended: sender,
        });
    }

    //TODO: we can simply plug the correct value in if we stop using HashMap
    let sender = reverse_lookup(true_origin).expect("true_origin to be present");

    let num = u16::from_be_bytes(*num);
    if num > 100 {
        return Err(ValidateError::MalformedAuthData);
    }

    let mut cc_list = Vec::new();
    for _ in 0..num {
        let cc;
        (cc, plaintext) = plaintext
            .split_first_chunk()
            .ok_or(ValidateError::MalformedAuthData)?;
        let cc = u32::from_be_bytes(*cc);

        let nick = reverse_lookup(ID(cc))
            .map(ToOwned::to_owned)
            .unwrap_or(format!("unknown-{cc}"));

        cc_list.push(nick);
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

fn reverse_lookup(id: ID) -> Option<&'static str> {
    PRINCIPALS
        .iter()
        .find(|(_, p)| p.id == id)
        .map(|(nick, _)| *nick)
}
