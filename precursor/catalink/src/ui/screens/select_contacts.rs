use std::fmt::Write;

use crate::data;
use crate::ui::screens::UiScreen;
use crate::ui::screens::compose::Compose;
use crate::ui::screens::contacts::Contacts;
use crate::ui::widgets::button_bar::{BarButton, ButtonBar};
use crate::ui::widgets::scrolling_selector::ScrollingSelector;
use crate::ui::widgets::{Drawable, PositionAble, Selectable, Widget, WidgetEvent};
use crate::ui::{Key, Ui, UiEvent};

use gam::{DrawStyle, GlyphStyle, PixelColor, Point, Rectangle, RoundedRectangle, TextBounds};

const OUTER_MARGIN: isize = 2;
const INNER_MARGIN: isize = 5;
const HEIGHT: isize = 60;

#[derive(Debug)]
pub struct Contact {
    pub inner: data::Contact,
    width: isize,
    /// Whether the cursor is at the contact.
    selected: bool,
    // Whether the contact was marked to send a message to.
    marked: bool,
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

        if self.marked {
            let mut marked = ui.new_textview(TextBounds::GrowableFromBr(
                self.position
                    + Point::new(
                        ui.screen_size.x - INNER_MARGIN - 5,
                        HEIGHT - INNER_MARGIN - 3,
                    ),
                200,
            ));
            marked.draw_border = true;
            marked.rounded_border = Some(5);
            marked.margin = Point { x: 4, y: 2 };
            marked.style = GlyphStyle::Regular;
            marked.clear_area = false;
            write!(marked.text, "✓").unwrap();

            ui.post_textview(&mut marked)?;
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

pub struct SelectContacts {
    selector: ScrollingSelector<Contact>,
    button_bar: ButtonBar,
}

impl SelectContacts {
    pub fn new(ui: &mut Ui, initial_recipients: &Vec<String>) -> SelectContacts {
        let mut selector = ScrollingSelector::new(ui.screen_size.y - 40);

        const CURSOR_Y_OFFSET: isize = 60;
        let contacts = ui.system.get_contacts().unwrap();

        for (contact_i, contact) in contacts.enumerate() {
            let contact = contact.unwrap();

            let marked = initial_recipients
                .iter()
                .find(|other| *other == &contact.name)
                .is_some();

            selector.add(Contact {
                inner: contact,
                selected: false,
                marked,
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
                BarButton::new("↩")
                    .style(GlyphStyle::ExtraLarge)
                    .offset(Point { x: 5, y: -2 }),
                BarButton::new("📝")
                    .style(GlyphStyle::ExtraLarge)
                    .offset(Point { x: 0, y: 0 }),
            ],
        );

        Self {
            button_bar,
            selector,
        }
    }
}

impl UiScreen for SelectContacts {
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
                let contact = self.selector.get_mut(selection).unwrap();
                contact.marked = !contact.marked;
                log::info!(
                    "Changed selection for {}: {}",
                    contact.inner.name,
                    contact.marked
                );
                ui.add_event(UiEvent::Redraw);
            }
            UiEvent::Keys(keys) => {
                if keys.is_pressed(&Key::Fn1) {
                    let contacts_screen = Box::new(Contacts::new(ui));
                    ui.add_event(UiEvent::ChangeScreen(contacts_screen));
                } else if keys.is_pressed(&Key::Fn2) {
                    let mut contacts = Vec::new();
                    for i in 0..self.selector.len() {
                        let contact = self.selector.get(i).unwrap();
                        if contact.marked {
                            contacts.push(contact.inner.name.clone());
                        }
                    }

                    let syncing_screen = Box::new(Compose::new(ui, contacts));
                    ui.add_event(UiEvent::ChangeScreen(syncing_screen));
                }
            }
            _ => (),
        }
        Ok(())
    }
}
