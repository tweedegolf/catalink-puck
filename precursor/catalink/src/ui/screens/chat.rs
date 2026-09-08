use crate::ui::widgets::Drawable;
use crate::ui::widgets::button_bar::{BarButton, ButtonBar};
use crate::ui::{Key, Ui, UiEvent};
use gam::{GlyphStyle, Point, TextBounds};
use std::fmt::Write;

#[derive(Debug)]
pub struct History {
    pub text: String,
    pub is_input: bool,
}

pub struct Chat {
    pub bubble_width: u16,
    pub margin: Point,
    pub bubble_margin: Point,
    pub bubble_radius: u16,
    pub bubble_space: isize,
    pub history: Vec<History>,
    pub scroll_position: isize,
    button_bar: ButtonBar,
}

impl Chat {
    pub fn new(ui: &mut Ui) -> Chat {
        let button_bar = ButtonBar::new(
            ui,
            40,
            vec![
                BarButton::new("↩")
                    .style(GlyphStyle::ExtraLarge)
                    .offset(Point { x: 13, y: -3 }),
                BarButton::new("Test!"),
                BarButton::new("Test"),
                BarButton::new("Test"),
            ],
        );

        Chat {
            bubble_width: ((ui.screen_size.x / 5) * 4) as u16,
            margin: Point::new(4, 45),
            bubble_margin: Point::new(4, 4),
            bubble_radius: 4,
            bubble_space: 4,
            history: vec![
                History {
                    text: "Test".to_string(),
                    is_input: false,
                },
                History {
                    text: "Test".to_string(),
                    is_input: true,
                },
                History {
                    text: "Test".to_string(),
                    is_input: false,
                },
                History {
                    text: "Test".to_string(),
                    is_input: false,
                },
                History {
                    text: "Test".to_string(),
                    is_input: true,
                },
                History {
                    text: "Test".to_string(),
                    is_input: false,
                },
                History {
                    text: "Test".to_string(),
                    is_input: false,
                },
            ],
            scroll_position: 0,
            button_bar,
        }
    }
}

impl crate::ui::UiScreen for Chat {
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error> {
        ui.clear();
        let mut bubble_baseline = ui.screen_size.y - self.margin.y + self.scroll_position;

        log::trace!("drawing chat history");
        for h in self.history.iter().rev() {
            let mut bubble_tv = if h.is_input {
                ui.new_textview(TextBounds::GrowableFromBr(
                    Point::new(ui.screen_size.x - self.margin.x, bubble_baseline),
                    self.bubble_width,
                ))
            } else {
                ui.new_textview(TextBounds::GrowableFromBl(
                    Point::new(self.margin.x, bubble_baseline),
                    self.bubble_width,
                ))
            };
            if h.is_input {
                bubble_tv.border_width = 1;
            } else {
                bubble_tv.border_width = 2;
            }
            bubble_tv.draw_border = true;
            bubble_tv.clear_area = true;
            bubble_tv.rounded_border = Some(self.bubble_radius);
            bubble_tv.style = gam::SYSTEM_STYLE;
            bubble_tv.margin = self.bubble_margin;
            bubble_tv.ellipsis = false;
            bubble_tv.insertion = None;
            write!(bubble_tv.text, "{}", h.text.as_str())
                .expect("couldn't write history text to TextView");
            log::trace!("posting {}", bubble_tv.text);
            ui.post_textview(&mut bubble_tv)
                .expect("couldn't render bubble textview");

            if let Some(bounds) = bubble_tv.bounds_computed {
                println!("bounds.br.y: {}, bounds.br.y: {}", bounds.br.y, bounds.tl.y);
                bubble_baseline -=
                    (bounds.br.y - bounds.tl.y) + self.bubble_space + self.bubble_margin.y;
                if bubble_baseline <= 0 {
                    break;
                }
            } else {
                break;
            }
        }
        self.button_bar.redraw(ui)?;
        ui.redraw();
        Ok(())
    }

    fn handle_event(&mut self, ui: &mut Ui, event: UiEvent) -> Result<(), xous::Error> {
        match event {
            UiEvent::Line(line) => {
                self.history.push(History {
                    text: line,
                    is_input: true,
                });
                self.scroll_position = 0;
                self.redraw(ui)?;
            }
            UiEvent::Keys(keys) => {
                let mut redraw = false;
                if keys.is_pressed(&Key::Char('↑')) {
                    self.scroll_position += 35;
                    redraw = true;
                }
                if keys.is_pressed(&Key::Char('↓')) {
                    self.scroll_position -= 35;
                    if self.scroll_position < 0 {
                        self.scroll_position = 0;
                    }
                    redraw = true;
                }
                if redraw {
                    self.redraw(ui)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
}
