use clap::{Parser, Subcommand};
use std::io::{self, BufRead};

/// Flare Launcher - A Raycast-compatible launcher for Linux
#[derive(Parser)]
#[command(name = "flare")]
#[command(about = "A focused launcher for your desktop", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// dmenu compatibility mode - read options from stdin, output selection to stdout
    Dmenu {
        /// Case insensitive matching
        #[arg(short = 'i')]
        case_insensitive: bool,

        /// Prompt string to display
        #[arg(short = 'p', default_value = "")]
        prompt: String,

        /// Number of lines to display (ignored, for compatibility)
        #[arg(short = 'l')]
        lines: Option<u32>,

        /// Monitor to display on (ignored, for compatibility)
        #[arg(short = 'm')]
        monitor: Option<i32>,

        /// Font (ignored, for compatibility)
        #[arg(short = 'f', long = "fn")]
        font: Option<String>,
    },
}

/// Holds the state for a dmenu session
#[derive(Debug, Clone)]
pub struct DmenuSession {
    pub items: Vec<String>,
    pub case_insensitive: bool,
    pub prompt: String,
}

impl DmenuSession {
    /// Create a new DmenuSession by reading items from stdin
    pub fn from_stdin(case_insensitive: bool, prompt: String) -> io::Result<Self> {
        let stdin = io::stdin();
        let items: Vec<String> = stdin
            .lock()
            .lines()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            items,
            case_insensitive,
            prompt,
        })
    }

    /// Output the selected item to stdout
    pub fn output_selection(&self, selection: &str) {
        println!("{}", selection);
    }

    /// Filter items based on search query
    pub fn filter_items(&self, query: &str) -> Vec<String> {
        if query.is_empty() {
            return self.items.clone();
        }

        let query_lower = query.to_lowercase();
        self.items
            .iter()
            .filter(|item| {
                if self.case_insensitive {
                    item.to_lowercase().contains(&query_lower)
                } else {
                    item.contains(query)
                }
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dmenu_session_empty() {
        let session = DmenuSession {
            items: vec![],
            case_insensitive: false,
            prompt: String::new(),
        };
        assert!(session.items.is_empty());
    }

    #[test]
    fn test_dmenu_session_with_items() {
        let session = DmenuSession {
            items: vec!["Option 1".into(), "Option 2".into()],
            case_insensitive: true,
            prompt: "Select:".into(),
        };
        assert_eq!(session.items.len(), 2);
        assert!(session.case_insensitive);
        assert_eq!(session.prompt, "Select:");
    }

    #[test]
    fn test_filter_case_sensitive() {
        let session = DmenuSession {
            items: vec!["Firefox".into(), "CHROME".into(), "vivaldi".into()],
            case_insensitive: false,
            prompt: String::new(),
        };
        let filtered = session.filter_items("Fire");
        assert_eq!(filtered, vec!["Firefox"]);
    }

    #[test]
    fn test_filter_case_insensitive() {
        let session = DmenuSession {
            items: vec!["Firefox".into(), "CHROME".into(), "vivaldi".into()],
            case_insensitive: true,
            prompt: String::new(),
        };
        let filtered = session.filter_items("chrome");
        assert_eq!(filtered, vec!["CHROME"]);
    }

    #[test]
    fn test_filter_empty_query() {
        let session = DmenuSession {
            items: vec!["A".into(), "B".into(), "C".into()],
            case_insensitive: false,
            prompt: String::new(),
        };
        let filtered = session.filter_items("");
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_filter_no_matches() {
        let session = DmenuSession {
            items: vec!["Firefox".into(), "Chrome".into()],
            case_insensitive: false,
            prompt: String::new(),
        };
        let filtered = session.filter_items("Safari");
        assert!(filtered.is_empty());
    }
}
