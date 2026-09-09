use crate::ui::{Ui, widgets::Drawable};
use blitstr2::glyphstyle::glyph_to_height_hint;
use gam::{DrawStyle, GlyphStyle, PixelColor, Point, Rectangle, TEXTVIEW_DEFAULT_STYLE};

#[derive(Debug)]
pub struct BarButton {
    text: &'static str,
    style: Option<GlyphStyle>,
    offset: Option<Point>,
}

impl BarButton {
    pub fn new(text: &'static str) -> Self {
        Self {
            text,
            style: None,
            offset: None,
        }
    }

    pub fn style(mut self, style: GlyphStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn offset(mut self, offset: Point) -> Self {
        self.offset = Some(offset);
        self
    }
}

#[derive(Debug)]
pub struct ButtonBar {
    size: Point,
    buttons: Vec<BarButton>,
}

impl ButtonBar {
    pub fn new(ui: &mut Ui, height: isize, buttons: Vec<BarButton>) -> Self {
        Self {
            size: Point::new(ui.screen_size.x, height),
            buttons,
        }
    }
}

impl Drawable for ButtonBar {
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error> {
        let width = ui.screen_size.x / self.buttons.len() as isize;
        for (i, button) in self.buttons.iter().enumerate() {
            let offset = width * i as isize - 1;
            let bounds = Rectangle {
                tl: gam::Point {
                    x: offset,
                    y: ui.screen_size.y - self.size.y,
                },
                br: Point {
                    x: width + offset,
                    y: ui.screen_size.y - 1,
                },
                style: DrawStyle {
                    fill_color: Some(PixelColor::Light),
                    stroke_color: Some(PixelColor::Dark),
                    stroke_width: 2,
                },
            };
            ui.draw_rectangle(bounds)?;
            let mut text_bounds = bounds;
            text_bounds.tl.y += (self.size.y
                - glyph_to_height_hint(button.style.unwrap_or(TEXTVIEW_DEFAULT_STYLE)) as isize)
                / 2;

            if let Some(offset) = button.offset {
                text_bounds.tl += offset;
                text_bounds.br += offset;
            }
            let mut textview = ui.new_textview(gam::TextBounds::CenteredTop(text_bounds));
            if let Some(style) = button.style {
                textview.style = style;
            }
            textview.draw_border = false;
            textview.clear_area = false;
            textview.text.push_str(&button.text);
            ui.post_textview(&mut textview)?;
        }
        Ok(())
    }
}
