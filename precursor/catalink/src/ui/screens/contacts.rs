use std::fmt::Write;

use crate::ui::widgets::button_bar::{BarButton, ButtonBar};
use crate::ui::widgets::scrolling_selector::ScrollingSelector;
use crate::ui::widgets::{Drawable, PositionAble, Selectable, Widget};
use crate::ui::{Ui, UiEvent};

use gam::{DrawStyle, GlyphStyle, PixelColor, Point, Rectangle, RoundedRectangle, TextBounds};

const OUTER_MARGIN: isize = 2;
const INNER_MARGIN: isize = 5;
const HEIGHT: isize = 60;

#[derive(Debug)]
pub struct Contact {
    pub name: String,
    pub last_message: String,
    pub last_message_time: String,
    pub self_sent: bool,
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
            self.position + Point::new(INNER_MARGIN, INNER_MARGIN - 5),
            200,
        ));
        name.draw_border = false;
        name.clear_area = false;
        name.style = GlyphStyle::Large;
        name.text = self.name.clone();
        ui.post_textview(&mut name)?;

        let mut last_message = ui.new_textview(TextBounds::GrowableFromBl(
            self.position + Point::new(INNER_MARGIN, HEIGHT - INNER_MARGIN),
            200,
        ));
        last_message.draw_border = false;
        last_message.style = GlyphStyle::Regular;
        last_message.clear_area = false;
        write!(
            last_message.text,
            "{}{}",
            if self.self_sent { "↖" } else { "↘" },
            self.last_message.clone()
        )
        .unwrap();
        ui.post_textview(&mut last_message)?;

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
        time.text = self.last_message_time.clone();
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
            amount.margin = Point { x: 2, y: 2 };
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
    selector: ScrollingSelector,
    button_bar: ButtonBar,
}

impl Contacts {
    pub fn new(ui: &mut Ui) -> Contacts {
        let mut selector = ScrollingSelector::new(ui.screen_size.y - 40);
        selector.add(Contact {
            name: "United States".to_string(),
            last_message: "Hi".to_string(),
            last_message_time: "09/02/26 00:00".to_string(),
            self_sent: false,
            unread_count: 5,
            selected: false,
            position: Point { x: 0, y: 0 },
            width: ui.screen_size.x,
        });

        selector.add(Contact {
            selected: true,
            name: "China".to_string(),
            position: Point { x: 0, y: 60 },
            last_message: "嗨".to_string(),
            last_message_time: "09/02/26 00:00".to_string(),
            self_sent: true,
            unread_count: 0,
            width: ui.screen_size.x,
        });

        selector.add(Contact {
            selected: false,
            name: "Russia".to_string(),
            position: Point { x: 0, y: 120 },
            last_message: "Привет".to_string(),
            last_message_time: "09/02/26 00:00".to_string(),
            self_sent: false,
            unread_count: 3,
            width: ui.screen_size.x,
        });

        selector.add(Contact {
            selected: false,
            name: "France".to_string(),
            position: Point { x: 0, y: 180 },
            last_message: "Bonjour".to_string(),
            last_message_time: "09/02/26 00:00".to_string(),
            self_sent: false,
            unread_count: 3,
            width: ui.screen_size.x,
        });

        selector.add(Contact {
            selected: false,
            name: "United Kingdom".to_string(),
            position: Point { x: 0, y: 240 },
            last_message: "Hi".to_string(),
            last_message_time: "09/02/26 00:00".to_string(),
            self_sent: false,
            unread_count: 0,
            width: ui.screen_size.x,
        });

        selector.add(Contact {
            selected: false,
            name: "India".to_string(),
            position: Point { x: 0, y: 300 },
            last_message: "नमस्ते".to_string(),
            last_message_time: "09/02/26 00:00".to_string(),
            self_sent: false,
            unread_count: 0,
            width: ui.screen_size.x,
        });

        selector.add(Contact {
            selected: false,
            name: "Jeremy".to_string(),
            position: Point { x: 0, y: 360 },
            last_message: "Hello there".to_string(),
            last_message_time: "09/02/26 00:00".to_string(),
            self_sent: false,
            unread_count: 0,
            width: ui.screen_size.x,
        });

        selector.add(Contact {
            selected: false,
            name: "Susan".to_string(),
            position: Point { x: 0, y: 420 },
            last_message: "Hiii 😻".to_string(),
            last_message_time: "09/02/26 00:00".to_string(),
            self_sent: false,
            unread_count: 0,
            width: ui.screen_size.x,
        });

        selector.add(Contact {
            selected: false,
            name: "Susan2".to_string(),
            position: Point { x: 0, y: 480 },
            last_message: "Hiii! 😻".to_string(),
            last_message_time: "09/02/26 00:00".to_string(),
            self_sent: false,
            unread_count: 0,
            width: ui.screen_size.x,
        });

        selector.add(Contact {
            selected: false,
            name: "Jeremy3".to_string(),
            position: Point { x: 0, y: 540 },
            last_message: "Hello thar".to_string(),
            last_message_time: "09/02/26 00:00".to_string(),
            self_sent: false,
            unread_count: 0,
            width: ui.screen_size.x,
        });
        selector.update();

        let button_bar = ButtonBar::new(
            ui,
            40,
            vec![
                BarButton::new("☰")
                    .style(GlyphStyle::ExtraLarge)
                    .offset(Point { x: 0, y: 7 }),
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

impl crate::ui::UiScreen for Contacts {
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
            _ => (),
        }
        Ok(())
    }
}
