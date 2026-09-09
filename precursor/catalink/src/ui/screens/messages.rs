use std::fmt::Write;

use crate::ui::UiEvent::ChangeScreen;
use crate::ui::screens::UiScreen;
use crate::ui::screens::contacts::Contacts;
use crate::ui::widgets::button_bar::{BarButton, ButtonBar};
use crate::ui::widgets::scrolling_selector::ScrollingSelector;
use crate::ui::widgets::{Drawable, PositionAble, Selectable, Widget};
use crate::ui::{Key, Ui, UiEvent};

use gam::{GlyphStyle, Line, Point, TextBounds, TextView};

const OUTER_MARGIN: isize = 2;
const MESSAGE_MARGIN: isize = 5;
const SEND_REC_MARGIN: isize = 10;
const LINE_HEIGHT: isize = 23;

#[derive(Debug)]
pub struct Message {
    text: String,
    time: String,
    self_sent: bool,
    sender: String,
    size: Point,
    selected: bool,
    position: Point,
    width: isize,
}

impl Message {
    pub fn new(ui: &mut Ui, sender: String, self_sent: bool, text: String) -> Self {
        let width = ui.screen_size.x - OUTER_MARGIN - SEND_REC_MARGIN;
        let mut res = Self {
            text,
            time: "09/02/26 00:00".to_string(),
            self_sent,
            sender,
            size: Point { x: width, y: 0 },
            selected: false,
            position: Point { x: 0, y: 0 },
            width,
        };

        let mut main_text = res.main_textview(ui);
        ui.bounds_compute_textview(&mut main_text)
            .expect("Failed to compute text bounds");
        let bounds_computed = main_text.bounds_computed.unwrap();
        res.size.y = bounds_computed.br.y - bounds_computed.tl.y + MESSAGE_MARGIN;
        res
    }

    fn main_textview(&mut self, ui: &mut Ui) -> TextView {
        let mut main_text = ui.new_textview(TextBounds::GrowableFromTl(
            self.position + Point::new(OUTER_MARGIN, OUTER_MARGIN),
            self.size.x as u16,
        ));
        main_text.rounded_border = Some(5);
        main_text.clear_area = false;
        if self.selected {
            main_text.border_width = 2;
        }
        write!(
            main_text.text,
            "{}{}\n\n{}",
            if self.self_sent { "↖" } else { "↘" },
            self.sender,
            self.text
        )
        .expect("Failed to write text");
        main_text
    }
}

impl Drawable for Message {
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error> {
        let mut main_text = self.main_textview(ui);
        ui.draw_line(Line::new(
            self.position + Point::new(0 + OUTER_MARGIN, LINE_HEIGHT),
            self.position + Point::new(self.width - 3, LINE_HEIGHT),
        ))?;

        let mut time_text = ui.new_textview(TextBounds::GrowableFromTr(
            self.position + Point::new(self.width, 3),
            100,
        ));
        time_text.clear_area = false;
        time_text.text = self.time.clone();
        time_text.draw_border = false;
        ui.post_textview(&mut time_text)?;

        ui.post_textview(&mut main_text)?;
        Ok(())
    }
}

impl PositionAble for Message {
    fn get_size(&self) -> Point {
        self.size
    }

    fn set_position(&mut self, position: Point) {
        self.position = position;
        if self.self_sent {
            self.position += Point::new(SEND_REC_MARGIN, 0);
        }
    }
}

impl Selectable for Message {
    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}

impl Widget for Message {}

pub struct Messages {
    selector: ScrollingSelector,
    button_bar: ButtonBar,
}

