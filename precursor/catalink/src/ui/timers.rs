use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, TryRecvError, channel};
use std::thread;
use std::time::{Duration, Instant};

use crate::ui::UiEvent;

enum Command {
    Set(u32, Instant),
    Stop(u32),
}

pub struct Timers {
    sender: Sender<Command>,
    next_id: u32,
}

impl Timers {
    pub fn start(&mut self, after: Duration) -> Timer {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.sender
            .send(Command::Set(id, Instant::now() + after))
            .ok();
        Timer {
            id,
            sender: self.sender.clone(),
        }
    }
}

pub struct Timer {
    id: u32,
    sender: Sender<Command>,
}

impl Timer {
    pub fn fired(&self, event: &UiEvent) -> bool {
        matches!(event, UiEvent::Timer(id) if *id == self.id)
    }

    pub fn stop(self) {
        _ = self.sender.send(Command::Stop(self.id));
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        _ = self.sender.send(Command::Stop(self.id));
    }
}

pub fn start_timer_thread(listener: xous::SID, message: u32) -> Result<Timers, xous::Error> {
    let listener = xous::connect(listener)?;
    let (sender, receiver) = channel();
    thread::spawn(move || run(receiver, listener, message as usize));
    Ok(Timers { sender, next_id: 0 })
}

fn run(receiver: Receiver<Command>, listener: xous::CID, opcode: usize) {
    let mut timers: Vec<(u32, Instant)> = Vec::new();

    loop {
        loop {
            match receiver.try_recv() {
                Ok(cmd) => apply(&mut timers, cmd),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        let now = Instant::now();
        timers.retain(|&(id, when)| {
            if when <= now {
                let msg = xous::Message::new_scalar(opcode, id as usize, 0, 0, 0);
                xous::send_message(listener, msg).ok();
                false
            } else {
                true
            }
        });

        let received = match timers.iter().map(|&(_, when)| when).min() {
            Some(next) => {
                match receiver.recv_timeout(next.saturating_duration_since(Instant::now())) {
                    Err(RecvTimeoutError::Timeout) => continue,
                    other => other.ok(),
                }
            }
            None => receiver.recv().ok(),
        };
        match received {
            Some(cmd) => apply(&mut timers, cmd),
            None => return,
        }
    }
}

fn apply(timers: &mut Vec<(u32, Instant)>, cmd: Command) {
    match cmd {
        Command::Set(id, when) => timers.push((id, when)),
        Command::Stop(id) => timers.retain(|&(t, _)| t != id),
    }
}
