use gam::Point;

use crate::ui::{
    Key, Ui, UiEvent,
    widgetbox::{WidgetBox, WidgetRef},
    widgets::Widget,
};

pub struct ScrollingSelector {
    step_size: isize,
    widgets: Vec<WidgetRef>,
    selected: Option<usize>,
    scroll_offset: isize,
    vertical_size: isize,
}

impl ScrollingSelector {
    pub fn new(vertical_size: isize) -> Self {
        Self { step_size: 0, widgets: Vec::new(), selected: None, scroll_offset: 0, vertical_size }
    }

    pub fn add(&mut self, widgets: &WidgetBox, widget_ref: WidgetRef) {
        let widget = widgets.get(&widget_ref).unwrap();
        if let Some(size) = widget.get_size() {
            if self.step_size < size.y {
                self.step_size = size.y;
            }
            self.widgets.push(widget_ref);
        }
    }

    fn scroll(&mut self, widgets: &WidgetBox, selected: usize) {
        let mut offset = 0;
        for (index, widget_ref) in self.widgets.iter().enumerate() {
            if let Some(widget) = widgets.get(&widget_ref) &&
                let Some(size) = widget.get_size() {
                    if index == selected {
                        if offset < self.scroll_offset {
                            self.scroll_offset = offset;
                        } else if offset + size.y > self.scroll_offset + self.vertical_size {
                            self.scroll_offset = offset + size.y - self.vertical_size;
                            if self.scroll_offset < 0 {
                                self.scroll_offset = 0;
                            }
                        }
                        return;

                    } else {
                        offset += size.y;
                    }
            }
        }
    }

    pub fn update(&mut self, widgets: &WidgetBox) {
        if let Some(selected) = self.selected {
            self.scroll(widgets, selected);
        }
        let mut offset = -self.scroll_offset;
        for (index, widget_ref) in self.widgets.iter().enumerate() {
            if let Some(mut widget) = widgets.get_mut(&widget_ref) &&
                let Some(size) = widget.get_size() {
                    widget.set_position(Point::new(0, offset));
                    if self.selected == Some(index) {
                        widget.set_selected(true);
                    } else {
                        widget.set_selected(false);
                    }
                    offset += size.y;
                }
            }
    }
}

impl Widget for ScrollingSelector {
    fn redraw(&mut self, _ui: &Ui) -> Result<(), xous::Error> {
        Ok(())
    }

    fn handle_event(&mut self, _ui: &Ui, widgets: &WidgetBox, event: &UiEvent) -> Result<(), xous::Error> {
        let mut change = false;
        match event {
            UiEvent::Line(_) => (),
            UiEvent::Keys(keys) => {
                if keys.is_pressed(&Key::Char('↑')) {
                    if let Some(selected) = &mut self.selected {
                        if *selected != 0 {
                            *selected -= 1;
                        }
                    } else {
                        self.selected = Some(0);
                    }
                    change = true;
                }
                if keys.is_pressed(&Key::Char('↓')) {
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
        }
        println!("Selected: {:?}", self.selected);
        if change {
            self.update(widgets);
        }
        Ok(())
    }
}
