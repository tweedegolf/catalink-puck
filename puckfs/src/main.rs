use puckfs::{Broker, InMessage, OutMessage, RecvError};

fn main() {
    use std::collections::VecDeque;
    let mut broker = Broker::open("localhost:14400").unwrap();

    let mut actions = VecDeque::from([
        OutMessage::hello().unwrap(),
        OutMessage::ReadFile(b"msg"),
        OutMessage::ReadFile(b"world"),
        OutMessage::Remove(b"newmsg"),
        OutMessage::WriteFile(b"newmsg", b"I have a plan!"),
        OutMessage::ReadFile(b"newmsg"),
        OutMessage::ReadDir(b"tmp/"),
    ]);
    let mut greeted = false;
    loop {
        match broker.read() {
            Ok(msg) =>
            {
                #[allow(clippy::single_match)]
                match msg {
                    InMessage::Hello(s, r) => {
                        println!("Hello: {s}  {r}");
                        greeted = true;
                        actions.pop_front();
                        broker.reinit(s, r + 1);
                    }
                    InMessage::ReadFile(data) => {
                        let data = std::str::from_utf8(&data).unwrap();
                        println!("ReadFile:\n{data}");
                    }
                    InMessage::ReadDir(data) => {
                        let data = std::str::from_utf8(&data).unwrap();
                        println!("ReadDir:");
                        for mut set in data.lines().map(|line| line.split('\0')) {
                            let file = set.next().unwrap();
                            let time = set.next().unwrap();
                            let size = set.next().unwrap();
                            println!("\t{file}\t{time}\t{size}");
                        }
                    }
                    _ => println!("{msg:?}"),
                }
            }
            Err(RecvError::Incomplete) => {
                if let Some(action) = actions.front()
                    && broker.send(*action).is_ok()
                    && greeted
                {
                    actions.pop_front();
                }
                broker.perform_io().unwrap();
            }
            _ => panic!(),
        }
    }
}
