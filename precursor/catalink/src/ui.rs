pub mod screens;
pub mod widgets;

use std::slice::Iter;

use num_traits::*;
use ux_api::minigfx::*;

use gam::{Gam, Gid, Point};
use widgets::WidgetEvent;
use xous::MessageEnvelope;
use xous_ipc::Buffer;

use crate::{system::System, ui::screens::UiScreen};

#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive, Copy, Clone)]
enum RawUiEvent {
    Redraw = 0,
    Line,
    Key,
    Focus,
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

#[derive(Debug, PartialEq, Eq)]
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
    Quit,
}

impl UiEvent {
    pub fn from_xous(msg: MessageEnvelope) -> Result<UiEvent, xous::Error> {
        match FromPrimitive::from_usize(msg.body.id()) {
            Some(RawUiEvent::Redraw) | Some(RawUiEvent::Focus) => Ok(UiEvent::Redraw),
            Some(RawUiEvent::Line) => {
                let buffer =
                    unsafe { Buffer::from_memory_message(msg.body.memory_message().unwrap()) };
                let s = buffer.as_flat::<String, _>().unwrap().to_string();
                Ok(UiEvent::Line(s))
            }
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
    pub gam: Gam,
    pub sid: xous::SID,
    pub content: Gid,
    events: Vec<UiEvent>,
    needs_redraw: bool,
    running: bool,
    pub system: System,
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
        Ui {
            screen_size,
            gam,
            content,
            sid,
            events: Vec::new(),
            needs_redraw: false,
            running: true,
            system,
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
            self.events.push(
                UiEvent::from_xous(xous::receive_message(self.sid).unwrap())
                    .expect("Failed to convert xous message"),
            );
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
