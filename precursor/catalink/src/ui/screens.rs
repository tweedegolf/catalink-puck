use crate::ui::{Ui, UiEvent};

pub mod chat;
pub mod contacts;

pub trait UiScreen {
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error>;
    fn handle_event(&mut self, ui: &mut Ui, event: UiEvent) -> Result<(), xous::Error>;
}
