use crate::clipboard_history::manager::INTERNAL_CLIPBOARD_CHANGE;
use anyhow::{Context, Result};
use arboard::Clipboard;
use enigo::{Enigo, Key as EnigoKey, Keyboard};
use lazy_static::lazy_static;
use rdev::Key;
use std::collections::HashMap;
use std::sync::{atomic::Ordering, Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(target_os = "linux")]
use evdev::{uinput::VirtualDevice, KeyCode};
#[cfg(target_os = "linux")]
use xkbcommon::xkb;

/// List of known terminal emulator WM_CLASS names (lowercase for comparison)
const TERMINAL_CLASSES: &[&str] = &[
    "gnome-terminal",
    "konsole",
    "xterm",
    "urxvt",
    "rxvt",
    "terminator",
    "tilix",
    "alacritty",
    "kitty",
    "st",
    "foot",
    "wezterm",
    "hyper",
    "guake",
    "yakuake",
    "tilda",
    "terminology",
    "xfce4-terminal",
    "lxterminal",
    "mate-terminal",
    "qterminal",
    "sakura",
    "termite",
    "cool-retro-term",
    "eterm",
    "rio",
    "warp",
    "tabby",
    "blackbox",
    "contour",
    "deepin-terminal",
];

/// Check if the currently focused X11 window is a terminal emulator
#[cfg(target_os = "linux")]
fn is_focused_window_terminal() -> bool {
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, GetPropertyReply};

    let result = (|| -> Option<bool> {
        let (conn, _screen_num) = x11rb::connect(None).ok()?;

        // Get the focused window
        let focus = conn.get_input_focus().ok()?.reply().ok()?;
        let mut window = focus.focus;

        // If focus is on root or None, no terminal
        if window == x11rb::NONE || window == 0 {
            return Some(false);
        }

        // Get WM_CLASS atom
        let wm_class_atom = conn
            .intern_atom(false, b"WM_CLASS")
            .ok()?
            .reply()
            .ok()?
            .atom;

        // Try to get WM_CLASS from the focused window, walking up the tree if needed
        for _ in 0..10 {
            if window == x11rb::NONE || window == 0 {
                break;
            }

            let reply: GetPropertyReply = conn
                .get_property(false, window, wm_class_atom, AtomEnum::STRING, 0, 1024)
                .ok()?
                .reply()
                .ok()?;

            if reply.value_len > 0 {
                // WM_CLASS is two null-terminated strings: instance name and class name
                let value = String::from_utf8_lossy(&reply.value).to_lowercase();
                let parts: Vec<&str> = value.split('\0').collect();

                for part in parts {
                    let part = part.trim();
                    if part.is_empty() {
                        continue;
                    }
                    for terminal in TERMINAL_CLASSES {
                        if part.contains(terminal) {
                            return Some(true);
                        }
                    }
                }
                // Found WM_CLASS but it's not a terminal
                return Some(false);
            }

            // Walk up to parent window
            let tree = conn.query_tree(window).ok()?.reply().ok()?;
            if tree.parent == tree.root || tree.parent == x11rb::NONE {
                break;
            }
            window = tree.parent;
        }

        Some(false)
    })();

    result.unwrap_or(false)
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPress(char),
}

struct InternalClipboardGuard;

impl InternalClipboardGuard {
    fn new() -> Self {
        INTERNAL_CLIPBOARD_CHANGE.store(true, Ordering::SeqCst);
        Self
    }
}

impl Drop for InternalClipboardGuard {
    fn drop(&mut self) {
        INTERNAL_CLIPBOARD_CHANGE.store(false, Ordering::SeqCst);
    }
}

pub trait InputManager: Send + Sync {
    fn start_listening(&self, callback: Box<dyn Fn(InputEvent) + Send + Sync>) -> Result<()>;
    fn inject_text(&self, text: &str) -> Result<()>;
    fn inject_key_clicks(&self, key: EnigoKey, count: usize) -> Result<()>;
}

