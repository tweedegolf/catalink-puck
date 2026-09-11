use std::{io, net, time::Duration};

use thiserror::Error;

mod ringbuf;

#[cfg(not(target_os = "xous"))]
use rand::TryRng;
#[cfg(target_os = "xous")]
use rand_core::RngCore;

macro_rules! debug_println {
    ($($x: expr),*) => {
        #[cfg(debug_assertions)]
        println!($($x),*);
        #[cfg(not(debug_assertions))]
        ()
    }
}

/// This represents a "hello" sent to a puckfs server. Note that a hello message is time-sensitive.
#[derive(Debug, Copy, Clone)]
pub struct HelloMsg(u128);

/// The messages that can be sent to a remote puckfs server.
#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum OutMessage<'a> {
    Bye,
    Error(&'a str),
    Hello(HelloMsg),
    ReadFile(&'a [u8]),
    WriteFile(&'a [u8], &'a [u8]),
    Remove(&'a [u8]),
    ReadDir(&'a [u8]),
}

/// Messages received from a remote puckfs server.
#[derive(Debug, Clone)]
#[repr(u16)]
pub enum InMessage {
    Skip,
    Error(MsgBuffer),
    Bye,
    Hello(u32, u32),
    ReadFile(MsgBuffer),
    WriteFile,
    Remove,
    ReadDir(DirView),
}

/// Represent a directory structure received from the remote.
/// Using `.iter()` gives the directory entries.
#[derive(Debug, Clone)]
pub struct DirView(MsgBuffer);

/// A directory entry from a `DirView` structure.
#[derive(Debug, Clone, Copy)]
pub struct DirEntry<'a> {
    pub file_name: &'a [u8],
    pub time: u128,
    pub size: u128,
}

#[allow(unused)]
#[repr(u16)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum PacketType {
    Ack,
    Partial,
    #[default]
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
pub struct InvalidPacketType(pub u16);

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
pub enum SendError {
    #[error("crypto error: {0:?}")]
    Crypto(#[from] aead::Error),
    #[error("io error: {0:?}")]
    Io(#[from] io::Error),
    #[error("already sending a message, wait")]
    Busy,
    #[error("message is too large")]
    TooLarge,
}

#[derive(Error, Debug)]
pub enum RecvError {
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
    #[error("incomplete message")]
    Incomplete,
    #[error("message is too large")]
    TooLarge(#[from] arrayvec::CapacityError),
}

#[derive(Debug)]
struct Response<'a> {
    cmd: PacketType,
    seq_no: u32,
    ack: u32,
    plaintext: &'a mut [u8],
}

const TMP_KEYID: u32 = 54941102;

/* timeout for reading from a packet */
const READ_TIMEOUT: Duration = Duration::from_millis(1000);

/* timeout for reading from a packet */
const RETRY_TIMEOUT: Duration = Duration::from_millis(100);

/* amount of entries in the ringbuf; determines the maximum message size icw. MTU */
const RINGBUF_ENTRIES: usize = 4; // 1 << 4;

/* the minimum MTU size needed */
const MIN_PACKET_LEN: usize = 8 + 24 + 4 + 2 + 16;

/* the absolute maximum MTU size on the internet */
const MAX_MTU_SIZE: usize = 1472;

/* the absolute maximum message size */
const MAX_MESSAGE_SIZE: usize = RINGBUF_ENTRIES * (MAX_MTU_SIZE - MIN_PACKET_LEN);

type SecretKey = aead::Key<chacha20poly1305::XChaCha20Poly1305>;
type UdpBuffer = arrayvec::ArrayVec<u8, MAX_MTU_SIZE>;
pub type MsgBuffer = arrayvec::ArrayVec<u8, MAX_MESSAGE_SIZE>;

/// An object represents the connection to a remote puckfs broker.
#[allow(non_snake_case)]
pub struct Broker {
    socket: net::UdpSocket,
    MTU: usize,
    key_id: u32,
    key: SecretKey,
    send: ringbuf::Buffer<UdpBuffer, RINGBUF_ENTRIES>,
    recv: ringbuf::Buffer<(PacketType, UdpBuffer), RINGBUF_ENTRIES>,
}

impl Broker {
    /// Open a new connection to a puckfs broker.
    pub fn open(addr: impl net::ToSocketAddrs) -> Result<Self, BrokerError> {
        //TODO: randomize the port
        let local_port = 49152;
        let socket_addr = net::SocketAddr::new(
            net::IpAddr::V4(net::Ipv4Addr::new(0, 0, 0, 0)),
            local_port,
        );

        let socket = net::UdpSocket::bind(socket_addr)?;
        socket.connect(addr)?;
        socket.set_read_timeout(Some(READ_TIMEOUT))?;

        use base64ct::{Base64, Encoding};
        let mut key = SecretKey::default();
        let key =
            Base64::decode("36jOMfRiLqvQY+FsLJvVtXYeDEOn3Q2jmUozQooT6LY=", &mut key)?.try_into()?;

        Ok(Broker {
            socket,
            MTU: 1500,
            key_id: TMP_KEYID,
            send: ringbuf::Buffer::new(0),
            recv: ringbuf::Buffer::new(0),
            key,
        })
    }

    fn serialize<'a>(&self, mut cmd: PacketType, msg: &'a [u8]) -> (UdpBuffer, &'a [u8]) {
        let mut payload = UdpBuffer::new();
        payload.extend(self.recv.count().to_be_bytes());

        let consumed = std::cmp::min(self.MTU - MIN_PACKET_LEN, msg.len());
        if consumed < msg.len() {
            cmd = PacketType::Partial;
        }

        payload.extend((cmd as u16).to_be_bytes());
        payload
            .try_extend_from_slice(&msg[..consumed])
            .expect("UdpBuffer to be large enough");

        (payload, &msg[consumed..])
    }

    fn encode_command(
        &self,
        seqno: u32,
        mut plaintext: UdpBuffer,
    ) -> Result<UdpBuffer, aead::Error> {
        // update the "ack" portion, since it will be out of date
        plaintext[0..4].copy_from_slice(&self.recv.count().to_be_bytes());

        let mut ad = [0u8; 8];
        ad[0..4].copy_from_slice(&self.key_id.to_be_bytes());
        ad[4..8].copy_from_slice(&seqno.to_be_bytes());

        use chacha20poly1305::{
            XChaCha20Poly1305, XNonce,
            aead::{AeadInOut, KeyInit},
        };

        #[cfg(target_os = "xous")]
        let mut rng = trng::Trng::new(&xous_names::XousNames::new().expect("to get a xous handle"))
            .expect("to have a rng");
        #[cfg(not(target_os = "xous"))]
        let mut rng = rand::rngs::SysRng;

        let cipher = XChaCha20Poly1305::new(&self.key);
        let mut nonce = XNonce::default();
        rng.try_fill_bytes(&mut nonce)
            .expect("to have ample random bytes");

        cipher.encrypt_in_place(&nonce, &ad, &mut plaintext)?;
        let ciphertext = plaintext;

        let mut buf = UdpBuffer::new();
        buf.extend(ad);
        buf.extend(nonce);
        buf.extend(ciphertext);

        Ok(buf)
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
        let cmd = u16::from_be_bytes(*cmd).try_into()?;

        Ok(Response {
            cmd,
            seq_no,
            ack,
            plaintext,
        })
    }

    fn send_ack(&self) -> Result<(), aead::Error> {
        let (msg, []) = self.serialize(PacketType::Ack, &[]) else {
            unreachable!("ack packet too big to fit in MTU?");
        };
        let buf = self.encode_command(self.send.count(), msg)?;
        self.send_packet(&buf);

        Ok(())
    }

    fn send_packet(&self, buf: &[u8]) {
        let _sent = self.socket.send(buf);
    }

    fn recv_packet(&mut self) -> Result<UdpBuffer, RecvError> {
        let mut buf = [0u8; MAX_MTU_SIZE];
        let size = self.socket.recv(&mut buf)?;

        let buf = buf[..size]
            .try_into()
            .expect("UdpBuffer to be large enough");

        Ok(buf)
    }

    /// Attempt to send messages that are currently scheduled to be resent, and receive messages
    /// from the remote. This can block for a maximum duration of `READ_TIMEOUT`.
    pub fn perform_io(&mut self) -> Result<(), aead::Error> {
        if self.send.is_stale(RETRY_TIMEOUT) {
            //debug_println!("no ackno, retrying... {}", self.send.r.0);
            for (seq, entry) in self.send.queue() {
                let buf = self.encode_command(seq, entry)?;
                self.send_packet(&buf);
            }
            self.send.mark_sent();
        }

        if let Ok(mut buf) = self.recv_packet()
            && let Ok(response) = self.parse_response(&mut buf)
        {
            if response.ack > self.send.count() {
                debug_println!("got a future ack {}?", response.ack);
            }

            if response.cmd != PacketType::Hello && response.seq_no > self.recv.count() {
                debug_println!("dropped packets detected");
            } else if response.cmd == PacketType::Ack {
                // nothing
            } else if response.seq_no < self.recv.count() {
                //debug_println!("re-ack {:?} {}/{}", std::time::Instant::now(), response.seq_no, self.recv.count());
                self.send_ack()?;
            } else if self
                .recv
                .push((
                    response.cmd,
                    response.plaintext[..]
                        .try_into()
                        .expect("UdpBuffer to be large enough"),
                ))
                .is_ok()
            {
                self.send.mark_clear(response.ack);
                self.send_ack()?;
            }
        }

        Ok(())
    }

    /// Immediately send a message to the remote puckfs broker, and store it in an internal
    /// send buffer to be re-sent if needed.
    pub fn send(&mut self, cmd: OutMessage) -> Result<(), SendError> {
        let (cmd, data) = to_raw_packet(cmd).ok_or(SendError::TooLarge)?;
        let mut data = data.as_ref();

        if !self.send.is_empty() {
            return Err(SendError::Busy);
        }

        if data.len() > (self.MTU - MIN_PACKET_LEN) * RINGBUF_ENTRIES {
            return Err(SendError::TooLarge);
        }

        loop {
            let (msg, rest) = self.serialize(cmd, data);
            self.send.push(msg).expect("message to fit in buffer");
            if rest.is_empty() {
                break;
            } else {
                data = rest;
            }
        }

        for (seq, msg) in self.send.queue() {
            let buf = self.encode_command(seq, msg)?;
            self.send_packet(&buf);
        }

        Ok(())
    }

    /// Attempt to retrieve a message from the internal receive buffer.
    pub fn read(&mut self) -> Result<InMessage, RecvError> {
        let mut buffer = MsgBuffer::new();

        let process = || {
            for (seq, (cmd, msg)) in self.recv.queue() {
                buffer.try_extend_from_slice(&msg)?;
                if cmd != PacketType::Partial {
                    return Ok((seq, (cmd, buffer)));
                }
            }
            Err(RecvError::Incomplete)
        };

        let (seq_no, (cmd, data)) = process()?;
        self.recv.mark_clear(seq_no);

        Ok(from_raw_packet(cmd, data))
    }

    /// Re-synchronize the internal buffers with a "hello" message received from
    /// the remote puckfs broker.
    pub fn reposition(&mut self, send_pos: u32, recv_pos: u32) {
        self.send.reposition(send_pos);
        self.recv.reposition(recv_pos);
    }
}

impl OutMessage<'_> {
    /// Generate a fresh "hello" message.
    pub fn hello() -> Result<OutMessage<'static>, std::time::SystemTimeError> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

        Ok(OutMessage::Hello(HelloMsg(millis)))
    }
}

impl DirView {
    /// Give an iterator of `DirEntry` items listing the entries in a `DirView`.
    pub fn iter(&self) -> impl Iterator<Item = DirEntry<'_>> {
        self.0
            .split(|&x| x == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| {
                let mut components = line.split(|&x| x == b'\0');
                let Some(file_name) = components.next() else {
                    return DirEntry {
                        file_name: line,
                        time: 0,
                        size: 0,
                    };
                };
                let size = components
                    .next()
                    .and_then(|x| std::str::from_utf8(x).ok())
                    .and_then(|x| x.parse().ok())
                    .unwrap_or_default();
                let time = components
                    .next()
                    .and_then(|x| std::str::from_utf8(x).ok())
                    .and_then(|x| x.parse().ok())
                    .unwrap_or_default();

                DirEntry {
                    file_name,
                    time,
                    size,
                }
            })
    }
}

