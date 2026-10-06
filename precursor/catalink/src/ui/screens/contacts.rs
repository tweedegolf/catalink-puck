use std::fmt::Write;

use crate::data;
use crate::ui::screens::UiScreen;
use crate::ui::screens::messages::Messages;
use crate::ui::screens::syncing::Syncing;
use crate::ui::widgets::button_bar::{BarButton, ButtonBar};
use crate::ui::widgets::scrolling_selector::ScrollingSelector;
use crate::ui::widgets::{Drawable, PositionAble, Selectable, Widget, WidgetEvent};
use crate::ui::{Key, Ui, UiEvent};

use gam::{DrawStyle, GlyphStyle, PixelColor, Point, Rectangle, RoundedRectangle, TextBounds};

const OUTER_MARGIN: isize = 2;
const INNER_MARGIN: isize = 5;
const HEIGHT: isize = 60;

const MAX_PREVIEW_BODY_LEN: usize = 30;

#[derive(Debug)]
pub struct Contact {
    pub inner: data::Contact,
    pub last_message: Option<data::Message>,
    pub unread_count: usize,
    width: isize,
    selected: bool,
    position: Point,
}

impl Drawable for Contact {
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error> {
        let rect = RoundedRectangle {
            border: Rectangle {
                tl: Point::new(
                    self.position.x + OUTER_MARGIN,
                    self.position.y + OUTER_MARGIN,
                ),
                br: self.position
                    + Point {
                        x: ui.screen_size.x - OUTER_MARGIN - 1,
                        y: HEIGHT - OUTER_MARGIN,
                    },
                style: DrawStyle {
                    fill_color: None,
                    stroke_color: Some(PixelColor::Dark),
                    stroke_width: if self.selected { 4 } else { 2 },
                },
            },
            radius: 6,
        };
        ui.draw_rounded_rectangle(rect)?;

        let mut name = ui.new_textview(TextBounds::GrowableFromTl(
            self.position + Point::new(INNER_MARGIN + 3, INNER_MARGIN - 5),
            200,
        ));
        name.draw_border = false;
        name.clear_area = false;
        name.style = GlyphStyle::Large;
        name.text = self.inner.name.clone();
        ui.post_textview(&mut name)?;

        let mut last_message = ui.new_textview(TextBounds::GrowableFromBl(
            self.position + Point::new(INNER_MARGIN, HEIGHT - INNER_MARGIN),
            200,
        ));
        last_message.draw_border = false;
        last_message.style = GlyphStyle::Regular;
        last_message.clear_area = false;

        let mut time = ui.new_textview(TextBounds::GrowableFromTr(
            self.position
                + Point::new(
                    self.position.x + ui.screen_size.x - INNER_MARGIN,
                    INNER_MARGIN,
                ),
            (self.position.x - INNER_MARGIN) as u16,
        ));
        time.draw_border = false;
        time.style = GlyphStyle::Regular;
        time.clear_area = false;

        if let Some(_last_message) = &self.last_message {
            let mut body = _last_message.body().to_string();
            // TODO more robust overflow detection?
            if body.len() > MAX_PREVIEW_BODY_LEN {
                body = format!("{}...", &body[0..MAX_PREVIEW_BODY_LEN - 3]);
            }

            write!(
                last_message.text,
                "{}{}",
                if _last_message.inner.sender == ui.system.our_contact().name {
                    "↖"
                } else {
                    "↘"
                },
                body
            )
            .unwrap();
            time.text = _last_message.time_str();
        }

        ui.post_textview(&mut last_message)?;
        ui.post_textview(&mut time)?;

        if self.unread_count > 0 {
            let mut amount = ui.new_textview(TextBounds::GrowableFromBr(
                self.position
                    + Point::new(
                        ui.screen_size.x - INNER_MARGIN - 5,
                        HEIGHT - INNER_MARGIN - 3,
                    ),
                200,
            ));
            amount.draw_border = true;
            amount.rounded_border = Some(5);
            amount.margin = Point { x: 4, y: 2 };
            amount.style = GlyphStyle::Regular;
            amount.clear_area = false;
            write!(amount.text, "{}", self.unread_count).unwrap();
            ui.post_textview(&mut amount)?;
        }
        Ok(())
    }
}

impl PositionAble for Contact {
    fn get_size(&self) -> Point {
        Point::new(self.width, HEIGHT)
    }

    fn set_position(&mut self, position: Point) {
        self.position = position
    }
}

impl Selectable for Contact {
    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}

impl Widget for Contact {}

pub struct Contacts {
    selector: ScrollingSelector<Contact>,
    button_bar: ButtonBar,
}

impl Contacts {
    pub fn new(ui: &mut Ui) -> Contacts {
        let mut selector = ScrollingSelector::new(ui.screen_size.y - 40);

        const CURSOR_Y_OFFSET: isize = 60;
        let contacts = ui.system.derive_contact_state().unwrap();

        for (contact_i, (contact, state)) in contacts.into_iter().enumerate() {
            selector.add(Contact {
                inner: contact,
                last_message: state.last_message,
                unread_count: state.unread_count,
                selected: false,
                position: Point {
                    x: 0,
                    y: contact_i as isize * CURSOR_Y_OFFSET,
                },
                width: ui.screen_size.x,
            });
        }
        selector.update();

        let button_bar = ButtonBar::new(
            ui,
            40,
            vec![
                BarButton::new("📝")
                    .style(GlyphStyle::ExtraLarge)
                    .offset(Point { x: 0, y: 0 }),
                BarButton::new("📥")
                    .style(GlyphStyle::Large)
                    .offset(Point { x: 0, y: -4 }),
                BarButton::new("+👤")
                    .style(GlyphStyle::Large)
                    .offset(Point { x: 0, y: -4 }),
                BarButton::new("🛠")
                    .style(GlyphStyle::Large)
                    .offset(Point { x: -4, y: -4 }),
            ],
        );

        Self {
            button_bar,
            selector,
        }
    }
}

impl UiScreen for Contacts {
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error> {
        ui.clear();
        self.selector.redraw(ui)?;
        self.button_bar.redraw(ui)?;
        ui.redraw();
        Ok(())
    }

    fn handle_event(&mut self, ui: &mut Ui, event: UiEvent) -> Result<(), xous::Error> {
        self.selector.handle_event(ui, &event)?;
        match event {
            UiEvent::WidgetEvent(WidgetEvent::Selected(selection)) => {
                let contact = self.selector.get(selection).unwrap();
                let messages_screen = Box::new(Messages::new(ui, contact.inner.clone()));
                ui.add_event(UiEvent::ChangeScreen(messages_screen));
            }
            UiEvent::Keys(keys) => {
                if keys.is_pressed(&Key::Fn2) {
                    let syncing_screen = Box::new(Syncing::new(ui));
                    ui.add_event(UiEvent::ChangeScreen(syncing_screen));
                }
            }
            _ => (),
        }
        Ok(())
    }
}
