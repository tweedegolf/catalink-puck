use crate::ui::{Ui, UiEvent};

pub mod compose;
pub mod contacts;
pub mod messages;
pub mod syncing;

pub trait UiScreen {
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error>;
    fn handle_event(&mut self, ui: &mut Ui, event: UiEvent) -> Result<(), xous::Error>;
}
