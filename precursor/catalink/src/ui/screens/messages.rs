use std::fmt::Write;

use crate::data;
use crate::ui::UiEvent::ChangeScreen;
use crate::ui::screens::UiScreen;
use crate::ui::screens::contacts::Contacts;
use crate::ui::screens::select_contacts::SelectContacts;
use crate::ui::screens::syncing::Syncing;
use crate::ui::widgets::button_bar::{BarButton, ButtonBar};
use crate::ui::widgets::scrolling_selector::ScrollingSelector;
use crate::ui::widgets::{Drawable, PositionAble, Selectable, Widget, WidgetEvent};
use crate::ui::{Key, Ui, UiEvent};

use gam::{GlyphStyle, Line, Point, TextBounds, TextView};

const OUTER_MARGIN: isize = 2;
const MESSAGE_MARGIN: isize = 5;
const SEND_REC_MARGIN: isize = 10;
const LINE_HEIGHT: isize = 23;

#[derive(Debug)]
pub struct Message {
    inner: data::Message,
    self_sent: bool,
    size: Point,
    selected: bool,
    position: Point,
    width: isize,
}

impl Message {
    pub fn new(ui: &mut Ui, inner: data::Message) -> Self {
        let width = ui.screen_size.x - OUTER_MARGIN * 2 - SEND_REC_MARGIN;
        let self_sent = inner.inner.sender == ui.system.our_contact().name;
        let mut res = Self {
            inner,
            size: Point { x: width, y: 0 },
            selected: false,
            position: Point { x: 0, y: 0 },
            width,
            self_sent,
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
            self.inner.inner.sender,
            self.inner.body()
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
            self.position + Point::new(self.width, LINE_HEIGHT),
        ))?;

        let mut time_text = ui.new_textview(TextBounds::GrowableFromTr(
            self.position + Point::new(self.width, 3),
            100,
        ));
        time_text.clear_area = false;
        time_text.text = self.inner.time_str();
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
    selector: ScrollingSelector<Message>,
    contact: data::Contact,
    button_bar: ButtonBar,
}

impl Messages {
    pub fn new(ui: &mut Ui, contact: data::Contact) -> Self {
        let mut selector = ScrollingSelector::new(ui.screen_size.y - 40);
        let messages: Vec<_> = ui
            .system
            .get_messages()
            .unwrap()
            .filter_map(|message| message.ok())
            .filter(|message| message.is_relevant_to(&contact))
            .collect();

        for message in messages {
            selector.add(Message::new(ui, message));
        }

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
                    .offset(Point { x: 1, y: -2 }),
                BarButton::new("📝")
                    .style(GlyphStyle::Large)
                    .offset(Point { x: -4, y: -4 }),
            ],
        );

        Self {
            button_bar,
            selector,
            contact,
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
            UiEvent::WidgetEvent(WidgetEvent::Selected(selection)) => {
                let message = self.selector.get(selection).unwrap();
                let us = ui.system.our_contact();
                let recipients = message
                    .inner
                    .relevant_contacts()
                    .filter(|contact| contact != &us)
                    .map(|contact| contact.name)
                    .collect();

                let select_contacts_screen = Box::new(SelectContacts::new(ui, &recipients));
                ui.add_event(UiEvent::ChangeScreen(select_contacts_screen));
            }
            UiEvent::Keys(keys) => {
                if keys.is_pressed(&Key::Fn1) {
                    let contacts_screen = Box::new(Contacts::new(ui));
                    ui.add_event(ChangeScreen(contacts_screen));
                } else if keys.is_pressed(&Key::Fn2) {
                    let syncing_screen = Box::new(Syncing::new(ui));
                    ui.add_event(UiEvent::ChangeScreen(syncing_screen));
                } else if keys.is_pressed(&Key::Fn4) {
                    let select_contacts_screen =
                        Box::new(SelectContacts::new(ui, &vec![self.contact.name.clone()]));
                    ui.add_event(UiEvent::ChangeScreen(select_contacts_screen));
                }
            }
            _ => (),
        }
        Ok(())
    }
}
