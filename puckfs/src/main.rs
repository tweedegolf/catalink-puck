//use std::{io, net, time::Duration};

use puckfs::{Broker, PacketType, RecvError};

fn parse_hello(msg: &str) -> Option<(u32, u32)> {
    let mut parts = msg.split(' ');

    let a = parts.next()?.parse().ok()?;
    let b = parts.next()?.parse().ok()?;

    Some((a, b))
}

fn main() {
    use std::collections::VecDeque;
    let mut broker = Broker::open("localhost:14400").unwrap();

    broker.send_hello().unwrap();
    let mut actions = VecDeque::from([
        |x: &mut Broker| x.send(PacketType::ReadFile, b"msg"),
        |x: &mut Broker| x.send(PacketType::ReadFile, b"world"),
        |x: &mut Broker| x.send(PacketType::Remove, b"newmsg"),
        |x: &mut Broker| x.send(PacketType::WriteFile, b"newmsg\x00I have a plan!"),
        |x: &mut Broker| x.send(PacketType::ReadFile, b"newmsg"),
        |x: &mut Broker| x.send(PacketType::ReadDir, b"tmp/"),
    ]);
    let mut greeted = false;
    loop {
        match broker.read() {
            Ok((cmd, data)) => {
                let data = std::str::from_utf8(&data).unwrap();
                #[allow(clippy::single_match)]
                match cmd {
                    PacketType::Hello => {
                        println!("Hello: {data}");
                        greeted = true;
                        let (s, r) = parse_hello(data).unwrap();
                        broker.reinit(s, r + 1);
                    }
                    PacketType::ReadDir => {
                        println!("ReadDir:");
                        for mut set in data.lines().map(|line| line.split('\0')) {
                            let file = set.next().unwrap();
                            let time = set.next().unwrap();
                            let size = set.next().unwrap();
                            println!("\t{file}\t{time}\t{size}");
                        }
                    }
                    _ => println!("{cmd:?}: {}", data),
                }
            }
            Err(RecvError::Incomplete) => {
                broker.perform_io().unwrap();
                if greeted
                    && let Some(action) = actions.front()
                    && action(&mut broker).is_ok()
                {
                    actions.pop_front();
                }
            }
            _ => panic!(),
        }
    }
}
