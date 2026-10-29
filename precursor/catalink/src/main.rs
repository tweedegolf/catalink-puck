#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

mod data;
mod puck;
mod sync;
mod system;
mod ui;

#[cfg(feature = "tts")]
use tts_frontend::*;

use crate::system::System;
use crate::ui::Ui;

pub(crate) const SERVER_NAME_CATALINK: &str = "_CATALINK_";

fn main() -> ! {
    const HEAP_LARGER_LIMIT: usize = 2048 * 1024;
    let new_limit = HEAP_LARGER_LIMIT;
    let result = xous::rsyscall(xous::SysCall::AdjustProcessLimit(
        xous::Limits::HeapMaximum as usize,
        0,
        new_limit,
    ));

    if let Ok(xous::Result::Scalar2(1, current_limit)) = result {
        xous::rsyscall(xous::SysCall::AdjustProcessLimit(
            xous::Limits::HeapMaximum as usize,
            current_limit,
            new_limit,
        ))
        .unwrap();
        log::info!("Heap limit increased to: {}", new_limit);
    } else {
        panic!("Unsupported syscall!");
    }

    let stack_size = 1024 * 1024;
    std::thread::Builder::new()
        .stack_size(stack_size)
        .spawn(wrapped_main)
        .unwrap()
        .join()
        .unwrap()
}

fn wrapped_main() -> ! {
    log_server::init_wait().unwrap();
    log::set_max_level(log::LevelFilter::Info);
    log::info!("CATALINK PID is {}", xous::process::id());

    let xns = xous_names::XousNames::new().unwrap();
    let sid = xns
        .register_name(SERVER_NAME_CATALINK, None)
        .expect("can't register server");

    let mut ui = Ui::new(&xns, sid, System::new());

    let starting_screen = Box::new(crate::ui::screens::contacts::Contacts::new(&mut ui));
    ui.run(starting_screen);

    log::info!("Quitting");
    xous::terminate_process(0)
}