fn with_clipboard_text<F>(text: &str, is_terminal: bool, paste_action: F) -> Result<()>
where
    F: FnOnce(bool) -> Result<()>,
{
    const CLIPBOARD_PASTE_DELAY: Duration = Duration::from_millis(5);
    let _guard = InternalClipboardGuard::new();

    let mut clipboard = Clipboard::new().context("Failed to initialize clipboard")?;
    let original_content = clipboard.get_text().ok();

    clipboard
        .set_text(text)
        .context("Failed to set clipboard text")?;
    thread::sleep(CLIPBOARD_PASTE_DELAY);

    let paste_result = paste_action(is_terminal);

    thread::sleep(CLIPBOARD_PASTE_DELAY);

    if let Some(original) = original_content {
        if let Err(e) = clipboard.set_text(original) {
            tracing::warn!(error = %e, "Failed to restore clipboard content");
        }
    } else if let Err(e) = clipboard.set_text("") {
        tracing::warn!(error = %e, "Failed to clear clipboard");
    }

    paste_result
}

pub struct RdevInputManager {
    enigo: Mutex<Enigo>,
}

impl RdevInputManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            enigo: Mutex::new(
                Enigo::new(&enigo::Settings::default())
                    .context("Failed to initialize enigo for rdev manager")?,
            ),
        })
    }
}

impl InputManager for RdevInputManager {
    fn start_listening(&self, callback: Box<dyn Fn(InputEvent) + Send + Sync>) -> Result<()> {
        let callback = Arc::new(callback);

        thread::spawn(move || {
            let mut shift_pressed = false;
            let cb = move |event: rdev::Event| match event.event_type {
                rdev::EventType::KeyPress(key) => {
                    if key == Key::ShiftLeft || key == Key::ShiftRight {
                        shift_pressed = true;
                    }
                    if let Some(ch) = key_to_char(&key, shift_pressed) {
                        callback(InputEvent::KeyPress(ch));
                    }
                }
                rdev::EventType::KeyRelease(key) => {
                    if key == Key::ShiftLeft || key == Key::ShiftRight {
                        shift_pressed = false;
                    }
                }
                _ => (),
            };
            if let Err(error) = rdev::listen(cb) {
                tracing::error!(?error, "rdev listener error");
            }
        });
        Ok(())
    }

    fn inject_text(&self, text: &str) -> Result<()> {
        if text.chars().all(|c| c == '\u{8}') {
            return self.inject_key_clicks(EnigoKey::Backspace, text.len());
        }

        let is_terminal = is_focused_window_terminal();

        with_clipboard_text(text, is_terminal, |is_terminal| {
            let mut enigo = self.enigo.lock().expect("enigo mutex poisoned");
            enigo.key(EnigoKey::Control, enigo::Direction::Press)?;
            if is_terminal {
                // Terminals use Ctrl+Shift+V for paste
                enigo.key(EnigoKey::Shift, enigo::Direction::Press)?;
            }
            enigo.key(EnigoKey::Unicode('v'), enigo::Direction::Click)?;
            if is_terminal {
                enigo.key(EnigoKey::Shift, enigo::Direction::Release)?;
            }
            enigo.key(EnigoKey::Control, enigo::Direction::Release)?;
            Ok(())
        })
    }

    fn inject_key_clicks(&self, key: EnigoKey, count: usize) -> Result<()> {
        let mut enigo = self.enigo.lock().expect("enigo mutex poisoned");
        for _ in 0..count {
            enigo.key(key, enigo::Direction::Click)?;
        }
        Ok(())
    }
}

// this implementation for wayland, because wayland is a pain and rdev no worky
#[cfg(target_os = "linux")]
pub struct EvdevInputManager {
    virtual_device: Mutex<VirtualDevice>,
}