impl Messages {
    pub fn new(ui: &mut Ui) -> Self {
        let mut selector = ScrollingSelector::new(ui.screen_size.y - 40);
        selector.add(Message::new(ui, "United States".to_string(), false, "Lorem ipsum dolor sit amet consectetur adipiscing elit. Quisque faucibus ex sapien vitae pellentesque sem placerat. In id cursus mi pretium tellus duis convallis. Tempus leo eu aenean sed diam urna tempor. Pulvinar vivamus fringilla lacus nec metus bibendum egestas. Iaculis massa nisl malesuada lacinia integer nunc posuere. Ut hendrerit semper vel class aptent taciti sociosqu. Ad litora torquent per conubia nostra inceptos himenaeos.".to_string()));
        selector.add(Message::new(ui, "Me".to_string(),true, "Lorem ipsum dolor sit amet consectetur adipiscing elit. Quisque faucibus ex sapien vitae pellentesque sem placerat. In id cursus mi pretium tellus duis convallis. Tempus leo eu aenean sed diam urna tempor. Pulvinar vivamus fringilla lacus nec metus bibendum egestas. Iaculis massa nisl malesuada lacinia integer nunc posuere. Ut hendrerit semper vel class aptent taciti sociosqu. Ad litora torquent per conubia nostra inceptos himenaeos.".to_string() ));
        selector.add(Message::new(ui, "Me".to_string(),true, "Lorem ipsum dolor sit amet consectetur adipiscing elit. Quisque faucibus ex sapien vitae pellentesque sem placerat. In id cursus mi pretium tellus duis convallis. Tempus leo eu aenean sed diam urna tempor. Pulvinar vivamus fringilla lacus nec metus bibendum egestas. Iaculis massa nisl malesuada lacinia integer nunc posuere. Ut hendrerit semper vel class aptent taciti sociosqu. Ad litora torquent per conubia nostra inceptos himenaeos.".to_string() ));
        selector.add(Message::new(ui, "United States".to_string(), false, "Lorem ipsum dolor sit amet consectetur adipiscing elit. Quisque faucibus ex sapien vitae pellentesque sem placerat. In id cursus mi pretium tellus duis convallis. Tempus leo eu aenean sed diam urna tempor. Pulvinar vivamus fringilla lacus nec metus bibendum egestas. Iaculis massa nisl malesuada lacinia integer nunc posuere. Ut hendrerit semper vel class aptent taciti sociosqu. Ad litora torquent per conubia nostra inceptos himenaeos.".to_string()));
        selector.add(Message::new(ui, "Me".to_string(),true, "Lorem ipsum dolor sit amet consectetur adipiscing elit. Quisque faucibus ex sapien vitae pellentesque sem placerat. In id cursus mi pretium tellus duis convallis. Tempus leo eu aenean sed diam urna tempor. Pulvinar vivamus fringilla lacus nec metus bibendum egestas. Iaculis massa nisl malesuada lacinia integer nunc posuere. Ut hendrerit semper vel class aptent taciti sociosqu. Ad litora torquent per conubia nostra inceptos himenaeos.".to_string() ));
        selector.add(Message::new(ui, "Me".to_string(),true, "Lorem ipsum dolor sit amet consectetur adipiscing elit. Quisque faucibus ex sapien vitae pellentesque sem placerat. In id cursus mi pretium tellus duis convallis. Tempus leo eu aenean sed diam urna tempor. Pulvinar vivamus fringilla lacus nec metus bibendum egestas. Iaculis massa nisl malesuada lacinia integer nunc posuere. Ut hendrerit semper vel class aptent taciti sociosqu. Ad litora torquent per conubia nostra inceptos himenaeos.".to_string() ));
        selector.set_selection(selector.len() - 1);
        selector.update();

        let button_bar = ButtonBar::new(
            ui,
            40,
            vec![
                BarButton::new("↩")
                    .style(GlyphStyle::ExtraLarge)
                    .offset(Point { x: 5, y: -2 }),
                BarButton::new("📥")
                    .style(GlyphStyle::Large)
                    .offset(Point { x: 0, y: -4 }),
                BarButton::new("🗑")
                    .style(GlyphStyle::ExtraLarge)
                    .offset(Point { x: 1, y: 0 }),
                BarButton::new("📝")
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

impl UiScreen for Messages {
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
            UiEvent::Keys(keys) => {
                log::info!("keys: {keys:?}");
                if keys.is_pressed(&Key::Fn1) {
                    let contacts_screen = Box::new(Contacts::new(ui));
                    ui.add_event(ChangeScreen(contacts_screen));
                }
            }
            _ => (),
        }
        Ok(())
    }
}
