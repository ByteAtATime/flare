use crate::clipboard_history::manager::{
    ClipboardHistoryManager, MANAGER as CLIPBOARD_MANAGER_STATIC,
};
use crate::error::AppError;
use crate::snippets::input_manager::{InputEvent, InputManager};
use crate::snippets::manager::SnippetManager;
use arboard::Clipboard;
use chrono::{DateTime, Duration, Local, Months};
use enigo::Key as EnigoKey;
use once_cell::sync::Lazy;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use uuid::Uuid;

const BUFFER_SIZE: usize = 30;
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

static PLACEHOLDER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"\{(?P<name>\w+)(?P<attributes>(?:\s+\w+=(?:"[^"]*"|\S+))*)?(?P<modifiers>(?:\s*\|\s*[\w%-]+)*)\}"#).unwrap()
});
static ATTRIBUTE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"\s*(?P<key>\w+)=(?:"(?P<q_value>[^"]*)"|(?P<uq_value>\S+))"#).unwrap()
});
static OFFSET_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?P<sign>[+-])(?P<num>\d+)(?P<unit>[ymhMd])").unwrap());

pub struct ResolvedSnippet {
    pub content: String,
    pub cursor_pos: Option<usize>,
}

#[derive(Debug)]
struct ParsedPlaceholder<'a> {
    name: &'a str,
    attributes: HashMap<&'a str, &'a str>,
    modifiers: Vec<&'a str>,
}

pub struct ExpansionEngine {
    buffer: Arc<Mutex<String>>,
    snippet_manager: Arc<SnippetManager>,
    input_manager: Arc<dyn InputManager>,
}

impl ExpansionEngine {
    pub fn new(snippet_manager: Arc<SnippetManager>, input_manager: Arc<dyn InputManager>) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(String::with_capacity(BUFFER_SIZE))),
            snippet_manager,
            input_manager,
        }
    }

    pub fn start_listening(&self) -> anyhow::Result<()> {
        let engine = Arc::new(self.clone_for_thread());
        self.input_manager.start_listening(Box::new(move |event| {
            engine.handle_key_press(event);
        }))?;
        Ok(())
    }

    fn clone_for_thread(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            snippet_manager: self.snippet_manager.clone(),
            input_manager: self.input_manager.clone(),
        }
    }

    fn handle_key_press(&self, event: InputEvent) {
        let InputEvent::KeyPress(ch) = event;
        let mut buffer = self.buffer.lock().expect("snippet buffer mutex poisoned");

        match ch {
            '\u{8}' => {
                buffer.pop();
            }
            '\n' | '\t' | '\u{1b}' => {
                buffer.clear();
            }
            ch if ch.is_control() => (),
            _ => {
                buffer.push(ch);
                if buffer.len() > BUFFER_SIZE {
                    buffer.remove(0);
                }
            }
        }

        if let Ok(snippets) = self.snippet_manager.list_snippets(None) {
            for snippet in snippets {
                if buffer.ends_with(&snippet.keyword) {
                    let (keyword, content, id) =
                        (snippet.keyword.clone(), snippet.content.clone(), snippet.id);
                    let manager = self.snippet_manager.clone();
                    drop(buffer);
                    self.expand_snippet(&keyword, &content);
                    thread::spawn(move || {
                        let _ = manager.snippet_was_used(id);
                    });
                    break;
                }
            }
        }
    }

    fn expand_snippet(&self, keyword: &str, content: &str) {
        let mut backspaces = String::new();
        for _ in 0..keyword.len() {
            backspaces.push('\u{8}');
        }

        let clipboard_manager_lock = CLIPBOARD_MANAGER_STATIC
            .lock()
            .expect("clipboard manager mutex poisoned");
        let resolved_result = parse_and_resolve_placeholders(
            content,
            &self.snippet_manager,
            clipboard_manager_lock.as_ref(),
        );

        let resolved = match resolved_result {
            Ok(res) => res,
            Err(e) => {
                tracing::error!(error = %e, "Error resolving placeholders");
                ResolvedSnippet {
                    content: content.to_string(),
                    cursor_pos: None,
                }
            }
        };

        let content_to_paste = resolved.content;

        let chars_to_move_left = if let Some(pos) = resolved.cursor_pos {
            content_to_paste.chars().count() - pos
        } else {
            0
        };

        let input_manager = self.input_manager.clone();

        thread::spawn(move || {
            if let Err(e) = input_manager.inject_text(&backspaces) {
                tracing::error!(error = %e, "Failed to inject backspaces");
            }
            thread::sleep(std::time::Duration::from_millis(50));
            if let Err(e) = input_manager.inject_text(&content_to_paste) {
                tracing::error!(error = %e, "Failed to inject snippet content");
            }

            if chars_to_move_left > 0 {
                thread::sleep(std::time::Duration::from_millis(50));
                if let Err(e) =
                    input_manager.inject_key_clicks(EnigoKey::LeftArrow, chars_to_move_left)
                {
                    tracing::error!(error = %e, "Failed to inject cursor movement");
                }
            }
        });

        let mut buffer = self.buffer.lock().expect("snippet buffer mutex poisoned");
        buffer.clear();
    }
}