#[cfg(target_os = "linux")]
lazy_static! {
    static ref EVDEV_CHAR_MAP: HashMap<char, (KeyCode, bool)> = {
        [
            ('a', (KeyCode::KEY_A, false)),
            ('b', (KeyCode::KEY_B, false)),
            ('c', (KeyCode::KEY_C, false)),
            ('d', (KeyCode::KEY_D, false)),
            ('e', (KeyCode::KEY_E, false)),
            ('f', (KeyCode::KEY_F, false)),
            ('g', (KeyCode::KEY_G, false)),
            ('h', (KeyCode::KEY_H, false)),
            ('i', (KeyCode::KEY_I, false)),
            ('j', (KeyCode::KEY_J, false)),
            ('k', (KeyCode::KEY_K, false)),
            ('l', (KeyCode::KEY_L, false)),
            ('m', (KeyCode::KEY_M, false)),
            ('n', (KeyCode::KEY_N, false)),
            ('o', (KeyCode::KEY_O, false)),
            ('p', (KeyCode::KEY_P, false)),
            ('q', (KeyCode::KEY_Q, false)),
            ('r', (KeyCode::KEY_R, false)),
            ('s', (KeyCode::KEY_S, false)),
            ('t', (KeyCode::KEY_T, false)),
            ('u', (KeyCode::KEY_U, false)),
            ('v', (KeyCode::KEY_V, false)),
            ('w', (KeyCode::KEY_W, false)),
            ('x', (KeyCode::KEY_X, false)),
            ('y', (KeyCode::KEY_Y, false)),
            ('z', (KeyCode::KEY_Z, false)),
            ('A', (KeyCode::KEY_A, true)),
            ('B', (KeyCode::KEY_B, true)),
            ('C', (KeyCode::KEY_C, true)),
            ('D', (KeyCode::KEY_D, true)),
            ('E', (KeyCode::KEY_E, true)),
            ('F', (KeyCode::KEY_F, true)),
            ('G', (KeyCode::KEY_G, true)),
            ('H', (KeyCode::KEY_H, true)),
            ('I', (KeyCode::KEY_I, true)),
            ('J', (KeyCode::KEY_J, true)),
            ('K', (KeyCode::KEY_K, true)),
            ('L', (KeyCode::KEY_L, true)),
            ('M', (KeyCode::KEY_M, true)),
            ('N', (KeyCode::KEY_N, true)),
            ('O', (KeyCode::KEY_O, true)),
            ('P', (KeyCode::KEY_P, true)),
            ('Q', (KeyCode::KEY_Q, true)),
            ('R', (KeyCode::KEY_R, true)),
            ('S', (KeyCode::KEY_S, true)),
            ('T', (KeyCode::KEY_T, true)),
            ('U', (KeyCode::KEY_U, true)),
            ('V', (KeyCode::KEY_V, true)),
            ('W', (KeyCode::KEY_W, true)),
            ('X', (KeyCode::KEY_X, true)),
            ('Y', (KeyCode::KEY_Y, true)),
            ('Z', (KeyCode::KEY_Z, true)),
            ('1', (KeyCode::KEY_1, false)),
            ('2', (KeyCode::KEY_2, false)),
            ('3', (KeyCode::KEY_3, false)),
            ('4', (KeyCode::KEY_4, false)),
            ('5', (KeyCode::KEY_5, false)),
            ('6', (KeyCode::KEY_6, false)),
            ('7', (KeyCode::KEY_7, false)),
            ('8', (KeyCode::KEY_8, false)),
            ('9', (KeyCode::KEY_9, false)),
            ('0', (KeyCode::KEY_0, false)),
            ('!', (KeyCode::KEY_1, true)),
            ('@', (KeyCode::KEY_2, true)),
            ('#', (KeyCode::KEY_3, true)),
            ('$', (KeyCode::KEY_4, true)),
            ('%', (KeyCode::KEY_5, true)),
            ('^', (KeyCode::KEY_6, true)),
            ('&', (KeyCode::KEY_7, true)),
            ('*', (KeyCode::KEY_8, true)),
            ('(', (KeyCode::KEY_9, true)),
            (')', (KeyCode::KEY_0, true)),
            ('-', (KeyCode::KEY_MINUS, false)),
            ('_', (KeyCode::KEY_MINUS, true)),
            ('=', (KeyCode::KEY_EQUAL, false)),
            ('+', (KeyCode::KEY_EQUAL, true)),
            ('[', (KeyCode::KEY_LEFTBRACE, false)),
            ('{', (KeyCode::KEY_LEFTBRACE, true)),
            (']', (KeyCode::KEY_RIGHTBRACE, false)),
            ('}', (KeyCode::KEY_RIGHTBRACE, true)),
            ('\\', (KeyCode::KEY_BACKSLASH, false)),
            ('|', (KeyCode::KEY_BACKSLASH, true)),
            (';', (KeyCode::KEY_SEMICOLON, false)),
            (':', (KeyCode::KEY_SEMICOLON, true)),
            ('\'', (KeyCode::KEY_APOSTROPHE, false)),
            ('"', (KeyCode::KEY_APOSTROPHE, true)),
            (',', (KeyCode::KEY_COMMA, false)),
            ('<', (KeyCode::KEY_COMMA, true)),
            ('.', (KeyCode::KEY_DOT, false)),
            ('>', (KeyCode::KEY_DOT, true)),
            ('/', (KeyCode::KEY_SLASH, false)),
            ('?', (KeyCode::KEY_SLASH, true)),
            ('`', (KeyCode::KEY_GRAVE, false)),
            ('~', (KeyCode::KEY_GRAVE, true)),
            (' ', (KeyCode::KEY_SPACE, false)),
            ('\n', (KeyCode::KEY_ENTER, false)),
            ('\t', (KeyCode::KEY_TAB, false)),
        ]
        .iter()
        .copied()
        .collect()
    };
}

