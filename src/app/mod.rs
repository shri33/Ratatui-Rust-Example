//! Application module
//!
//! Contains the main application state, configuration management,
//! and clipboard integration functionality.

pub mod config;

use once_cell::sync::Lazy;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Mutex;

pub use config::Config;

/// Maximum number of clipboard history entries
const MAX_CLIPBOARD_HISTORY: usize = 50;

/// Input modes for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMode {
    /// Normal navigation mode
    Normal,
    /// Text editing mode
    Editing,
    /// Emoji picker mode
    EmojiPicker,
    /// Clipboard history mode
    ClipboardHistory,
}

impl Default for InputMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// Application state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    /// Current input mode
    pub input_mode: InputMode,
    /// Currently selected input field (0=text, 1=emoji, 2=hyperlink)
    pub selected_input: usize,
    /// Text input content
    pub text_input: String,
    /// Emoji input content
    pub emoji_input: String,
    /// Hyperlink input content
    pub hyperlink_input: String,
    /// Current emoji category index
    pub emoji_category_index: usize,
    /// Current emoji index within category
    pub emoji_index: usize,
    /// Clipboard history
    pub clipboard_history: VecDeque<String>,
    /// Selected clipboard history index
    pub clipboard_history_index: usize,
    /// Application configuration
    pub config: Config,
    /// Whether the application should quit
    pub should_quit: bool,
    /// Current tab index
    pub current_tab: usize,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        Self {
            input_mode: InputMode::Normal,
            selected_input: 0,
            text_input: String::new(),
            emoji_input: String::new(),
            hyperlink_input: String::new(),
            emoji_category_index: 0,
            emoji_index: 0,
            clipboard_history: VecDeque::new(),
            clipboard_history_index: 0,
            config: Config::default(),
            should_quit: false,
            current_tab: 0,
        }
    }

    /// Get the primary color from config
    pub fn primary_color(&self) -> Color {
        match self.config.theme_index {
            0 => Color::Cyan,
            1 => Color::Green,
            2 => Color::Yellow,
            3 => Color::Red,
            4 => Color::Blue,
            5 => Color::Magenta,
            _ => Color::White,
        }
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 3;
    }

    /// Switch to previous tab
    pub fn prev_tab(&mut self) {
        self.current_tab = if self.current_tab == 0 {
            2
        } else {
            self.current_tab - 1
        };
    }

    /// Copy content to clipboard and add to history
    pub fn copy_to_clipboard(&mut self, content: &str) {
        if content.is_empty() {
            return;
        }

        // Add to clipboard history
        self.clipboard_history.push_front(content.to_string());

        // Limit history size
        if self.clipboard_history.len() > MAX_CLIPBOARD_HISTORY {
            self.clipboard_history.pop_back();
        }

        // Reset history index
        self.clipboard_history_index = 0;

        // Copy to system clipboard
        if let Err(e) = self.copy_to_system_clipboard(content) {
            log::error!("Failed to copy to clipboard: {e:?}");
        }
    }

    /// Copy to system clipboard
    fn copy_to_system_clipboard(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        use arboard::Clipboard;
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(content)?;
        Ok(())
    }

    /// Paste from clipboard
    pub fn paste_from_clipboard(&self) -> String {
        self.get_system_clipboard().unwrap_or_else(|e| {
            log::error!("Failed to get clipboard content: {e:?}");
            String::new()
        })
    }

    /// Get system clipboard content
    fn get_system_clipboard(&self) -> Result<String, Box<dyn std::error::Error>> {
        use arboard::Clipboard;
        let mut clipboard = Clipboard::new()?;
        Ok(clipboard.get_text()?)
    }

    /// Get clipboard history item at index
    pub fn get_clipboard_history_item(&self, index: usize) -> Option<&String> {
        self.clipboard_history.get(index)
    }

    /// Get clipboard history length
    pub fn clipboard_history_len(&self) -> usize {
        self.clipboard_history.len()
    }

    /// Get theme colors
    pub fn theme_colors(&self) -> (Color, Color, Color) {
        match self.config.theme_index {
            0 => (Color::Cyan, Color::Blue, Color::Gray),
            1 => (Color::Green, Color::LightGreen, Color::DarkGray),
            2 => (Color::Yellow, Color::LightYellow, Color::Gray),
            3 => (Color::Red, Color::LightRed, Color::DarkGray),
            4 => (Color::Blue, Color::LightBlue, Color::Gray),
            5 => (Color::Magenta, Color::LightMagenta, Color::DarkGray),
            _ => (Color::White, Color::Gray, Color::DarkGray),
        }
    }

    /// Switch to next theme
    pub fn next_theme(&mut self) {
        self.config.theme_index = (self.config.theme_index + 1) % 6;
    }

    /// Add table_state field and methods
    pub fn table_state() -> &'static Mutex<ratatui::widgets::TableState> {
        static TABLE_STATE: Lazy<Mutex<ratatui::widgets::TableState>> =
            Lazy::new(|| Mutex::new(ratatui::widgets::TableState::default()));
        &TABLE_STATE
    }
}

/// Available emoji categories for the picker
pub const EMOJI_CATEGORIES: &[&str] = &[
    "smileys",
    "animals",
    "food",
    "travel",
    "activities",
    "nature",
    "objects",
    "symbols",
];

/// Emoji mappings by category
pub const EMOJI_MAP: &[(&str, &[&str])] = &[
    (
        "smileys",
        &["😀", "😃", "😄", "😁", "😆", "😅", "🤣", "😂", "🙂", "🙃"],
    ),
    (
        "animals",
        &["🐶", "🐱", "🐭", "🐹", "🐰", "🦊", "🐻", "🐼", "🐨", "🐯"],
    ),
    (
        "food",
        &["🍎", "🍊", "🍋", "🍌", "🍉", "🍇", "🍓", "🫐", "🍈", "🍒"],
    ),
    (
        "travel",
        &["🚗", "🚕", "🚙", "🚌", "🚎", "🏎️", "🚓", "🚑", "🚒", "🚐"],
    ),
    (
        "activities",
        &["⚽", "🏀", "🏈", "⚾", "🥎", "🎾", "🏐", "🏉", "🥏", "🎱"],
    ),
    (
        "nature",
        &["🌲", "🌳", "🌴", "🌵", "🌶️", "🌷", "🌸", "🌹", "🌺", "🌻"],
    ),
    (
        "objects",
        &["💎", "🔔", "🎵", "🎶", "💝", "🎁", "📱", "💻", "⌚", "📷"],
    ),
    (
        "symbols",
        &["❤️", "💙", "💚", "💛", "🧡", "💜", "🖤", "🤍", "🤎", "💔"],
    ),
];

// Config static for global access (remove duplicate imports)
static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

// Then access it like this:
pub fn get_config() -> Config {
    CONFIG.lock().unwrap().clone().unwrap_or_default()
}

pub fn set_config(config: Config) {
    let mut guard = CONFIG.lock().unwrap();
    *guard = Some(config);
}

// src/app/mod.rs
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("App logic goes here!");
    Ok(())
}