fn parse_attributes(attr_str: &str) -> HashMap<&str, &str> {
    ATTRIBUTE_REGEX
        .captures_iter(attr_str)
        .filter_map(|cap| {
            let key = cap.name("key")?.as_str();
            let value = cap
                .name("q_value")
                .or_else(|| cap.name("uq_value"))?
                .as_str();
            Some((key, value))
        })
        .collect()
}

fn parse_modifiers(mod_str: &str) -> Vec<&str> {
    mod_str
        .split('|')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

fn apply_modifiers(mut value: String, modifiers: &[&str]) -> String {
    for &modifier in modifiers {
        value = match modifier {
            "uppercase" => value.to_uppercase(),
            "lowercase" => value.to_lowercase(),
            "trim" => value.trim().to_string(),
            "percent-encode" => utf8_percent_encode(&value, FRAGMENT).to_string(),
            "json-stringify" => serde_json::to_string(&value).unwrap_or(value),
            _ => value,
        };
    }
    value
}

fn translate_date_format(format_str: &str) -> String {
    let mut result = String::with_capacity(format_str.len());
    let mut in_literal = false;
    let mut chars = format_str.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\'' {
            in_literal = !in_literal;
            continue;
        }

        if in_literal {
            result.push(c);
            continue;
        }

        let mut count = 1;
        while chars.peek() == Some(&c) {
            chars.next();
            count += 1;
        }

        let format_specifier = match (c, count) {
            // year
            ('y', 4) => "%Y",
            ('y', 2) => "%y",
            // month
            ('M', 4) => "%B",
            ('M', 3) => "%b",
            ('M', 2) => "%m",
            ('M', 1) => "%-m",
            // day of week
            ('E', 4) => "%A",
            ('E', 1..=3) => "%a",
            // day of month
            ('d', 2) => "%d",
            ('d', 1) => "%-d",
            // hour (0-23)
            ('H', 2) => "%H",
            ('H', 1) => "%-H",
            // hour (1-12)
            ('h', 2) => "%I",
            ('h', 1) => "%-I",
            // minute
            ('m', 2) => "%M",
            ('m', 1) => "%-M",
            // second
            ('s', 2) => "%S",
            ('s', 1) => "%-S",
            // fractional second
            ('S', 3) => "%f",
            // am/pm
            ('a', 1) => "%p",
            // timezone
            ('Z', 1) => "%z",

            (other_char, num_chars) => {
                // treat as literal
                for _ in 0..num_chars {
                    result.push(other_char);
                }

                ""
            }
        };

        result.push_str(format_specifier);
    }
    result
}

