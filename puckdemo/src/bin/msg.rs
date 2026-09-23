use puckfs::{Broker, DirView, InMessage, OutMessage, RecvError};
use puckmsg::{PrincipalsDb, ValidatedMessage};

use puckdemo::Rng;

mod blocking {
    use std::time::SystemTime;

    use rand::thread_rng;

    use super::*;
    fn recv(broker: &mut Broker<Rng>) -> InMessage {
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

    fn handshake(addr: &str) -> Broker<Rng> {
        let mut new_broker = Broker::open(addr, thread_rng()).unwrap();
        let broker = &mut new_broker;

        broker.send(OutMessage::hello().unwrap()).unwrap();
        let InMessage::Hello(s, r) = recv(broker) else {
            panic!("unexpected response");
        };
        broker.reposition(s, r + 1);

        new_broker
    }

    fn dir(broker: &mut Broker<Rng>, dir: &str) -> DirView {
        broker.send(OutMessage::ReadDir(dir.as_bytes())).unwrap();
        let InMessage::ReadDir(entries) = recv(broker) else {
            panic!("unexpected response");
        };

        entries
    }

    fn post(db: &PrincipalsDb, broker: &mut Broker<Rng>) {
        let rcpts = ["United Kingdom", "United States"];
        let timestamp = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs();
        for (who, data) in rcpts.iter().zip(
            puckmsg::envelope(db, "Russia", rcpts, "This is test message", broker.nonce()).unwrap(),
        ) {
            broker
                .send(OutMessage::WriteFile(
                    format!("in/{timestamp}-{who}").as_bytes(),
                    &data,
                ))
                .unwrap();
            let InMessage::WriteFile = recv(broker) else {
                panic!("unexpected response")
            };
        }
    }

    fn get(db: &PrincipalsDb, broker: &mut Broker<Rng>) {
        for entry in dir(broker, "in/").iter() {
            //FIXME: we should ideally have a "Path" type
            let name = String::from_utf8_lossy(entry.file_name);
            println!("[{name}]");
            broker
                .send(OutMessage::ReadFile(format!("in/{name}").as_bytes()))
                .unwrap();

            let InMessage::ReadFile(contents) = recv(broker) else {
                panic!("unexpected response")
            };
            let ValidatedMessage {
                sender,
                recipients,
                body,
                time,
            } = puckmsg::validate(db, &contents).unwrap();
            println!("{sender:?}");
            println!("{recipients:?}");
            println!("{time:?}");
            println!("{}", String::from_utf8_lossy(&body));
        }
    }

    pub fn demo() {
        let db = &puckdemo::PRINCIPALS;
        let mut broker = handshake("localhost:14400");

        post(db, &mut broker);
        get(db, &mut broker);
    }
}

mod asynchronous {
    use puckmsg::{ValidatedMessage, async_api::*};

    pub async fn demo() {
        let db = &puckdemo::PRINCIPALS;

        let mut broker = puckfs::Broker::open("localhost:14400", rand::thread_rng()).unwrap();

        handshake(&mut broker).await.unwrap();

        post(db, &mut broker, "eric", &["gopuck"], "Hello Async!")
            .await
            .unwrap();

        for entry in get(db, &mut broker, "gopuck").await.unwrap() {
            let ValidatedMessage {
                sender,
                recipients,
                body,
                time,
            } = entry.unwrap();
            println!("{sender:?}");
            println!("{recipients:?}");
            println!("{time:?}");
            println!("{}", String::from_utf8_lossy(&body));
        }
    }
}

fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    let dummy = std::task::Waker::noop();
    let mut cx = std::task::Context::from_waker(dummy);
    let mut fut = std::pin::pin!(fut);
    loop {
        if let std::task::Poll::Ready(msg) = fut.as_mut().poll(&mut cx) {
            return msg;
        }
    }
}

fn main() {
    println!("--demonstrator--");
    blocking::demo();
    // println!("--async demonstrator--");
    // block_on(asynchronous::demo());
}
