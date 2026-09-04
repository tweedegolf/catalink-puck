use gam::Point;

use crate::ui::{Ui, UiEvent};

pub mod button_bar;
pub mod scrolling_selector;

pub trait Selectable {
    fn set_selected(&mut self, _selected: bool);
}

pub trait PositionAble {
    fn get_size(&self) -> Point;
    fn set_position(&mut self, _position: Point);
}

pub trait Drawable {
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error>;
}

pub trait Widget {
    fn handle_event(&mut self, _ui: &Ui, _event: &UiEvent) -> Result<(), xous::Error> {
        Ok(())
    }
}

pub trait DrawableWidget: Drawable + Widget {}
impl<W> DrawableWidget for W where W: Drawable + Widget {}

impl<C> Widget for C
where
    C: Iterator<Item: Widget>,
{
    fn handle_event(&mut self, ui: &Ui, event: &UiEvent) -> Result<(), xous::Error> {
        for mut widget in self {
            widget.handle_event(ui, event)?;
        }
        Ok(())
    }
}

impl<C> Drawable for C
where
    C: Iterator<Item: Drawable>,
{
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error> {
        for mut widget in self {
            widget.redraw(ui)?;
        }
        Ok(())
    }
}