fn resolve_value<'a>(
    placeholder: &ParsedPlaceholder,
    snippet_manager: &SnippetManager,
    clipboard_manager: Option<&ClipboardHistoryManager>,
) -> Result<String, AppError> {
    let now = Local::now();

    match placeholder.name {
        "cursor" => Ok(String::new()),
        "uuid" => Ok(Uuid::new_v4().to_string().to_uppercase()),
        "clipboard" => {
            let offset: u32 = placeholder
                .attributes
                .get("offset")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            if offset > 0 {
                if let Some(cm) = clipboard_manager {
                    if let Some(content) = cm.get_content_by_offset(offset)? {
                        return Ok(content);
                    }
                }
                return Ok(String::new());
            }

            Ok(Clipboard::new()
                .ok()
                .and_then(|mut c| c.get_text().ok())
                .unwrap_or_default())
        }
        "snippet" => {
            if let Some(name) = placeholder.attributes.get("name") {
                if let Some(snippet) = snippet_manager.find_snippet_by_name(name)? {
                    if PLACEHOLDER_REGEX.is_match(&snippet.content) {
                        return Ok(String::new());
                    }
                    return Ok(snippet.content);
                }
            }
            Ok(String::new())
        }
        "date" | "time" | "datetime" | "day" => {
            let mut date_time: DateTime<Local> = now;
            if let Some(offset_str) = placeholder.attributes.get("offset") {
                for cap in OFFSET_REGEX.captures_iter(offset_str) {
                    let sign = cap.name("sign").unwrap().as_str();
                    let num: i64 = cap.name("num").unwrap().as_str().parse().unwrap_or(0);
                    let val = if sign == "-" { -num } else { num };

                    match cap.name("unit").unwrap().as_str() {
                        "y" => {
                            let months_to_add = val * 12;
                            if months_to_add > 0 {
                                date_time = date_time
                                    .checked_add_months(Months::new(months_to_add as u32))
                                    .unwrap_or(date_time);
                            } else if months_to_add < 0 {
                                date_time = date_time
                                    .checked_sub_months(Months::new(-months_to_add as u32))
                                    .unwrap_or(date_time);
                            }
                        }
                        "M" => {
                            if val > 0 {
                                date_time = date_time
                                    .checked_add_months(Months::new(val as u32))
                                    .unwrap_or(date_time);
                            } else if val < 0 {
                                date_time = date_time
                                    .checked_sub_months(Months::new(-val as u32))
                                    .unwrap_or(date_time);
                            }
                        }
                        "d" => {
                            date_time = date_time
                                .checked_add_signed(Duration::days(val))
                                .unwrap_or(date_time)
                        }
                        "h" => {
                            date_time = date_time
                                .checked_add_signed(Duration::hours(val))
                                .unwrap_or(date_time)
                        }
                        "m" => {
                            date_time = date_time
                                .checked_add_signed(Duration::minutes(val))
                                .unwrap_or(date_time)
                        }
                        _ => {}
                    }
                }
            }

            let format_str = if let Some(fmt) = placeholder.attributes.get("format") {
                translate_date_format(fmt)
            } else {
                match placeholder.name {
                    "date" => "%-d %b %Y".to_string(),
                    "time" => "%-I:%M %p".to_string(),
                    "datetime" => "%-d %b %Y at %-I:%M %p".to_string(),
                    "day" => "%A".to_string(),
                    _ => "".to_string(),
                }
            };

            Ok(date_time
                .format(&format_str)
                .to_string()
                .replace("am", "AM")
                .replace("pm", "PM"))
        }
        _ => Ok(String::new()),
    }
}

