use core::fmt::Write;

use log::info;
use num_traits::ToPrimitive;
use ux_api::minigfx::*;

use String;
use gam::*;

use crate::ui::{
    Key, Keys, Ui, UiEvent,
    widgets::{Drawable, Widget},
};

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone)]
pub struct ImefDescriptor {
    pub input_canvas: Option<ux_api::service::api::Gid>,
    pub prediction_canvas: Option<ux_api::service::api::Gid>,
    pub predictor: Option<String>,
    pub token: [u32; 4],
    pub predictor_token: Option<[u32; 4]>,
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ApiToken {
    pub gam_token: [u32; 4],
    pub api_token: [u32; 4],
}

#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum ImefOpcode {
    ConnectBackend,
    RegisterListener,
    ProcessKeys,
    SetMenuMode,
    Redraw,
    Quit,
}
macro_rules! emoji_item {
    ($emoji:expr, $imef_conn:ident) => {
        MenuItem {
            name: String::from(&$emoji.to_string()),
            action_conn: Some($imef_conn),
            action_opcode: ImefOpcode::ProcessKeys.to_u32().unwrap(),
            action_payload: MenuPayload::Scalar([
                $emoji.into(),
                '\u{0000}'.into(),
                '\u{0000}'.into(),
                '\u{0000}'.into(),
            ]),
            close_on_select: true,
        }
    };
}

pub(crate) fn emoji_menu(imef_conn: xous::CID) {
    let _ = std::thread::spawn({
        move || {
            menu_matic(
                vec![
                    emoji_item!('😃', imef_conn),
                    emoji_item!('😒', imef_conn),
                    emoji_item!('🤔', imef_conn),
                    emoji_item!('😅', imef_conn),
                    emoji_item!('🤣', imef_conn),
                    MenuItem {
                        name: String::from("Close Menu"),
                        action_conn: None,
                        action_opcode: 0,
                        action_payload: MenuPayload::Scalar([0, 0, 0, 0]),
                        close_on_select: true,
                    },
                ],
                gam::EMOJI_MENU_NAME,
                None,
            );
        }
    });
}

pub struct TextEditor {
    can_unpick: bool,
    last_trigger_char: Option<usize>,
    line: String,
    characters: usize,
    insertion: usize,
    menu_mode: bool,
    bounding_box: Rectangle,
    #[cfg(feature = "tts")]
    tts: TtsFrontend,
}

impl TextEditor {
    pub fn new(bounding_box: Rectangle) -> TextEditor {
        TextEditor {
            can_unpick: false,
            last_trigger_char: Some(0),
            line: String::new(),
            characters: 0,
            insertion: 0,
            menu_mode: false,
            bounding_box,
        }
    }

    pub fn set_menu_mode(&mut self, mode: bool) {
        self.menu_mode = mode;
    }

