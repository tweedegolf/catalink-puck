#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]
mod ui;

#[cfg(feature = "tts")]
use tts_frontend::*;

use crate::ui::Ui;

pub(crate) const SERVER_NAME_CATALINK: &str = "_CATALINK_";

fn main() -> ! {
    log_server::init_wait().unwrap();
    log::set_max_level(log::LevelFilter::Info);
    log::info!("CATALINK PID is {}", xous::process::id());

    let xns = xous_names::XousNames::new().unwrap();
    let sid = xns.register_name(SERVER_NAME_CATALINK, None).expect("can't register server");

    let mut ui = Ui::new(&xns, sid);
    ui.run(Box::new(crate::ui::screens::contacts::Contacts::new(ui.screen_size)));

    log::info!("Quitting");
    xous::terminate_process(0)
}