#[cfg(target_os = "linux")]
impl EvdevInputManager {
    pub fn new() -> Result<Self> {
        let mut key_codes: std::collections::HashSet<KeyCode> =
            EVDEV_CHAR_MAP.values().map(|(kc, _)| *kc).collect();
        key_codes.extend([
            KeyCode::KEY_LEFTSHIFT,
            KeyCode::KEY_LEFTCTRL,
            KeyCode::KEY_V,
            KeyCode::KEY_BACKSPACE,
            KeyCode::KEY_LEFT,
        ]);

        let mut attribute_set = evdev::AttributeSet::new();
        for key in key_codes {
            attribute_set.insert(key);
        }

        let uinput_device = evdev::uinput::VirtualDevice::builder()
            .context("Failed to get virtual device builder")?
            .name("Global Automata Text Injection")
            .with_keys(&attribute_set)
            .context("Failed to set keys for virtual device")?
            .build()
            .context("Failed to build virtual device")?;

        Ok(Self {
            virtual_device: Mutex::new(uinput_device),
        })
    }

    fn send_key_click(&self, device: &mut VirtualDevice, key: KeyCode) -> Result<()> {
        let press = evdev::InputEvent::new(evdev::EventType::KEY.0, key.0, 1);
        let release = evdev::InputEvent::new(evdev::EventType::KEY.0, key.0, 0);
        let syn = evdev::InputEvent::new(
            evdev::EventType::SYNCHRONIZATION.0,
            evdev::SynchronizationCode::SYN_REPORT.0,
            0,
        );
        device.emit(&[press, syn.clone()])?;
        device.emit(&[release, syn])?;
        thread::sleep(Duration::from_millis(2));
        Ok(())
    }

    fn enigo_to_evdev(key: EnigoKey) -> Option<KeyCode> {
        match key {
            EnigoKey::LeftArrow => Some(KeyCode::KEY_LEFT),
            EnigoKey::Backspace => Some(KeyCode::KEY_BACKSPACE),
            _ => None,
        }
    }
}

