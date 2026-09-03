use gam::Point;

use crate::ui::{Ui, UiEvent, widgetbox::WidgetBox};

pub mod button_bar;
pub mod scrolling_selector;

pub trait Widget {
    fn redraw(&mut self, ui: &Ui) -> Result<(), xous::Error>;
    fn handle_event(&mut self, _ui: &Ui, _widgets: &WidgetBox, _event: &UiEvent) -> Result<(), xous::Error> {
        Ok(())
    }
    fn set_position(&mut self, _position: Point) {}
    fn set_selected(&mut self, _selected: bool) {}
    fn get_size(&self) -> Option<Point> {
        None
    }
    fn get_id(&self) -> Option<usize> {
        Some(0)
    }
}
