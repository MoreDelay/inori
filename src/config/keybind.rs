use crate::event_handler::Result;
use crate::update::Message::{self, *};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

pub enum KeybindTarget {
    Map(KeybindMap),
    Msg(Message),
}
use KeybindTarget::*;

pub struct KeybindMap(pub HashMap<KeyEvent, KeybindTarget>);

#[cfg(all(feature = "dvorak_movement_keys", feature = "qwerty_movement_keys"))]
compile_error!("Cannot have both default dvorak and qwerty movement keys");

impl KeybindMap {
    pub fn default() -> Self {
        let mut keybindings = HashMap::new();
        let empty = KeyModifiers::empty();
        #[cfg(feature = "dvorak_movement_keys")]
        {
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('t'), empty),
                Msg(Direction(crate::config::Dirs::Vert(
                    crate::config::Vertical::Up,
                ))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('h'), empty),
                Msg(Direction(crate::config::Dirs::Vert(
                    crate::config::Vertical::Down,
                ))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('d'), empty),
                Msg(Direction(crate::config::Dirs::Horiz(
                    crate::config::Horizontal::Left,
                ))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('n'), empty),
                Msg(Direction(crate::config::Dirs::Horiz(
                    crate::config::Horizontal::Right,
                ))),
            );
        }
        #[cfg(feature = "qwerty_movement_keys")]
        {
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('k'), empty),
                Msg(Direction(crate::config::Dirs::Vert(
                    crate::config::Vertical::Up,
                ))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('j'), empty),
                Msg(Direction(crate::config::Dirs::Vert(
                    crate::config::Vertical::Down,
                ))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('h'), empty),
                Msg(Direction(crate::config::Dirs::Horiz(
                    crate::config::Horizontal::Left,
                ))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('l'), empty),
                Msg(Direction(crate::config::Dirs::Horiz(
                    crate::config::Horizontal::Right,
                ))),
            );
        }
        keybindings
            .insert(KeyEvent::new(KeyCode::Char('p'), empty), Msg(PlayPause));
        keybindings.insert(KeyEvent::new(KeyCode::Enter, empty), Msg(Select));
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('1'), empty),
            Msg(SwitchScreen(super::Screen::Library)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('2'), empty),
            Msg(SwitchScreen(super::Screen::Queue)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('q'), empty),
            Msg(SwitchState(super::State::Done)),
        );
        keybindings
            .insert(KeyEvent::new(KeyCode::Backspace, empty), Msg(Delete));
        keybindings
            .insert(KeyEvent::new(KeyCode::Tab, empty), Msg(ToggleScreen));
        keybindings.insert(KeyEvent::new(KeyCode::Char(' '), empty), Msg(Fold));
        keybindings
            .insert(KeyEvent::new(KeyCode::Char('-'), empty), Msg(Clear));
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('/'), empty),
            Msg(LocalSearch(super::SearchMsg::Start)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('g'), empty),
            Msg(GlobalSearch(super::SearchMsg::Start)),
        );
        keybindings.insert(KeyEvent::new(KeyCode::Esc, empty), Msg(Escape));
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('r'), empty),
            Msg(Set(super::Toggle::Repeat)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('z'), empty),
            Msg(Set(super::Toggle::Random)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('s'), empty),
            Msg(Set(super::Toggle::Single)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('c'), empty),
            Msg(Set(super::Toggle::Consume)),
        );

        Self(keybindings)
    }
    pub fn insert(&mut self, msg: Message, bind: &[KeyEvent]) {
        if bind.len() == 1 {
            self.0.insert(bind[0], KeybindTarget::Msg(msg));
            return;
        }
        if self.0.contains_key(&bind[0]) {
            match self.0.get_mut(&bind[0]).unwrap() {
                KeybindTarget::Map(m) => {
                    return m.insert(msg, &bind[1..]);
                }
                KeybindTarget::Msg(_m) => panic!("keybind shadowed"),
            }
        } else {
            self.0.insert(
                bind[0],
                KeybindTarget::Map(KeybindMap(HashMap::new())),
            );
            self.insert(msg, bind)
        }
    }
    pub fn lookup(&self, bind: &[KeyEvent]) -> Option<&KeybindTarget> {
        if bind.len() == 0 {
            return None;
        }
        if bind.len() == 1 {
            return self.0.get(&bind[0]);
        }
        match self.0.get(&bind[0]) {
            Some(KeybindTarget::Map(m)) => m.lookup(&bind[1..]),
            None => None,
            o => o,
        }
    }
}

pub fn parse_keybind_single(s: &str) -> Option<KeyCode> {
    if s.len() == 1 {
        if let Some(c) = s.chars().nth(0) {
            Some(KeyCode::Char(c))
        } else {
            None
        }
    } else {
        match s {
            "<space>" => Some(KeyCode::Char(' ')),
            "<esc>" => Some(KeyCode::Esc),
            "<tab>" => Some(KeyCode::Tab),
            "<backspace>" => Some(KeyCode::Backspace),
            "<delete>" => Some(KeyCode::Delete),
            "<up>" => Some(KeyCode::Up),
            "<down>" => Some(KeyCode::Down),
            "<left>" => Some(KeyCode::Left),
            "<right>" => Some(KeyCode::Right),
            _ => None,
        }
    }
}
pub fn parse_keybind(s: String) -> Result<Vec<KeyEvent>> {
    let mut out: Vec<KeyEvent> = Vec::new();
    for word in s.split(' ') {
        if word.starts_with("C-") {
            out.push(KeyEvent::new(
                parse_keybind_single(&word[2..])
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::CONTROL,
            ))
        } else if word.starts_with("M-") {
            out.push(KeyEvent::new(
                parse_keybind_single(&word[2..])
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::META,
            ))
        } else if word.starts_with("S-") {
            out.push(KeyEvent::new(
                parse_keybind_single(&word[2..])
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::SUPER,
            ))
        } else if word.starts_with("C-M-") {
            out.push(KeyEvent::new(
                parse_keybind_single(&word[4..])
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::CONTROL | KeyModifiers::META,
            ))
        } else {
            out.push(KeyEvent::new(
                parse_keybind_single(&word)
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::empty(),
            ))
        }
    }
    Ok(out)
}
