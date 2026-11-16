pub mod screens;
pub mod timers;
pub mod widgets;

use std::slice::Iter;

use num_traits::*;
use ux_api::minigfx::*;

use gam::{Gam, Gid, Point};
use widgets::WidgetEvent;
use xous::MessageEnvelope;
use xous_ipc::Buffer;

use crate::{
    sync::{Sync, SyncError, SyncMessages, SystemState, start_sync_thread},
    system::System,
    ui::screens::UiScreen,
};
use timers::{Timer, Timers, start_timer_thread};

#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive, Copy, Clone)]
pub enum RawUiEvent {
    Redraw = 0,
    Line,
    Key,
    Focus,
    Timer,
    LoadState,
    Sync,
    SyncSendSuccesFul,
    AddContactSuccesFul,
    SyncError,
    Quit,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Key {
    Char(char),
    Fn1,
    Fn2,
    Fn3,
    Fn4,
    Enter,
    Unknown(u32),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Keys {
    keys: [Key; 4],
}

impl From<usize> for Key {
    fn from(value: usize) -> Self {
        match value {
            13 => Self::Enter,
            17 => Self::Fn1,
            18 => Self::Fn2,
            19 => Self::Fn3,
            20 => Self::Fn4,
            _ => {
                if let Some(c) = char::from_u32(value as u32) {
                    Self::Char(c)
                } else {
                    Self::Unknown(value as u32)
                }
            }
        }
    }
}

impl Keys {
    pub fn is_pressed(&self, key: &Key) -> bool {
        self.keys.contains(&key)
    }

    pub fn iter(&self) -> Iter<Key> {
        self.keys.iter()
    }
}

pub enum UiEvent {
    Line(String),
    Keys(Keys),
    Redraw,
    ChangeScreen(Box<dyn UiScreen>),
    WidgetEvent(WidgetEvent),
    Timer(u32),
    Sync(SyncEvent),
    Quit,
}

impl std::fmt::Debug for UiEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Line(arg0) => f.debug_tuple("Line").field(arg0).finish(),
            Self::Keys(arg0) => f.debug_tuple("Keys").field(arg0).finish(),
            Self::Redraw => write!(f, "Redraw"),
            Self::ChangeScreen(_) => f.debug_tuple("ChangeScreen").finish(),
            Self::WidgetEvent(arg0) => f.debug_tuple("WidgetEvent").field(arg0).finish(),
            Self::Timer(arg0) => f.debug_tuple("Timer").field(arg0).finish(),
            Self::Sync(arg0) => f.debug_tuple("Sync").field(arg0).finish(),
            Self::Quit => write!(f, "Quit"),
        }
    }
}

#[derive(Debug)]
pub enum SyncEvent {
    LoadState,
    Sync,
    SendSuccesful,
    AddContactSuccesful,
    Error(SyncError),
}

impl UiEvent {
    pub fn from_xous(
        msg: MessageEnvelope,
        state: &mut SystemState,
    ) -> Result<UiEvent, xous::Error> {
        match FromPrimitive::from_usize(msg.body.id()) {
            Some(RawUiEvent::Timer) => {
                if let xous::Message::Scalar(xous::ScalarMessage {
                    id: _, arg1: k1, ..
                }) = msg.body
                {
                    Ok(UiEvent::Timer(k1 as u32))
                } else {
                    Err(xous::Error::ParseError)
                }
            }
            Some(RawUiEvent::Redraw) | Some(RawUiEvent::Focus) => Ok(UiEvent::Redraw),
            Some(RawUiEvent::Line) => {
                let buffer =
                    unsafe { Buffer::from_memory_message(msg.body.memory_message().unwrap()) };
                let s = buffer.as_flat::<String, _>().unwrap().to_string();
                Ok(UiEvent::Line(s))
            }
            Some(RawUiEvent::LoadState) => Ok(UiEvent::Sync(SyncEvent::LoadState)),
            Some(RawUiEvent::Sync) => Ok(UiEvent::Sync(SyncEvent::Sync)),
            Some(RawUiEvent::SyncSendSuccesFul) => Ok(UiEvent::Sync(SyncEvent::SendSuccesful)),
            Some(RawUiEvent::AddContactSuccesFul) => {
                Ok(UiEvent::Sync(SyncEvent::AddContactSuccesful))
            }
            Some(RawUiEvent::SyncError) => Ok(UiEvent::Sync(SyncEvent::Error(
                state.last_error.take().ok_or(xous::Error::NoError)?,
            ))),
            Some(RawUiEvent::Quit) => Ok(UiEvent::Quit),
            Some(RawUiEvent::Key) => {
                if let xous::Message::Scalar(xous::ScalarMessage {
                    id: _,
                    arg1: k1,
                    arg2: k2,
                    arg3: k3,
                    arg4: k4,
                }) = msg.body
                {
                    {
                        let keys = Keys {
                            keys: [Key::from(k1), Key::from(k2), Key::from(k3), Key::from(k4)],
                        };
                        Ok(UiEvent::Keys(keys))
                    }
                } else {
                    Err(xous::Error::ParseError)
                }
            }
            None => Err(xous::Error::ParseError),
        }
    }
}

pub struct Ui {
    pub screen_size: Point,
    gam: Gam,
    pub sid: xous::SID,
    pub content: Gid,
    events: Vec<UiEvent>,
    needs_redraw: bool,
    running: bool,
    timers: Timers,
    sync: Sync,
}

