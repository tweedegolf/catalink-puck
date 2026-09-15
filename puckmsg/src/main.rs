use puckfs::{Broker, DirView, InMessage, OutMessage, RecvError};
use puckmsg::ValidatedMessage;

mod blocking {
    use super::*;
    fn recv(broker: &mut Broker) -> InMessage {
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

    fn handshake(addr: &str) -> Broker {
        let mut new_broker = Broker::open(addr).unwrap();
        let broker = &mut new_broker;

        broker.send(OutMessage::hello().unwrap()).unwrap();
        let InMessage::Hello(s, r) = recv(broker) else {
            panic!("unexpected response");
        };
        broker.reposition(s, r + 1);

        new_broker
    }

    fn dir(broker: &mut Broker, dir: &str) -> DirView {
        broker.send(OutMessage::ReadDir(dir.as_bytes())).unwrap();
        let InMessage::ReadDir(entries) = recv(broker) else {
            panic!("unexpected response");
        };

        entries
    }

    fn post(broker: &mut Broker) {
        let rcpts = ["gopuck"];
        for (who, data) in rcpts
            .iter()
            .zip(puckmsg::envelope("eric", rcpts, "Hello world!").unwrap())
        {
            broker
                .send(OutMessage::WriteFile(
                    format!("{who}/test").as_bytes(),
                    &data,
                ))
                .unwrap();
            let InMessage::WriteFile = recv(broker) else {
                panic!("unexpected response")
            };
        }
    }

    fn get(broker: &mut Broker) {
        for entry in dir(broker, "gopuck/").iter() {
            //FIXME: we should ideally have a "Path" type
            let name = String::from_utf8_lossy(entry.file_name);
            println!("[{name}]");
            broker
                .send(OutMessage::ReadFile(format!("gopuck/{name}").as_bytes()))
                .unwrap();

            let InMessage::ReadFile(contents) = recv(broker) else {
                panic!("unexpected response")
            };
            let ValidatedMessage { sender, recipients, body } = puckmsg::validate(&contents).unwrap();
            println!("{sender:?}");
            println!("{recipients:?}");
            println!("{}", String::from_utf8_lossy(&body));
        }
    }

    pub fn demo() {
        let mut broker = handshake("localhost:14400");

        post(&mut broker);
        get(&mut broker);
    }
}

mod asynchronous {
    use puckmsg::{ValidatedMessage, async_api::*};

    pub async fn demo() {
        let mut broker = puckfs::Broker::open("localhost:14400").unwrap();

        handshake(&mut broker).await.unwrap();

        post(&mut broker, "eric", &["gopuck"], "Hello Async!")
            .await
            .unwrap();

        for entry in get(&mut broker, "gopuck").await.unwrap() {
            let ValidatedMessage { sender, recipients, body } = entry.unwrap();
            println!("{sender:?}");
            println!("{recipients:?}");
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
    println!("--async demonstrator--");
    block_on(asynchronous::demo());
}
