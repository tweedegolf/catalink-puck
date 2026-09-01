#![allow(unused)]
use puckfs::{Broker, DirEntry, DirView, InMessage, MsgBuffer, OutMessage, RecvError};

use puckmsg::{envelope, validate};

use std::assert_matches;

fn block_on(broker: &mut Broker) -> InMessage {
    loop {
        match broker.read() {
            Ok(msg) => {
                return msg;
            }
            Err(RecvError::Incomplete) => {
                broker.perform_io().unwrap();
            }
            Err(err) => {
                panic!("unexpected error: {err:?}");
            }
        }
    }
}

macro_rules! expect {
    ($name: ident . recv($what: pat)) => {
        let $what = block_on($name) else {
            panic!("expected {}", stringify!($what));
        };
    };
}

fn handshake(addr: &str) -> Broker {
    let mut new_broker = Broker::open(addr).unwrap();
    let broker = &mut new_broker;

    broker.send(OutMessage::hello().unwrap()).unwrap();
    expect! { broker.recv(InMessage::Hello(s, r)) };
    broker.reposition(s, r + 1);

    new_broker
}

fn dir(broker: &mut Broker, dir: &str) -> DirView {
    broker.send(OutMessage::ReadDir(dir.as_bytes())).unwrap();
    expect! { broker.recv(InMessage::ReadDir(entries)) };

    entries
}

fn post(broker: &mut Broker) {
    let rcpts = ["gopuck"];
    for (who, data) in rcpts
        .iter()
        .zip(puckmsg::envelope("me", rcpts, "Hello world!").unwrap())
    {
        broker
            .send(OutMessage::WriteFile(
                format!("{who}/test").as_bytes(),
                &data,
            ))
            .unwrap();
        expect! { broker.recv(InMessage::WriteFile) };
    }
}

fn get(broker: &mut Broker) {
    broker
        .send(OutMessage::ReadDir("gopuck/".as_bytes()))
        .unwrap();
    expect! { broker.recv(InMessage::ReadDir(dir)) };
    for entry in dir.iter() {
        //FIXME: we should ideally have a "Path" type
        let name = String::from_utf8_lossy(entry.file_name);
        println!("[{name}]");
        broker.send(OutMessage::ReadFile(format!("gopuck/{name}").as_bytes()));
        expect! { broker.recv(InMessage::ReadFile(contents)) };
        let (sender, cc, msg) = validate(&contents).unwrap();
        println!("{sender:?}");
        println!("{cc:?}");
        println!("{}", String::from_utf8_lossy(&msg));
    }
}

fn main() {
    let mut broker = handshake("localhost:14400");

    post(&mut broker);
    get(&mut broker);
}
