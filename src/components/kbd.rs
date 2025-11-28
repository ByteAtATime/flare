use iced::{
    Border, Element,
    keyboard::{Key, Modifiers},
    widget::{container, row, text},
};

use crate::theme::Theme;

fn render_key<'a, Message>(theme: &'a Theme, key: impl Into<String>) -> Element<'a, Message>
where
    Message: 'a,
{
    container(text(key.into()).size(15))
        .style(|_theme| container::Style {
            background: Some(theme.colors.text_10.into()),
            border: Border::default().rounded(3),
            ..Default::default()
        })
        .padding([0, 6])
        .center_y(23) // weird number? idk why
        .into()
}

pub fn render_kbd<'a, Message>(
    theme: &'a Theme,
    key: Key,
    modifiers: Modifiers,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let mut row = row![].spacing(2);

    if modifiers.contains(Modifiers::COMMAND) {
        row = row.push(render_key(theme, "Ctrl"));
    }

    if modifiers.contains(Modifiers::ALT) {
        row = row.push(render_key(theme, "Alt"));
    }

    if modifiers.contains(Modifiers::SHIFT) {
        row = row.push(render_key(theme, "Shift"));
    }

    match key {
        Key::Character(c) => {
            row = row.push(render_key(theme, &c.to_uppercase().to_string()));
        }
        Key::Named(named_key) => {
            use iced::keyboard::key::Named;
            let key_str = match named_key {
                Named::Enter => "↵".to_string(),
                Named::Escape => "⎋".to_string(),
                Named::Backspace => "⌫".to_string(),
                Named::Tab => "⇥".to_string(),
                Named::Space => "␣".to_string(),
                Named::ArrowUp => "↑".to_string(),
                Named::ArrowDown => "↓".to_string(),
                Named::ArrowLeft => "←".to_string(),
                Named::ArrowRight => "→".to_string(),
                _ => format!("{:?}", named_key),
            };
            if !key_str.is_empty() {
                row = row.push(render_key(theme, key_str));
            }
        }
        _ => {}
    }

    row.into()
}