    pub fn update(&mut self, newkeys: &Keys, ui: &mut Ui) -> Result<Option<String>, xous::Error> {
        let mut retstring: Option<String> = None;
        info!("updating input area");
        let mut do_redraw = false;
        for k in newkeys.iter() {
            info!("got key '{:?}'", k);
            match k {
                Key::Char('\u{0000}') => (),
                Key::Char('←') => {
                    // move insertion point back
                    if !self.menu_mode {
                        if self.insertion > 0 {
                            log::debug!("moving insertion point back");
                            self.insertion -= 1;
                        }
                        ui.redraw();
                        self.can_unpick = false;
                        self.last_trigger_char = None;
                    } else {
                        return Ok(Some(String::from("←")));
                    }
                }
                Key::Char('→') => {
                    if !self.menu_mode {
                        if self.insertion < self.characters {
                            self.insertion += 1;
                        }
                        ui.redraw();
                        self.can_unpick = false;
                        self.last_trigger_char = None;
                    } else {
                        return Ok(Some(String::from("→")));
                    }
                }
                Key::Char('↑') => {
                    if !self.menu_mode {
                        // bring the insertion point to the front of the text box
                        self.insertion = 0;
                        ui.redraw();
                        self.can_unpick = false;
                        self.last_trigger_char = None;
                    } else {
                        return Ok(Some(String::from("↑")));
                    }
                }
                Key::Char('↓') => {
                    if !self.menu_mode {
                        // bring insertion point to the very end of the text box
                        self.insertion = self.characters;
                        do_redraw = true;
                        self.can_unpick = false;
                        // this means that when we resume typing after an edit, the predictor will set its
                        // insertion point at the very end, not the space
                        // prior to the last word...
                        self.last_trigger_char = Some(self.characters);
                    } else {
                        return Ok(Some(String::from("↓")));
                    }
                }
                Key::Char('\u{0008}') => {
                    // backspace
                    #[cfg(feature = "tts")]
                    self.tts
                        .tts_simple(t!("input.delete-tts", locales::LANG))
                        .unwrap();
                    if (self.characters > 0) && (self.insertion == self.characters) {
                        info!("simple backspace case");
                        self.line.pop();
                        self.characters -= 1;
                        self.insertion -= 1;
                        do_redraw = true;
                    } else if (self.characters > 0) && (self.insertion > 0) {
                        info!("mid-string backspace case");
                        // awful O(N) algo because we have to decode variable-length utf8 strings to
                        // figure out character boundaries first, make a copy
                        // of the string
                        let tempstr = self.line.to_string();
                        // clear the string
                        self.line.clear();

                        // delete the character in the array
                        let mut i = 0; // character position
                        for c in tempstr.chars() {
                            info!("checking index {}", i);
                            if i == self.insertion - 1 {
                                info!("skipping char");
                            } else {
                                info!("copying char {}", c);
                                self.line.push(c);
                            }
                            i += 1;
                        }
                        self.insertion -= 1;
                        self.characters -= 1;
                        do_redraw = true;
                    } else {
                        // ignore, we are either at the front of the string, or the string had no
                        // characters
                    }
                }
                Key::Enter => {
                    // carriage return
                    self.line.push('\n');
                    do_redraw = true;
                    self.insertion += 1;
                    self.characters += 1;
                    log::info!("Carriage return");
                }
                Key::Char(char) => {
                    if self.insertion == self.characters {
                        self.line.push(*char);
                        self.characters += 1;
                        self.insertion += 1;
                        do_redraw = true;
                    } else {
                        info!(
                            "handling case of inserting characters. insertion: {}",
                            self.insertion
                        );
                        // awful O(N) algo because we have to decode variable-length utf8 strings to
                        // figure out character boundaries first, make a copy
                        // of the string
                        let tempstr = self.line.to_string();
                        // clear the string
                        self.line.clear();

                        // insert the character in the array
                        let mut i = 0;
                        for c in tempstr.chars() {
                            info!("checking index {}", i);
                            if i == self.insertion {
                                info!("inserting char {}", char);
                                self.line.push(*char);
                                self.line.push(c);
                            } else {
                                info!("copying char {}", c);
                                self.line.push(c);
                            }
                            i += 1;
                        }
                        self.insertion += 1;
                        self.characters += 1;
                        do_redraw = true;
                    }
                }
                _ => (),
            }

            info!(
                "insertion point is {}, characters in string {}",
                self.insertion, self.characters
            );
        }
        if do_redraw {
            ui.redraw();
        }
        Ok(retstring)
    }

    fn redraw(&mut self, ui: &mut crate::ui::Ui) -> Result<(), xous::Error> {
        let mut input_tv = ui.new_textview(TextBounds::BoundingBox(self.bounding_box));
        input_tv.draw_border = false;
        input_tv.border_width = 1;
        input_tv.clear_area = false;
        input_tv.style = gam::SYSTEM_STYLE;
        input_tv.insertion = Some(self.insertion as _);
        write!(input_tv.text, "{}", self.line.as_str())
            .expect("couldn't update TextView string in input canvas");
        ui.post_textview(&mut input_tv)?;
        Ok(())
    }

    pub fn text(&self) -> &str {
        &self.line
    }
}

impl Drawable for TextEditor {
    fn redraw(&mut self, ui: &mut crate::ui::Ui) -> Result<(), xous::Error> {
        self.redraw(ui)?;
        Ok(())
    }
}

impl Widget for TextEditor {
    fn handle_event(&mut self, ui: &mut crate::ui::Ui, event: &UiEvent) -> Result<(), xous::Error> {
        match event {
            UiEvent::Keys(keys) => {
                log::info!("tracking keys: {:?}", keys);
                if keys.is_pressed(&Key::Char('😊')) {
                    ui.raise_menu(gam::EMOJI_MENU_NAME)?;
                } else {
                    if let Some(line) = self
                        .update(keys, ui)
                        .expect("couldn't update input self with latest key presses")
                    {
                        println!("line: {line}");
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }
}
