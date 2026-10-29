use std::time::Duration;

use gam::{GlyphStyle, Point, Rectangle, TextBounds};

use crate::ui::{
    SyncEvent, Ui, UiEvent,
    timers::Timer,
    widgets::{Drawable, Widget},
};

const ANIM_INTERVAL: Duration = Duration::from_millis(500);
const WIDTH: isize = 500;

pub enum SyncState {
    NotSyncing,
    Syncing,
    Success,
    Failed(String),
}

pub struct SyncPopup {
    state: SyncState,
    timer: Option<Timer>,
    anim_icon: bool,
}

impl SyncPopup {
    pub fn new() -> Self {
        Self {
            state: SyncState::NotSyncing,
            timer: None,
            anim_icon: false,
        }
    }

    pub fn set_syncing(&mut self, ui: &mut Ui) {
        self.state = SyncState::Syncing;
        self.timer = Some(ui.start_timer(ANIM_INTERVAL));
    }

    pub fn interaction_blocked(&self) -> bool {
        match self.state {
            SyncState::NotSyncing => false,
            SyncState::Syncing | SyncState::Success | SyncState::Failed(_) => true,
        }
    }
}

impl Drawable for SyncPopup {
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error> {
        let text = match &self.state {
            SyncState::NotSyncing => return Ok(()),
            SyncState::Syncing => {
                format!("{} Syncing", if self.anim_icon { "✉" } else { "📩" })
            }
            SyncState::Success => "☑ Success!".to_string(),
            SyncState::Failed(err) => err.clone(),
        };
        let center_x = ui.screen_size.x / 2;
        let mut textbox = ui.new_textview(TextBounds::CenteredTop(Rectangle::new(
            Point::new(center_x - WIDTH / 2, 100),
            Point::new(center_x + WIDTH / 2, 150),
        )));
        textbox.draw_border = true;
        textbox.clear_area = true;
        textbox.style = GlyphStyle::Large;
        textbox.text = text;
        textbox.border_width = 2;
        textbox.rounded_border = Some(6);
        ui.post_textview(&mut textbox)?;
        ui.gam.redraw().expect("Failed");
        Ok(())
    }
}

impl Widget for SyncPopup {
    fn handle_event(&mut self, ui: &mut Ui, event: &UiEvent) -> Result<(), xous::Error> {
        match (event, &self.state, &self.timer) {
            (UiEvent::Timer(_), SyncState::Syncing, Some(timer)) => {
                if timer.fired(event) {
                    self.anim_icon = !self.anim_icon;
                    self.redraw(ui)?;
                    self.timer = Some(ui.start_timer(ANIM_INTERVAL));
                }
            }
            (UiEvent::Sync(SyncEvent::Sync), SyncState::Syncing, _) => {
                self.timer.take();
                self.state = SyncState::Success;
            }
            (UiEvent::Sync(SyncEvent::Error(error)), SyncState::Syncing, _) => {
                self.state = SyncState::Failed(format!("⚠ Failed to sync: {error:?}"));
            }
            (UiEvent::Keys(_), SyncState::Success | SyncState::Failed(_), _) => {
                self.state = SyncState::NotSyncing;
                ui.redraw();
            }
            _ => (),
        };
        Ok(())
    }
}
