use crate::ui::UiEvent::ChangeScreen;
use crate::ui::{Key, Ui, UiEvent};
use crate::ui::{
    screens::{UiScreen, contacts::Contacts},
    widgets::{
        Drawable,
        button_bar::{BarButton, ButtonBar},
    },
};

use gam::{GlyphStyle, Point, TextBounds};

pub struct Syncing {
    button_bar: ButtonBar,
    message: String,
}

fn sync(system: &mut crate::system::System) -> Result<usize, crate::puck::Error> {
    let mut puck = crate::puck::Puck::new()?;
    let count = puck.sync(system)?;
    Ok(count)
}

impl Syncing {
    pub fn new(ui: &mut Ui) -> Self {
        let button_bar = ButtonBar::new(
            ui,
            40,
            vec![
                BarButton::new("↩")
                    .style(GlyphStyle::ExtraLarge)
                    .offset(Point { x: 5, y: -2 }),
            ],
        );

        let message = match result {
            Ok(count) => format!("Synchronized {count} message(s)"),
            Err(err) => format!("Synchronization failed\n{err}"),
        };

        Self {
            button_bar,
            message,
        }
    }
}

impl UiScreen for Syncing {
    fn redraw(&mut self, ui: &mut Ui) -> Result<(), xous::Error> {
        ui.clear();

        let mut time_text = ui.new_textview(TextBounds::GrowableFromTl(Point { x: 0, y: 0 }, 300));
        time_text.clear_area = false;
        time_text.text = self.message.clone();
        time_text.draw_border = false;
        ui.post_textview(&mut time_text)?;

        self.button_bar.redraw(ui)?;
        ui.redraw();
        Ok(())
    }

    fn handle_event(&mut self, ui: &mut Ui, event: UiEvent) -> Result<(), xous::Error> {
        match event {
            UiEvent::Keys(keys) => {
                if keys.is_pressed(&Key::Fn1) {
                    let contacts_screen = Box::new(Contacts::new(ui));
                    ui.add_event(ChangeScreen(contacts_screen));
                }
            }
            _ => (),
        }
        Ok(())
    }
}
