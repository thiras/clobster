//! Input event types and key mappings.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Simplified key representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Char(char),
    Enter,
    Escape,
    Backspace,
    Delete,
    Tab,
    BackTab,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
}

impl From<KeyCode> for Key {
    fn from(code: KeyCode) -> Self {
        match code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Enter => Key::Enter,
            KeyCode::Esc => Key::Escape,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Delete => Key::Delete,
            KeyCode::Tab => Key::Tab,
            KeyCode::BackTab => Key::BackTab,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::F(n) => Key::F(n),
            _ => Key::Char('\0'),
        }
    }
}

/// Key modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl From<KeyModifiers> for Modifiers {
    fn from(mods: KeyModifiers) -> Self {
        Self {
            ctrl: mods.contains(KeyModifiers::CONTROL),
            alt: mods.contains(KeyModifiers::ALT),
            shift: mods.contains(KeyModifiers::SHIFT),
        }
    }
}

/// A processed input event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputEvent {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl From<KeyEvent> for InputEvent {
    fn from(event: KeyEvent) -> Self {
        Self {
            key: Key::from(event.code),
            modifiers: Modifiers::from(event.modifiers),
        }
    }
}

impl InputEvent {
    /// Create a new input event.
    pub fn new(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    /// Check if this is a character input.
    pub fn is_char(&self) -> bool {
        matches!(self.key, Key::Char(_))
    }

    /// Get the character if this is a character input.
    pub fn char(&self) -> Option<char> {
        match self.key {
            Key::Char(c) => Some(c),
            _ => None,
        }
    }

    /// Check if Ctrl is held.
    pub fn ctrl(&self) -> bool {
        self.modifiers.ctrl
    }

    /// Check if Alt is held.
    pub fn alt(&self) -> bool {
        self.modifiers.alt
    }

    /// Check if Shift is held.
    pub fn shift(&self) -> bool {
        self.modifiers.shift
    }

    /// Check if this matches a key binding string (e.g., "Ctrl+q", "Enter").
    pub fn matches(&self, binding: &str) -> bool {
        let parts: Vec<&str> = binding.split('+').collect();
        let mut expected_ctrl = false;
        let mut expected_alt = false;
        let mut expected_shift = false;
        let mut expected_key = "";

        for part in parts {
            match part.to_lowercase().as_str() {
                "ctrl" => expected_ctrl = true,
                "alt" => expected_alt = true,
                "shift" => expected_shift = true,
                _ => expected_key = part,
            }
        }

        if self.modifiers.ctrl != expected_ctrl
            || self.modifiers.alt != expected_alt
            || self.modifiers.shift != expected_shift
        {
            return false;
        }

        match expected_key.to_lowercase().as_str() {
            "enter" => self.key == Key::Enter,
            "esc" | "escape" => self.key == Key::Escape,
            "backspace" => self.key == Key::Backspace,
            "delete" | "del" => self.key == Key::Delete,
            "tab" => self.key == Key::Tab,
            "up" => self.key == Key::Up,
            "down" => self.key == Key::Down,
            "left" => self.key == Key::Left,
            "right" => self.key == Key::Right,
            "home" => self.key == Key::Home,
            "end" => self.key == Key::End,
            "pageup" => self.key == Key::PageUp,
            "pagedown" => self.key == Key::PageDown,
            s if s.starts_with('f') && s.len() <= 3 => {
                if let Ok(n) = s[1..].parse::<u8>() {
                    self.key == Key::F(n)
                } else {
                    false
                }
            }
            s if s.len() == 1 => {
                if let Some(c) = s.chars().next() {
                    self.key == Key::Char(c) || self.key == Key::Char(c.to_ascii_uppercase())
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
