use std::cell::{Ref, RefCell, RefMut};

use crate::ui::widgets::Widget;
use crate::ui::{Ui, UiEvent};

pub struct WidgetBox {
    widgets: Vec<RefCell<Box<dyn Widget>>>,
}

#[derive(Debug)]
pub struct WidgetRef(usize);

impl WidgetBox {
    pub fn new() -> Self {
        Self { widgets: Vec::new() }
    }

    pub fn add<W: Widget + 'static>(&mut self, widget: W) -> WidgetRef {
        let index = self.widgets.len();
        self.widgets.push(RefCell::new(Box::new(widget)));
        WidgetRef(index)
    }

    pub fn redraw(&self, ui: &Ui) -> Result<(), xous::Error> {
        for i in 0..self.widgets.len() {
            if let Some(mut widget) = self.widgets.get(i).map(|thing| thing.borrow_mut()) {
                widget.redraw(ui)?;
            }
        }
        Ok(())
    }

    pub fn handle_event(&self, ui: &Ui, event: &UiEvent) -> Result<(), xous::Error> {
        for i in 0..self.widgets.len() {
            if let Some(mut widget) = self.widgets.get(i).map(|thing| thing.borrow_mut()) {
                widget.handle_event(ui, self, event)?;
            }
        }
        Ok(())
    }

    pub fn get<'b>(&'b self, widget_ref: &WidgetRef) -> Option<Ref<'b, Box<dyn Widget>>> {
        self.widgets.get(widget_ref.0).map(|widget| widget.borrow())
    }

    pub fn get_mut<'b>(&'b self, widget_ref: &WidgetRef) -> Option<RefMut<'b, Box<dyn Widget>>> {
        self.widgets.get(widget_ref.0).map(|widget| widget.borrow_mut())
    }
}