fn parse_hello(msg: &[u8]) -> Option<(u32, u32)> {
    let msg = std::str::from_utf8(msg).ok()?;
    let mut parts = msg.split(' ');

    let a = parts.next()?.parse().ok()?;
    let b = parts.next()?.parse().ok()?;

    Some((a, b))
}

fn from_raw_packet(cmd: PacketType, data: MsgBuffer) -> InMessage {
    match cmd {
        PacketType::Partial | PacketType::Ack => InMessage::Skip,
        PacketType::Error => InMessage::Error(data),
        PacketType::Bye => InMessage::Bye,
        PacketType::Hello => {
            if let Some((x, y)) = parse_hello(&data) {
                InMessage::Hello(x, y)
            } else {
                InMessage::Skip
            }
        }
        PacketType::ReadFile => InMessage::ReadFile(data),
        PacketType::WriteFile => InMessage::WriteFile,
        PacketType::Remove => InMessage::Remove,
        PacketType::ReadDir => InMessage::ReadDir(DirView(data)),
    }
}

#[allow(clippy::large_enum_variant)]
enum CowBuf<'a> {
    Borrowed(&'a [u8]),
    Owned(MsgBuffer),
}

impl<'a> CowBuf<'a> {
    fn as_ref(&'a self) -> &'a [u8] {
        match self {
            Self::Borrowed(x) => x,
            Self::Owned(x) => x,
        }
    }
}

fn to_raw_packet<'a>(cmd: OutMessage<'a>) -> Option<(PacketType, CowBuf<'a>)> {
    let result = match cmd {
        OutMessage::Bye => (PacketType::Bye, CowBuf::Borrowed(&[][..])),
        OutMessage::Hello(HelloMsg(data)) => (PacketType::Hello, {
            use std::io::Write;
            let mut buf = MsgBuffer::new();
            write!(&mut buf, "{data}").ok()?;
            CowBuf::Owned(buf)
        }),
        OutMessage::ReadFile(desc) => (PacketType::ReadFile, CowBuf::Borrowed(desc)),
        OutMessage::WriteFile(desc, data) => {
            let mut buf = MsgBuffer::new();
            buf.try_extend_from_slice(desc).ok()?;
            buf.try_push(0).ok()?;
            buf.try_extend_from_slice(data).ok()?;
            (PacketType::WriteFile, CowBuf::Owned(buf))
        }
        OutMessage::Remove(desc) => (PacketType::Remove, CowBuf::Borrowed(desc)),
        OutMessage::ReadDir(desc) => (PacketType::ReadDir, CowBuf::Borrowed(desc)),
        OutMessage::Error(text) => (PacketType::ReadDir, CowBuf::Borrowed(text.as_bytes())),
    };

    Some(result)
}