pub fn parse_and_resolve_placeholders(
    raw_content: &str,
    snippet_manager: &SnippetManager,
    clipboard_manager: Option<&ClipboardHistoryManager>,
) -> Result<ResolvedSnippet, AppError> {
    let mut resolved_content = String::with_capacity(raw_content.len());
    let mut cursor_pos: Option<usize> = None;
    let mut last_end = 0;

    for cap in PLACEHOLDER_REGEX.captures_iter(raw_content) {
        let full_match = cap.get(0).unwrap();
        resolved_content.push_str(&raw_content[last_end..full_match.start()]);

        let name = cap.name("name").unwrap().as_str();
        let attributes = parse_attributes(cap.name("attributes").map_or("", |m| m.as_str()));
        let modifiers = parse_modifiers(cap.name("modifiers").map_or("", |m| m.as_str()));

        let placeholder = ParsedPlaceholder {
            name,
            attributes,
            modifiers,
        };

        if placeholder.name == "cursor" {
            if cursor_pos.is_none() {
                cursor_pos = Some(resolved_content.chars().count());
            }
        } else {
            let value = resolve_value(&placeholder, snippet_manager, clipboard_manager)?;
            let modified_value = apply_modifiers(value, &placeholder.modifiers);
            resolved_content.push_str(&modified_value);
        }
        last_end = full_match.end();
    }
    resolved_content.push_str(&raw_content[last_end..]);

    Ok(ResolvedSnippet {
        content: resolved_content,
        cursor_pos,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clipboard_history::types::ContentType;

    #[test]
    fn test_plain_text() {
        let snippet_manager = SnippetManager::new_for_test().unwrap();
        let content = "Just some plain text.";
        let result = parse_and_resolve_placeholders(content, &snippet_manager, None).unwrap();
        assert_eq!(result.content, "Just some plain text.");
        assert_eq!(result.cursor_pos, None);
    }

    #[test]
    fn test_cursor_placeholder() {
        let snippet_manager = SnippetManager::new_for_test().unwrap();
        let content = "Hello, {cursor}World!";
        let result = parse_and_resolve_placeholders(content, &snippet_manager, None).unwrap();
        assert_eq!(result.content, "Hello, World!");
        assert_eq!(result.cursor_pos, Some(7));
    }

    #[test]
    fn test_multiple_cursor_placeholders() {
        let snippet_manager = SnippetManager::new_for_test().unwrap();
        let content = "{cursor} first, then {cursor} second";
        let result = parse_and_resolve_placeholders(content, &snippet_manager, None).unwrap();
        assert_eq!(result.content, " first, then  second");
        assert_eq!(result.cursor_pos, Some(0));
    }

    #[test]
    fn test_uuid_placeholder() {
        let snippet_manager = SnippetManager::new_for_test().unwrap();
        let content = "{uuid}";
        let result = parse_and_resolve_placeholders(content, &snippet_manager, None).unwrap();
        assert!(Uuid::parse_str(&result.content).is_ok());
    }

    #[test]
    fn test_clipboard_offset_placeholder() {
        let snippet_manager = SnippetManager::new_for_test().unwrap();
        let clipboard_manager = ClipboardHistoryManager::new_for_test().unwrap();
        clipboard_manager
            .add_item("hash1".into(), ContentType::Text, "Item 1".into(), None)
            .unwrap();
        thread::sleep(std::time::Duration::from_millis(1));
        clipboard_manager
            .add_item("hash0".into(), ContentType::Text, "Item 0".into(), None)
            .unwrap();

        let content = "Clipboard item at offset 1 is: {clipboard offset=\"1\"}";
        let result =
            parse_and_resolve_placeholders(content, &snippet_manager, Some(&clipboard_manager))
                .unwrap();

        assert_eq!(result.content, "Clipboard item at offset 1 is: Item 1");
    }

    #[test]
    fn test_snippet_placeholder() {
        let snippet_manager = SnippetManager::new_for_test().unwrap();
        snippet_manager
            .create_snippet("Greeting".into(), "greet".into(), "Hello, there!".into())
            .unwrap();

        let content = "I'd like to say: {snippet name=\"Greeting\"}";
        let result = parse_and_resolve_placeholders(content, &snippet_manager, None).unwrap();

        assert_eq!(result.content, "I'd like to say: Hello, there!");
    }

    #[test]
    fn test_recursive_snippet_placeholder() {
        let snippet_manager = SnippetManager::new_for_test().unwrap();
        snippet_manager
            .create_snippet(
                "recursion".into(),
                "recur".into(),
                "Oh no {snippet name=\"recursion\"}".into(),
            )
            .unwrap();

        let content = "{snippet name=\"recursion\"}";
        let result = parse_and_resolve_placeholders(content, &snippet_manager, None).unwrap();

        assert_eq!(result.content, "");
    }

    #[test]
    fn test_modifiers() {
        let snippet_manager = SnippetManager::new_for_test().unwrap();
        let content = "{snippet name=\"test\" | uppercase | trim}";
        snippet_manager
            .create_snippet("test".into(), "test".into(), "  some text  ".into())
            .unwrap();
        let result = parse_and_resolve_placeholders(content, &snippet_manager, None).unwrap();
        assert_eq!(result.content, "SOME TEXT");
    }

    #[test]
    fn test_json_stringify_modifier() {
        let snippet_manager = SnippetManager::new_for_test().unwrap();
        let content = "{snippet name=\"json\" | json-stringify}";
        snippet_manager
            .create_snippet("json".into(), "json".into(), "line1\nline2".into())
            .unwrap();
        let result = parse_and_resolve_placeholders(content, &snippet_manager, None).unwrap();
        assert_eq!(result.content, "\"line1\\nline2\"");
    }

    #[test]
    fn test_translate_date_format_string() {
        let raycast_format = "MMMM dd, yyyy 'at' hh:mm:ss a";
        let strftime_format = translate_date_format(raycast_format);
        assert_eq!(strftime_format, "%B %d, %Y at %I:%M:%S %p");
    }

    #[test]
    fn test_date_with_format_and_offset() {
        let snippet_manager = SnippetManager::new_for_test().unwrap();
        let content = "{date offset=\"+1d\" format=\"yyyy-MM-dd\"}";
        let result = parse_and_resolve_placeholders(content, &snippet_manager, None).unwrap();

        let tomorrow = Local::now().checked_add_signed(Duration::days(1)).unwrap();
        let expected = tomorrow.format("%Y-%m-%d").to_string();

        assert_eq!(result.content, expected);
    }

    #[test]
    fn test_invalid_placeholder() {
        let snippet_manager = SnippetManager::new_for_test().unwrap();
        let content = "Hello, {invalid_placeholder}!";
        let result = parse_and_resolve_placeholders(content, &snippet_manager, None).unwrap();
        assert_eq!(result.content, "Hello, !");
    }

    #[test]
    fn test_unclosed_placeholder() {
        let snippet_manager = SnippetManager::new_for_test().unwrap();
        let content = "Hello, {unclosed";
        let result = parse_and_resolve_placeholders(content, &snippet_manager, None).unwrap();
        assert_eq!(result.content, "Hello, {unclosed");
    }
}
