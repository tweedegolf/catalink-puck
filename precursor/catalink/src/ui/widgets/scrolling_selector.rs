use gam::Point;

use crate::ui::{
    Key, Ui,
    UiEvent::{self},
    widgets::{Drawable, PositionAble, Selectable, Widget, WidgetEvent},
};

pub trait Scrollable: Widget + Drawable + Selectable + PositionAble {}
impl<W> Scrollable for W where W: Widget + Drawable + Selectable + PositionAble {}

pub struct ScrollingSelector {
    widgets: Vec<Box<dyn Scrollable>>,
    selected: Option<usize>,
    scroll_offset: isize,
    height: isize,
}

impl ScrollingSelector {
    pub fn new(height: isize) -> Self {
        Self {
            widgets: Vec::new(),
            selected: None,
            scroll_offset: 0,
            height,
        }
    }

    pub fn add<W: Scrollable + 'static>(&mut self, widget: W) {
        self.widgets.push(Box::new(widget));
    }

    pub fn set_selection(&mut self, selection: usize) {
        if self.widgets.is_empty() {
            return;
        }
        let len = self.widgets.len();
        if len >= selection {
            self.selected = Some(len - 1);
        } else {
            self.selected = Some(selection);
        }
    }

    pub fn len(&self) -> usize {
        self.widgets.len()
    }

    fn scroll(&mut self, selected: usize) {
        let mut offset = 0;
        for (index, widget) in self.widgets.iter().enumerate() {
            let height = widget.get_size().y;
            if index == selected {
                if offset < self.scroll_offset {
                    self.scroll_offset = offset;
                } else if offset + height > self.scroll_offset + self.height {
                    self.scroll_offset = offset + height - self.height;
                    if self.scroll_offset < 0 {
                        self.scroll_offset = 0;
                    }
                }
                return;
            } else {
                offset += height;
            }
        }
    }

    pub fn update(&mut self) {
        if let Some(selected) = self.selected {
            self.scroll(selected);
        }
        let mut offset = -self.scroll_offset;
        for (index, widget) in self.widgets.iter_mut().enumerate() {
            widget.set_position(Point::new(0, offset));
            if self.selected == Some(index) {
                widget.set_selected(true);
            } else {
                widget.set_selected(false);
            }
            offset += widget.get_size().y;
        }
    }
}

impl Widget for ScrollingSelector {
    fn handle_event(&mut self, ui: &mut Ui, event: &UiEvent) -> Result<(), xous::Error> {
        let mut change = false;
        match event {
            UiEvent::Keys(keys) => {
                if let Some(selection) = self.selected
                    && keys.is_pressed(&Key::Enter)
                {
                    ui.add_event(UiEvent::WidgetEvent(WidgetEvent::Selected(selection)));
                }
                if keys.is_pressed(&Key::Char('↑')) && !self.widgets.is_empty() {
                    if let Some(selected) = &mut self.selected {
                        if *selected != 0 {
                            *selected -= 1;
                        }
                    } else {
                        self.selected = Some(0);
                    }
                    change = true;
                }
                if keys.is_pressed(&Key::Char('↓')) && !self.widgets.is_empty() {
                    if let Some(selected) = &mut self.selected {
                        *selected += 1;
                        if *selected >= self.widgets.len() {
                            *selected = self.widgets.len() - 1;
                        }
                    } else {
                        self.selected = Some(0);
                    }
                    change = true;
                }
            }
            _ => (),
        }
        println!("Selected: {:?}", self.selected);
        if change {
            self.update();
        }
        Ok(())
    }
}

impl Drawable for ScrollingSelector {
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error> {
        let mut offset = 0;
        for widget in self.widgets.iter_mut() {
            let height = widget.get_size().y;
            let top = offset;
            let bottom = offset + height;
            if !(bottom < self.scroll_offset || top > self.scroll_offset + self.height) {
                widget.redraw(ui)?;
            }
            offset = bottom;
        }
        ui.redraw();
        Ok(())
    }
}
