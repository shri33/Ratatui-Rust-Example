//! Input widget module providing text input, emoji picker, and hyperlink functionality

use std::io::{self, Result as IoResult};
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style},
    layout::Rect,
    Frame,
};

/// Input modes for the application
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    EmojiPicker,
    ClipboardHistory,
}

/// Error types for input operations
#[derive(Debug)]
pub enum InputError {
    Io(io::Error),
    InvalidFormat(String),
    ClipboardError(String),
}

impl From<io::Error> for InputError {
    fn from(err: io::Error) -> Self {
        InputError::Io(err)
    }
}

impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::Io(err) => write!(f, "IO error: {err}"),
            InputError::InvalidFormat(msg) => write!(f, "Invalid format: {msg}"),
            InputError::ClipboardError(msg) => write!(f, "Clipboard error: {msg}"),
        }
    }
}

impl std::error::Error for InputError {}

/// Clipboard manager for handling copy/paste operations
pub struct ClipboardManager {
    clipboard_content: String,
    history: Vec<String>,
    max_history_size: usize,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            clipboard_content: String::new(),
            history: Vec::new(),
            max_history_size: 10,
        }
    }

    pub fn copy(&mut self, text: &str) -> Result<(), InputError> {
        self.clipboard_content = text.to_string();
        
        if !text.is_empty() && self.history.last().map_or(true, |last| last != text) {
            self.history.push(text.to_string());
            if self.history.len() > self.max_history_size {
                self.history.remove(0);
            }
        }
        
        Ok(())
    }

    pub fn paste(&self) -> Result<String, InputError> {
        Ok(self.clipboard_content.clone())
    }

    pub fn get_history(&self) -> &[String] {
        &self.history
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

/// Input widget for handling various input types
pub struct InputWidget {
    cursor_position: usize,
    focused: bool,
    clipboard_manager: ClipboardManager,
    pub clipboard_history_index: usize,
}

impl Default for InputWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl InputWidget {
    pub fn new() -> Self {
        Self {
            cursor_position: 0,
            focused: false,
            clipboard_manager: ClipboardManager::new(),
            clipboard_history_index: 0,
        }
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    pub fn set_cursor_position(&mut self, position: usize) {
        self.cursor_position = position;
    }

    pub fn clipboard_manager(&self) -> &ClipboardManager {
        &self.clipboard_manager
    }

    pub fn clipboard_manager_mut(&mut self) -> &mut ClipboardManager {
        &mut self.clipboard_manager
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) -> IoResult<()> {
        let paragraph = Paragraph::new("Input Widget")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        frame.render_widget(paragraph, area);
        Ok(())
    }
}

/// Validate if a string is a valid URL
pub fn validate_url(url: &str) -> bool {
    if url.is_empty() {
        return false;
    }
    
    url.starts_with("http://") || 
    url.starts_with("https://") || 
    url.starts_with("ftp://") || 
    url.starts_with("ftps://") ||
    url.starts_with("file://")
}

/// Format a hyperlink with OSC 8 escape sequences for terminal support
pub fn format_hyperlink(url: &str, text: &str) -> String {
    if validate_url(url) {
        format!("\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\")
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com"));
        assert!(validate_url("http://example.com"));
        assert!(!validate_url("invalid-url"));
        assert!(!validate_url(""));
    }

    #[test]
    fn test_input_widget_creation() {
        let widget = InputWidget::new();
        assert_eq!(widget.cursor_position, 0);
        assert!(!widget.focused);
    }
}
