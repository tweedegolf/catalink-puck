#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

mod data;
mod system;
mod ui;

#[cfg(feature = "tts")]
use tts_frontend::*;

use crate::system::System;
use crate::ui::Ui;

pub(crate) const SERVER_NAME_CATALINK: &str = "_CATALINK_";

fn main() -> ! {
    log_server::init_wait().unwrap();
    log::set_max_level(log::LevelFilter::Info);
    log::info!("CATALINK PID is {}", xous::process::id());

    let xns = xous_names::XousNames::new().unwrap();
    let sid = xns
        .register_name(SERVER_NAME_CATALINK, None)
        .expect("can't register server");

    let mut system = System::new();
    let res = system.get_messages().unwrap();

    let messages: Vec<_> = res.collect();
    log::info!("{:?}", messages);

    let mut ui = Ui::new(&xns, sid);

    let starting_screen = Box::new(crate::ui::screens::contacts::Contacts::new(&mut ui));
    ui.run(starting_screen);

    log::info!("Quitting");
    xous::terminate_process(0)
}
