use std::fmt::Write;

use crate::ui::UiEvent::ChangeScreen;
use crate::ui::screens::UiScreen;
use crate::ui::screens::contacts::Contacts;
use crate::ui::widgets::button_bar::{BarButton, ButtonBar};
use crate::ui::widgets::text_editor::TextEditor;
use crate::ui::widgets::{Drawable, PositionAble, Selectable, Widget};
use crate::ui::{Key, Ui, UiEvent};

use gam::{GlyphStyle, Line, Point, Rectangle, RoundedRectangle, TextBounds, TextView};

const OUTER_MARGIN: isize = 3;
const INNER_MARGIN: isize = 3;
const BUTTON_BAR_HEIGHT: isize = 40;

pub struct Compose {
    button_bar: ButtonBar,
    receivers: Vec<String>,
    text_editor: TextEditor,
    line_height: isize,
}

impl Compose {
    pub fn new(ui: &mut Ui) -> Self {
        let button_bar = ButtonBar::new(
            ui,
            BUTTON_BAR_HEIGHT,
            vec![
                BarButton::new("↩")
                    .style(GlyphStyle::ExtraLarge)
                    .offset(Point { x: 5, y: -2 }),
                BarButton::new("+👤")
                    .style(GlyphStyle::Large)
                    .offset(Point { x: 0, y: -4 }),
                BarButton::new("🗑")
                    .style(GlyphStyle::ExtraLarge)
                    .offset(Point { x: 1, y: -2 }),
                BarButton::new("📨")
                    .style(GlyphStyle::Large)
                    .offset(Point { x: -4, y: -4 }),
            ],
        );

        let receivers = vec!["United States".to_string(), "China".to_string()];
        let line_height =
            Self::compute_line_height(ui, &receivers).expect("Failed to compute line height");

        let text_editor = TextEditor::new(Rectangle::new(
            Point::new(OUTER_MARGIN + INNER_MARGIN, INNER_MARGIN + line_height),
            Point::new(
                ui.screen_size.x - OUTER_MARGIN - INNER_MARGIN,
                ui.screen_size.y - BUTTON_BAR_HEIGHT - OUTER_MARGIN - INNER_MARGIN,
            ),
        ));

        Self {
            button_bar,
            receivers,
            text_editor,
            line_height,
        }
    }

    fn compute_line_height(ui: &mut Ui, receivers: &Vec<String>) -> Result<isize, xous::Error> {
        let mut receivers_text = Self::receivers_textbox(ui, receivers)?;
        ui.bounds_compute_textview(&mut receivers_text)?;
        Ok(receivers_text.bounds_computed.unwrap().br.y)
    }

    fn receivers_textbox(ui: &mut Ui, receivers: &Vec<String>) -> Result<TextView, xous::Error> {
        let mut receivers_text = ui.new_textview(TextBounds::GrowableFromTl(
            Point::new(0 + OUTER_MARGIN, OUTER_MARGIN),
            (ui.screen_size.x - OUTER_MARGIN - INNER_MARGIN) as u16,
        ));
        receivers_text.draw_border = false;
        receivers_text.clear_area = false;
        write!(receivers_text.text, "To: {}", receivers.join(", ")).unwrap();
        Ok(receivers_text)
    }

    fn draw_receivers(&mut self, ui: &mut Ui) -> Result<(), xous::Error> {
        let mut receivers_text = Self::receivers_textbox(ui, &self.receivers)?;
        ui.post_textview(&mut receivers_text)?;
        ui.draw_line(Line::new(
            Point::new(0 + OUTER_MARGIN, self.line_height),
            Point::new(ui.screen_size.x - OUTER_MARGIN, self.line_height),
        ))?;
        Ok(())
    }

    fn draw_border(&mut self, ui: &mut Ui) -> Result<(), xous::Error> {
        let mut rect = RoundedRectangle::new(
            Rectangle::new(
                Point::new(OUTER_MARGIN, OUTER_MARGIN),
                Point::new(
                    ui.screen_size.x - OUTER_MARGIN,
                    ui.screen_size.y - OUTER_MARGIN - BUTTON_BAR_HEIGHT,
                ),
            ),
            5,
        );
        rect.border.style = gam::DrawStyle {
            fill_color: None,
            stroke_color: Some(gam::PixelColor::Dark),
            stroke_width: 2,
        };
        ui.draw_rounded_rectangle(rect)?;
        Ok(())
    }
}

impl UiScreen for Compose {
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error> {
        ui.clear();
        self.draw_border(ui)?;
        self.draw_receivers(ui)?;
        self.button_bar.redraw(ui)?;
        self.text_editor.redraw(ui)?;
        ui.redraw();
        Ok(())
    }

    fn handle_event(&mut self, ui: &mut Ui, event: UiEvent) -> Result<(), xous::Error> {
        match &event {
            UiEvent::Keys(keys) => {
                log::info!("keys: {keys:?}");
                if keys.is_pressed(&Key::Fn1) {
                    let contacts_screen = Box::new(Contacts::new(ui));
                    ui.add_event(ChangeScreen(contacts_screen));
                }
            }
            _ => (),
        }
        self.text_editor.handle_event(ui, &event)?;
        Ok(())
    }
}
