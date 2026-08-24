use std::io;
use std::net;

use thiserror::Error;

macro_rules! debug_println {
    ($($x: tt),*) => {
        #[cfg(debug_assertions)]
        println!($($x),*);
        #[cfg(not(debug_assertions))]
        ()
    }
}

#[allow(unused)]
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
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
#[error("code '{0}'")]
struct InvalidPacketType(pub u16);

impl TryFrom<u16> for PacketType {
    type Error = InvalidPacketType;
    fn try_from(x: u16) -> Result<Self, InvalidPacketType> {
        let result = match x {
            0 => Self::Ack,
            1 => Self::Partial,
            2 => Self::Error,
            3 => Self::Bye,
            4 => Self::Hello,
            5 => Self::ReadFile,
            6 => Self::WriteFile,
            7 => Self::Remove,
            8 => Self::ReadDir,
            _ => return Err(InvalidPacketType(x)),
        };

        Ok(result)
    }
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

#[derive(Error, Debug)]
enum RecvError {
    #[error("io error: {0:?}")]
    Io(#[from] io::Error),
    #[error("too small packet received of {0} bytes")]
    Underflow(usize),
    #[error("incorrect key id: {0}")]
    KeyError(u32),
    #[error("crypto error: {0:?}")]
    Crypto(#[from] aead::Error),
    #[error("invalid command: {0}")]
    InvalidCmd(#[from] InvalidPacketType),
}

#[derive(Error, Debug)]
enum HandshakeError {
    #[error("could not send hello: {0:?}")]
    Send(#[from] SendError),
    #[error("could not read response: {0:?}")]
    Recv(#[from] RecvError),
}

#[derive(Debug)]
struct Response<'a> {
    cmd: PacketType,
    key_id: u32,
    seq_no: u32,
    ack: u32,
    plaintext: &'a mut [u8],
}

const TMP_KEYID: u32 = 54941102;

/* the overhead needed for ad, nonce, ack, cmd, msg and auth tag */
const OVERHEAD: usize = 54;

/* the absolute maximum MTU size on the internet */
const MIN_PACKET_LEN: usize = 8 + 24 + 4 + 2 + 16;
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
            key_id: TMP_KEYID,
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

    fn encode_command<'a>(
        &self,
        cmd: PacketType,
        data: &'a [u8],
    ) -> Result<(UdpBuffer, &'a [u8]), SendError> {
        let (ad, mut plaintext, rest) = self.serialize(cmd, data);

        debug_println!("sent: PacketType: {:?}, data: {:?}", cmd, data);

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

        Ok((buf, rest))
    }

    fn parse_response<'a>(&self, data: &'a mut UdpBuffer) -> Result<Response<'a>, RecvError> {
        let orig_len = data.len();
        let Some((ad, data)) = data.split_first_chunk_mut::<8>() else {
            return Err(RecvError::Underflow(orig_len));
        };
        let Some((nonce, data)) = data.split_first_chunk_mut() else {
            return Err(RecvError::Underflow(orig_len));
        };

        let ([key_id, seqno], []) = ad.as_chunks() else {
            unreachable!("key_id and seqno are less than 8 bytes?");
        };

        let key_id = u32::from_be_bytes(*key_id);
        let seq_no = u32::from_be_bytes(*seqno);

        if key_id != TMP_KEYID ^ 1 {
            return Err(RecvError::KeyError(key_id));
        }

        let Some((ciphertext, tag)) = data.split_last_chunk_mut() else {
            return Err(RecvError::Underflow(orig_len));
        };

        use chacha20poly1305::{
            XChaCha20Poly1305,
            aead::{AeadInOut, KeyInit},
        };

        let cipher = XChaCha20Poly1305::new(&self.key);
        let nonce = (*nonce).into();
        let tag = (*tag).into();

        cipher.decrypt_inout_detached(&nonce, &ad[..], ciphertext.into(), &tag)?;
        let plaintext = ciphertext;

        let Some((ack, plaintext)) = plaintext.split_first_chunk_mut() else {
            return Err(RecvError::Underflow(orig_len));
        };
        let Some((cmd, plaintext)) = plaintext.split_first_chunk_mut() else {
            return Err(RecvError::Underflow(orig_len));
        };

        let ack = u32::from_be_bytes(*ack);
        let cmd = PacketType::try_from(u16::from_be_bytes(*cmd))?;

        Ok(Response {
            cmd,
            key_id,
            seq_no,
            ack,
            plaintext,
        })
    }

    fn send_hello(&self) -> Result<(), SendError> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

        let mut buf = core::fmt::NumBuffer::new();
        let millis = millis.format_into(&mut buf);

        let size = millis.len();
        let (buf, rest) = self.encode_command(PacketType::Hello, millis.as_bytes())?;

        let sent = self.socket.send(&buf)?;
        if sent < buf.len() {
            return Err(SendError::SlicedPacket(sent, buf.len()));
        }

        if !rest.is_empty() {
            return Err(SendError::SlicedPacket(size - rest.len(), size));
        }

        Ok(())
    }

    fn recv_hello(&mut self) -> Result<UdpBuffer, RecvError> {
        let mut buf = [0u8; MAX_MTU_SIZE];
        let size = self.socket.recv(&mut buf)?;

        if size < MIN_PACKET_LEN {
            return Err(RecvError::Underflow(size));
        }

        let buf = UdpBuffer::from_iter(buf[..size].iter().copied());

        Ok(buf)
    }

    fn handshake(&mut self) -> Result<(), HandshakeError> {
        self.send_hello()?;
        let mut packet = self.recv_hello()?;
        let _response = self.parse_response(&mut packet)?;
        debug_println!("received: {:?}", _response);

        Ok(())
    }
}

fn main() {
    let mut broker = Broker::open("localhost:14400").unwrap();

    broker.handshake().unwrap();
}
