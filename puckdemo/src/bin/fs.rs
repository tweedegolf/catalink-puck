use puckfs::{Broker, DirEntry, InMessage, OutMessage, RecvError};

fn main() {
    use std::collections::VecDeque;
    let mut broker = Broker::open("localhost:14400", rand::thread_rng()).unwrap();

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
                        broker.reposition(s, r + 1);
                    }
                    InMessage::ReadFile(data) => {
                        let data = std::str::from_utf8(&data).unwrap();
                        println!("ReadFile:\n{data}");
                    }
                    InMessage::ReadDir(data) => {
                        println!("ReadDir:");
                        for DirEntry {
                            file_name,
                            time,
                            size,
                        } in data.iter()
                        {
                            let file_name = std::str::from_utf8(file_name).unwrap();
                            println!("\t{file_name}\t{time}\t{size}");
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