#[cfg(target_os = "linux")]
impl InputManager for EvdevInputManager {
    fn start_listening(&self, callback: Box<dyn Fn(InputEvent) + Send + Sync>) -> Result<()> {
        let devices: Vec<_> = evdev::enumerate()
            .map(|(_, device)| device)
            .filter(|d| {
                d.supported_keys()
                    .map_or(false, |keys| keys.contains(evdev::KeyCode::KEY_ENTER))
            })
            .collect();

        if devices.is_empty() {
            return Err(anyhow::anyhow!(
                "No keyboard devices found. Check permissions for /dev/input/*."
            ));
        }

        let callback = Arc::new(callback);

        for mut device in devices {
            let callback = Arc::clone(&callback);
            let device_name = device.name().unwrap_or("Unnamed Device").to_string();

            thread::spawn(move || {
                let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
                let keymap = xkb::Keymap::new_from_names(
                    &context,
                    "",
                    "",
                    "",
                    "",
                    None,
                    xkb::KEYMAP_COMPILE_NO_FLAGS,
                )
                .expect("Failed to create xkb keymap");
                let mut xkb_state = xkb::State::new(&keymap);

                loop {
                    match device.fetch_events() {
                        Ok(events) => {
                            for ev in events {
                                if ev.event_type() != evdev::EventType::KEY {
                                    continue;
                                }

                                const XKB_KEYCODE_OFFSET: u16 = 8;
                                let keycode = ev.code() + XKB_KEYCODE_OFFSET;
                                let direction = match ev.value() {
                                    0 => xkb::KeyDirection::Up,
                                    1 => xkb::KeyDirection::Down,
                                    _ => continue,
                                };

                                match direction {
                                    xkb::KeyDirection::Down => {
                                        xkb_state.update_key(keycode.into(), direction);

                                        if xkb_state.key_get_one_sym(keycode.into())
                                            == xkb::keysyms::KEY_BackSpace.into()
                                        {
                                            callback(InputEvent::KeyPress('\u{8}'));
                                        } else {
                                            let utf8_str = xkb_state.key_get_utf8(keycode.into());
                                            for ch in utf8_str.chars() {
                                                callback(InputEvent::KeyPress(ch));
                                            }
                                        }
                                    }
                                    _ => {
                                        xkb_state.update_key(keycode.into(), direction);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            if e.kind() != std::io::ErrorKind::WouldBlock {
                                tracing::error!(
                                    device = %device_name,
                                    error = %e,
                                    "Error fetching evdev events"
                                );
                                break;
                            }
                        }
                    }
                }
            });
        }

        Ok(())
    }

    fn inject_text(&self, text: &str) -> Result<()> {
        if text.chars().all(|c| c == '\u{8}') {
            return self.inject_key_clicks(EnigoKey::Backspace, text.len());
        }

        let is_terminal = is_focused_window_terminal();

        with_clipboard_text(text, is_terminal, |is_terminal| {
            let mut device = self
                .virtual_device
                .lock()
                .expect("virtual device mutex poisoned");
            let syn = evdev::InputEvent::new(
                evdev::EventType::SYNCHRONIZATION.0,
                evdev::SynchronizationCode::SYN_REPORT.0,
                0,
            );

            // Press Ctrl
            device.emit(&[
                evdev::InputEvent::new(evdev::EventType::KEY.0, KeyCode::KEY_LEFTCTRL.0, 1),
                syn.clone(),
            ])?;

            // For terminals, also press Shift (Ctrl+Shift+V)
            if is_terminal {
                device.emit(&[
                    evdev::InputEvent::new(evdev::EventType::KEY.0, KeyCode::KEY_LEFTSHIFT.0, 1),
                    syn.clone(),
                ])?;
            }

            self.send_key_click(&mut device, KeyCode::KEY_V)?;

            // Release Shift if pressed
            if is_terminal {
                device.emit(&[
                    evdev::InputEvent::new(evdev::EventType::KEY.0, KeyCode::KEY_LEFTSHIFT.0, 0),
                    syn.clone(),
                ])?;
            }

            // Release Ctrl
            device.emit(&[
                evdev::InputEvent::new(evdev::EventType::KEY.0, KeyCode::KEY_LEFTCTRL.0, 0),
                syn,
            ])?;
            Ok(())
        })
    }

    fn inject_key_clicks(&self, key: EnigoKey, count: usize) -> Result<()> {
        if let Some(keycode) = Self::enigo_to_evdev(key) {
            let mut device = self
                .virtual_device
                .lock()
                .expect("virtual device mutex poisoned");
            for _ in 0..count {
                self.send_key_click(&mut *device, keycode)?;
            }
        }
        Ok(())
    }
}

lazy_static! {
    static ref RDEV_KEY_MAP: HashMap<Key, (char, char)> = {
        [
            (Key::KeyA, ('a', 'A')),
            (Key::KeyB, ('b', 'B')),
            (Key::KeyC, ('c', 'C')),
            (Key::KeyD, ('d', 'D')),
            (Key::KeyE, ('e', 'E')),
            (Key::KeyF, ('f', 'F')),
            (Key::KeyG, ('g', 'G')),
            (Key::KeyH, ('h', 'H')),
            (Key::KeyI, ('i', 'I')),
            (Key::KeyJ, ('j', 'J')),
            (Key::KeyK, ('k', 'K')),
            (Key::KeyL, ('l', 'L')),
            (Key::KeyM, ('m', 'M')),
            (Key::KeyN, ('n', 'N')),
            (Key::KeyO, ('o', 'O')),
            (Key::KeyP, ('p', 'P')),
            (Key::KeyQ, ('q', 'Q')),
            (Key::KeyR, ('r', 'R')),
            (Key::KeyS, ('s', 'S')),
            (Key::KeyT, ('t', 'T')),
            (Key::KeyU, ('u', 'U')),
            (Key::KeyV, ('v', 'V')),
            (Key::KeyW, ('w', 'W')),
            (Key::KeyX, ('x', 'X')),
            (Key::KeyY, ('y', 'Y')),
            (Key::KeyZ, ('z', 'Z')),
            (Key::Num0, ('0', ')')),
            (Key::Num1, ('1', '!')),
            (Key::Num2, ('2', '@')),
            (Key::Num3, ('3', '#')),
            (Key::Num4, ('4', '$')),
            (Key::Num5, ('5', '%')),
            (Key::Num6, ('6', '^')),
            (Key::Num7, ('7', '&')),
            (Key::Num8, ('8', '*')),
            (Key::Num9, ('9', '(')),
            (Key::Space, (' ', ' ')),
            (Key::Slash, ('/', '?')),
            (Key::Dot, ('.', '>')),
            (Key::Comma, (',', '<')),
            (Key::Minus, ('-', '_')),
            (Key::Equal, ('=', '+')),
            (Key::LeftBracket, ('[', '{')),
            (Key::RightBracket, (']', '}')),
            (Key::BackSlash, ('\\', '|')),
            (Key::SemiColon, (';', ':')),
            (Key::Quote, ('\'', '"')),
            (Key::BackQuote, ('`', '~')),
        ]
        .iter()
        .copied()
        .collect()
    };
}

pub fn key_to_char(key: &Key, is_shifted: bool) -> Option<char> {
    match key {
        Key::Backspace => Some('\u{8}'),
        Key::Return | Key::KpReturn => Some('\n'),
        Key::Tab => Some('\t'),
        _ => RDEV_KEY_MAP
            .get(key)
            .map(|(c, s)| if is_shifted { *s } else { *c }),
    }
}
