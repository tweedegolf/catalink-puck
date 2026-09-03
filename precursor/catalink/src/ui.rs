pub mod screens;
pub mod widgetbox;
pub mod widgets;

use num_traits::*;
use ux_api::minigfx::*;

use gam::bitmap::Bitmap;
use gam::{Gam, Gid, Point};
use xous_ipc::Buffer;

use crate::ui::widgetbox::WidgetBox;

pub trait UiScreen {
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error>;
    fn handle_event(&mut self, ui: &mut Ui, event: UiEvent) -> Result<(), xous::Error>;
}

#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive)]
enum RawUiEvent {
    Redraw = 0,
    Line,
    Key,
    Focus,
    Quit,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Fn1,
    Fn2,
    Fn3,
    Fn4,
    Unknown(u32),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Keys {
    keys: [Key; 4],
}

impl From<usize> for Key {
    fn from(value: usize) -> Self {
        if let Some(c) = char::from_u32(value as u32) {
            Self::Char(c)
        } else {
            match value {
                11 => Self::Fn1,
                12 => Self::Fn2,
                13 => Self::Fn3,
                14 => Self::Fn4,
                _ => Self::Unknown(value as u32),
            }
        }
    }
}

impl Keys {
    pub fn is_pressed(&self, key: &Key) -> bool {
        self.keys.contains(&key)
    }
}

#[derive(Debug)]
pub enum UiEvent {
    Line(String),
    Keys(Keys),
}

pub struct Ui {
    pub screen_size: Point,
    pub gam: Gam,
    pub sid: xous::SID,
    pub content: Gid,
    events: Vec<UiEvent>,
}

impl Ui {
    pub fn new(xns: &xous_names::XousNames, sid: xous::SID) -> Self {
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
        let content = gam.request_content_canvas(gam_token).expect("Could not get content canvas");
        let screen_size = gam.get_canvas_bounds(content).expect("Could not get canvas dimensions");
        Ui { screen_size, gam, content, sid, events: Vec::new() }
    }

    pub fn clear(&self) {
        self.gam
            .draw_rectangle(
                self.content,
                Rectangle::new_with_style(
                    Point::new(0, 0),
                    self.screen_size,
                    DrawStyle { fill_color: Some(PixelColor::Light), stroke_color: None, stroke_width: 0 },
                ),
            )
            .expect("can't clear content area");
    }

    pub fn add_event(&mut self, event: UiEvent) {
        self.events.push(event);
    }

    pub fn redraw(&self) -> Result<(), xous::Error> {
        self.gam.redraw()
    }

    pub fn new_textview(&self, bounds: TextBounds) -> TextView {
        TextView::new(self.content, bounds)
    }

    pub fn post_textview(&self, textview: &mut TextView) -> Result<(), xous::Error> {
        self.gam.post_textview(textview)
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

    pub fn draw_bitmap(&self, bm: &Bitmap) -> Result<(), xous::Error> {
        self.gam.draw_bitmap(self.content, bm)
    }

    pub fn draw_circle(&self, circ: Circle) -> Result<(), xous::Error> {
        self.gam.draw_circle(self.content, circ)
    }

    pub fn run(&mut self, initial: Box<dyn UiScreen>) {
        let mut screen = initial;
        screen.redraw(self).expect("Failed to redraw");
        loop {
            let msg = xous::receive_message(self.sid).unwrap();
            match FromPrimitive::from_usize(msg.body.id()) {
                Some(RawUiEvent::Redraw) => {
                    self.redraw().expect("Failed to redraw");
                }
                Some(RawUiEvent::Line) => {
                    let buffer = unsafe { Buffer::from_memory_message(msg.body.memory_message().unwrap()) };
                    let s = buffer.as_flat::<String, _>().unwrap().to_string();
                    screen.handle_event(self, UiEvent::Line(s)).expect("Failed to handle event");
                }
                Some(RawUiEvent::Quit) => {
                    log::info!("Quitting application");
                    break;
                }
                Some(RawUiEvent::Key) => xous::msg_scalar_unpack!(msg, k1, k2, k3, k4, {
                    let keys = Keys { keys: [Key::from(k1), Key::from(k2), Key::from(k3), Key::from(k4)] };
                    screen.handle_event(self, UiEvent::Keys(keys)).expect("Failed to handle key event");
                }),
                _ => {
                    log::error!("Got unknown message");
                }
            }
        }
    }
}