impl Ui {
    pub fn new(xns: &xous_names::XousNames, sid: xous::SID, system: System) -> Self {
        let gam = gam::Gam::new(&xns).expect("Can't connect to GAM");
        let gam_token = gam
            .register_ux(gam::UxRegistration {
                app_name: "CATALINK".to_string(),
                ux_type: gam::UxType::Framebuffer,
                predictor: None,
                listener: sid.to_array(),
                redraw_id: RawUiEvent::Redraw.to_u32().unwrap(),
                gotinput_id: Some(RawUiEvent::Line.to_u32().unwrap()),
                audioframe_id: None,
                rawkeys_id: Some(RawUiEvent::Key.to_u32().unwrap()),
                focuschange_id: Some(RawUiEvent::Focus.to_u32().unwrap()),
            })
            .expect("Could not register GAM UX")
            .unwrap();
        let content = gam
            .request_content_canvas(gam_token)
            .expect("Could not get content canvas");
        let screen_size = gam
            .get_canvas_bounds(content)
            .expect("Could not get canvas dimensions");

        let timers = start_timer_thread(sid, RawUiEvent::Timer.to_u32().unwrap())
            .expect("Could not connect to timer thread");

        let sync = start_sync_thread(
            sid,
            SyncMessages {
                load_state: RawUiEvent::LoadState.to_u32().unwrap(),
                sync: RawUiEvent::Sync.to_u32().unwrap(),
                send_succesful: RawUiEvent::SyncSendSuccesFul.to_u32().unwrap(),
                add_contact_succesful: RawUiEvent::AddContactSuccesFul.to_u32().unwrap(),
                error: RawUiEvent::SyncError.to_u32().unwrap(),
            },
        )
        .expect("Failed to start sync thread");

        Ui {
            screen_size,
            gam,
            content,
            sid,
            events: Vec::new(),
            needs_redraw: false,
            running: true,
            timers,
            sync,
        }
    }

    pub fn clear(&self) {
        self.gam
            .draw_rectangle(
                self.content,
                Rectangle::new_with_style(
                    Point::new(0, 0),
                    self.screen_size,
                    DrawStyle {
                        fill_color: Some(PixelColor::Light),
                        stroke_color: None,
                        stroke_width: 0,
                    },
                ),
            )
            .expect("can't clear content area");
    }

    pub fn add_event(&mut self, event: UiEvent) {
        self.events.push(event);
    }

    pub fn redraw(&mut self) {
        self.needs_redraw;
    }

    pub fn new_textview(&self, bounds: TextBounds) -> TextView {
        TextView::new(self.content, bounds)
    }

    pub fn post_textview(&self, textview: &mut TextView) -> Result<(), xous::Error> {
        self.gam.post_textview(textview)
    }

    pub fn bounds_compute_textview(&self, textview: &mut TextView) -> Result<(), xous::Error> {
        self.gam.bounds_compute_textview(textview)
    }

    pub fn draw_line(&self, line: Line) -> Result<(), xous::Error> {
        self.gam.draw_line(self.content, line)
    }

    pub fn draw_rectangle(&self, rect: Rectangle) -> Result<(), xous::Error> {
        self.gam.draw_rectangle(self.content, rect)
    }

    pub fn draw_rounded_rectangle(&self, rr: RoundedRectangle) -> Result<(), xous::Error> {
        self.gam.draw_rounded_rectangle(self.content, rr)
    }

    pub fn draw_circle(&self, circ: Circle) -> Result<(), xous::Error> {
        self.gam.draw_circle(self.content, circ)
    }

    pub fn raise_menu(&self, menu: &str) -> Result<(), xous::Error> {
        self.gam.raise_menu(menu)
    }

    pub fn start_timer(&mut self, after: std::time::Duration) -> Timer {
        self.timers.start(after)
    }

    pub fn redraw_now(&mut self) -> Result<(), xous::Error> {
        self.gam.redraw()
    }

    fn handle_event(
        &mut self,
        screen: &mut Box<dyn UiScreen>,
        event: UiEvent,
    ) -> Result<(), xous::Error> {
        match event {
            UiEvent::Redraw => {
                self.needs_redraw = true;
            }
            UiEvent::Quit => {
                self.running = false;
            }
            UiEvent::ChangeScreen(new_screen) => {
                *screen = new_screen;
            }
            _ => screen.handle_event(self, event)?,
        }
        Ok(())
    }

    pub fn run(&mut self, mut screen: Box<dyn UiScreen>) {
        while self.running {
            match UiEvent::from_xous(
                xous::receive_message(self.sid).unwrap(),
                &mut *self.sync.state.lock().unwrap(),
            ) {
                Ok(message) => {
                    println!("Received xous event: {message:?}");
                    self.events.push(message);
                }
                Err(err) => {
                    log::error!("Failed to convert xous message: {err:?}")
                }
            }
            while let Some(event) = self.events.pop() {
                self.handle_event(&mut screen, event)
                    .expect("Failed to handle event");
            }
            if self.needs_redraw {
                screen.redraw(self).expect("Failed to redraw screen");
                self.gam.redraw().expect("Failed to redraw");
            }
        }
        log::info!("Shutting down catalink");
    }
}
