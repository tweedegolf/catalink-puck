use std::io;
use std::net;

use thiserror::Error;

#[allow(unused)]
#[repr(u16)]
enum PacketType {
    Ack,
    Partial,
    Error,
    Bye,
    Hello,
    ReadFile,
    WriteFile,
    Remove,
    ReadDir,
}

#[derive(Error, Debug)]
pub enum BrokerError {
    #[error("could not decode key: {0:?}")]
    KeyDecode(#[from] base64ct::Error),
    #[error("incorrect key length: {0:?}")]
    KeyLength(#[from] std::array::TryFromSliceError),
    #[error("io error: {0:?}")]
    Io(#[from] io::Error),
}

#[derive(Error, Debug)]
enum SendError {
    #[error("crypto error: {0:?}")]
    Crypto(#[from] aead::Error),
    #[error("io error: {0:?}")]
    Io(#[from] io::Error),
    #[error("packet not fully sent: {0} of {1} bytes")]
    SlicedPacket(usize, usize),
    #[error("could not get system time")]
    TimeError(#[from] std::time::SystemTimeError),
}

/* the overhead needed for ad, nonce, ack, cmd, msg and auth tag */
const OVERHEAD: usize = 54;

/* the absolute maximum MTU size on the internet */
const MAX_MTU_SIZE: usize = 1472;

type SecretKey = aead::Key<chacha20poly1305::XChaCha20Poly1305>;
type UdpBuffer = arrayvec::ArrayVec<u8, MAX_MTU_SIZE>;

#[allow(non_snake_case)]
pub struct Broker {
    socket: net::UdpSocket,
    MTU: usize,
    key_id: u32,
    key: SecretKey,
    send_seqno: u32,
    recv_seqno: u32,
}

impl Broker {
    pub fn open(addr: impl net::ToSocketAddrs) -> Result<Self, BrokerError> {
        let socket = net::UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(addr)?;

        use base64ct::{Base64, Encoding};
        let mut key = SecretKey::default();
        let key =
            Base64::decode("36jOMfRiLqvQY+FsLJvVtXYeDEOn3Q2jmUozQooT6LY=", &mut key)?.try_into()?;

        Ok(Broker {
            socket,
            MTU: 1500,
            key_id: 54941102,
            send_seqno: 0,
            recv_seqno: 0,
            key,
        })
    }

    fn serialize<'a>(&self, mut cmd: PacketType, msg: &'a [u8]) -> ([u8; 8], UdpBuffer, &'a [u8]) {
        let mut hdr = [0u8; 8];
        hdr[0..4].copy_from_slice(&self.key_id.to_be_bytes());
        hdr[4..8].copy_from_slice(&self.send_seqno.to_be_bytes());

        let mut payload = UdpBuffer::new();
        payload.extend(self.recv_seqno.to_be_bytes());

        let consumed = std::cmp::min(self.MTU - OVERHEAD, msg.len());

        if consumed < msg.len() {
            cmd = PacketType::Partial;
        }

        payload.extend((cmd as u16).to_be_bytes());
        payload
            .try_extend_from_slice(&msg[..consumed])
            .expect("UdpBuffer to be large enough");

        (hdr, payload, &msg[consumed..])
    }

    fn raw_command<'a>(&self, cmd: PacketType, data: &'a [u8]) -> Result<&'a [u8], SendError> {
        let (ad, mut plaintext, rest) = self.serialize(cmd, data);

        use chacha20poly1305::{
            XChaCha20Poly1305, XNonce,
            aead::{AeadInOut, Generate, KeyInit},
        };

        let cipher = XChaCha20Poly1305::new(&self.key);
        let nonce = XNonce::generate();

        cipher.encrypt_in_place(&nonce, &ad, &mut plaintext)?;
        let ciphertext = plaintext;

        let mut buf = UdpBuffer::new();
        buf.extend(ad);
        buf.extend(nonce);
        buf.extend(ciphertext);

        let sent = self.socket.send(&buf)?;
        if sent < buf.len() {
            return Err(SendError::SlicedPacket(sent, buf.len()));
        }

        Ok(rest)
    }

    fn send_hello(&self) -> Result<(), SendError> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

        let mut buf = core::fmt::NumBuffer::new();
        let millis = millis.format_into(&mut buf);

        let size = millis.len();
        let rest = self.raw_command(PacketType::Hello, millis.as_bytes())?;
        if !rest.is_empty() {
            return Err(SendError::SlicedPacket(size - rest.len(), size));
        }

        Ok(())
    }
}

fn main() {
    let broker = Broker::open("localhost:14400").unwrap();

    broker.send_hello().unwrap();
}
