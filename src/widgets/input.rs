//! Input widget module providing text input, emoji picker, and hyperlink functionality
//! 
//! This module handles various input types including:
//! - Text input with clipboard support
//! - Emoji picker with categorized selection
//! - Hyperlink input with validation
//! 
//! # Example
//! ```rust
//! use crate::widgets::input::InputWidget;
//! let mut input_widget = InputWidget::new();
//! ```

use std::io::{self, Result as IoResult};
use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Clear};
use ratatui::Frame;

// Import from app module
use crate::app::{App, InputMode};

/// Emoji categories for the picker
const EMOJI_CATEGORIES: &[&str] = &["smileys", "animals", "food", "travel", "activities", "nature", "objects", "symbols"];

/// Emoji data mapping categories to (emoji, description) tuples
const EMOJI_MAP: &[(&str, &[(&str, &str)])] = &[
    ("smileys", &[
        ("😀", "grinning face"),
        ("😃", "grinning face with big eyes"),
        ("😄", "grinning face with smiling eyes"),
        ("😁", "beaming face with smiling eyes"),
        ("😆", "grinning squinting face"),
        ("😊", "smiling face with smiling eyes"),
        ("😇", "smiling face with halo"),
        ("🙂", "slightly smiling face"),
        ("🙃", "upside-down face"),
        ("😉", "winking face")
    ]),
    ("animals", &[
        ("🐶", "dog face"),
        ("🐱", "cat face"),
        ("🐭", "mouse face"),
        ("🐹", "hamster face"),
        ("🐰", "rabbit face"),
        ("🦊", "fox face"),
        ("🐻", "bear face"),
        ("🐼", "panda face"),
        ("🐨", "koala"),
        ("🐯", "tiger face")
    ]),
    ("food", &[
        ("🍎", "red apple"),
        ("🍊", "tangerine"),
        ("🍋", "lemon"),
        ("🍌", "banana"),
        ("🍉", "watermelon"),
        ("🍇", "grapes"),
        ("🍓", "strawberry"),
        ("🥝", "kiwi fruit"),
        ("🍒", "cherries"),
        ("🥭", "mango")
    ]),
    ("travel", &[
        ("🚗", "car"),
        ("🚕", "taxi"),
        ("🚙", "sport utility vehicle"),
        ("🚌", "bus"),
        ("🚎", "trolleybus"),
        ("🏎️", "racing car"),
        ("🚓", "police car"),
        ("🚑", "ambulance"),
        ("🚒", "fire engine"),
        ("🚐", "minibus")
    ]),
    ("activities", &[
        ("⚽", "soccer ball"),
        ("🏀", "basketball"),
        ("🏈", "american football"),
        ("⚾", "baseball"),
        ("🎾", "tennis"),
        ("🏐", "volleyball"),
        ("🏉", "rugby football"),
        ("🎱", "pool 8 ball"),
        ("🏓", "ping pong"),
        ("🏸", "badminton")
    ]),
    ("nature", &[
        ("🌲", "evergreen tree"),
        ("🌳", "deciduous tree"),
        ("🌴", "palm tree"),
        ("🌵", "cactus"),
        ("🌷", "tulip"),
        ("🌸", "cherry blossom"),
        ("🌹", "rose"),
        ("🌺", "hibiscus"),
        ("🌻", "sunflower"),
        ("🌼", "daisy")
    ]),
    ("objects", &[
        ("📱", "mobile phone"),
        ("💻", "laptop computer"),
        ("⌨️", "keyboard"),
        ("🖥️", "desktop computer"),
        ("🖨️", "printer"),
        ("📷", "camera"),
        ("📺", "television"),
        ("⏰", "alarm clock"),
        ("📚", "books"),
        ("✏️", "pencil")
    ]),
    ("symbols", &[
        ("❤️", "red heart"),
        ("💙", "blue heart"),
        ("💚", "green heart"),
        ("💛", "yellow heart"),
        ("🧡", "orange heart"),
        ("💜", "purple heart"),
        ("🖤", "black heart"),
        ("🤍", "white heart"),
        ("💔", "broken heart"),
        ("❣️", "heavy heart exclamation")
    ]),
];

/// Error types for input operations
#[derive(Debug)]
pub enum InputError {
    /// IO error during operation
    Io(io::Error),
    /// Invalid input format
    InvalidFormat(String),
    /// Clipboard operation failed
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

/// Input widget for handling various input types
pub struct InputWidget {
    /// Current cursor position in the input field
    cursor_position: usize,
    /// Whether the widget is in focus
    focused: bool,
}

impl Default for InputWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl InputWidget {
    /// Create a new input widget
    pub fn new() -> Self {
        Self {
            cursor_position: 0,
            focused: false,
        }
    }

    /// Set focus state of the widget
    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Render the input widget
    pub fn render(&self, frame: &mut Frame, area: Rect, app: &App) -> IoResult<()> {
        let primary_color = app.primary_color();
        
        // Demonstrate usage of keybindings and emoji map
        self.process_keybindings()?;
        self.process_emojis()?;

        Ok(())
    }

    /// Process keyboard bindings for demonstration
    fn process_keybindings(&self) -> IoResult<()> {
        let keybindings_data = [
            ("Enter", "Submit"),
            ("Esc", "Cancel"),
            ("Tab", "Next field"),
            ("Shift+Tab", "Previous field"),
            ("Ctrl+V", "Paste"),
        ];
        let keybindings: HashMap<&str, &[(&str, &str)]> = HashMap::from([
            ("Input", &keybindings_data[..]),
        ]);

        // Process keybindings
        if let Some((_, bindings)) = keybindings.iter().find(|(k, _)| **k == "Input") {
            for (key, description) in bindings.iter() {
                // Log or process key bindings
                log::debug!("Key binding: {key} -> {description}");
            }
        }
        Ok(())
    }

    /// Process emoji data for demonstration
    fn process_emojis(&self) -> IoResult<()> {
        if let Some((_, emojis)) = EMOJI_MAP.iter().find(|(category, _)| **category == "smileys") {
            for (emoji, description) in emojis.iter() {
                log::debug!("Emoji: {emoji} -> {description}");
            }
        }
        Ok(())
    }

    /// Handle keyboard events
    pub fn handle_key_event(&mut self, key: KeyEvent, app: &mut App) -> Result<bool, InputError> {
        match app.input_mode {
            InputMode::Normal => self.handle_normal_mode(key, app),
            InputMode::Editing => self.handle_editing_mode(key, app),
            InputMode::EmojiPicker => self.handle_emoji_picker_mode(key, app),
            InputMode::ClipboardHistory => self.handle_clipboard_mode(key, app),
        }
    }

    /// Handle key events in normal mode
    fn handle_normal_mode(&mut self, key: KeyEvent, app: &mut App) -> Result<bool, InputError> {
        match key.code {
            KeyCode::Char('e') => {
                app.input_mode = InputMode::Editing;
                Ok(true)
            }
            KeyCode::Char('p') => {
                app.input_mode = InputMode::EmojiPicker;
                Ok(true)
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.input_mode = InputMode::ClipboardHistory;
                Ok(true)
            }
            KeyCode::Up => {
                if app.selected_input > 0 {
                    app.selected_input -= 1;
                }
                Ok(true)
            }
            KeyCode::Down => {
                if app.selected_input < 2 {
                    app.selected_input += 1;
                }
                Ok(true)
            }
            KeyCode::Tab => {
                app.selected_input = (app.selected_input + 1) % 3;
                Ok(true)
            }
            _ => Ok(false)
        }
    }

    /// Handle key events in editing mode
    fn handle_editing_mode(&mut self, key: KeyEvent, app: &mut App) -> Result<bool, InputError> {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                app.input_mode = InputMode::Normal;
                Ok(true)
            }
            KeyCode::Char(c) => {
                match app.selected_input {
                    0 => app.text_input.push(c),
                    1 => app.emoji_input.push(c),
                    2 => app.hyperlink_input.push(c),
                    _ => {}
                }
                Ok(true)
            }
            KeyCode::Backspace => {
                match app.selected_input {
                    0 => { app.text_input.pop(); }
                    1 => { app.emoji_input.pop(); }
                    2 => { app.hyperlink_input.pop(); }
                    _ => {}
                }
                Ok(true)
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let content = match app.selected_input {
                    0 => app.text_input.clone(),
                    1 => app.emoji_input.clone(),
                    2 => app.hyperlink_input.clone(),
                    _ => return Ok(false),
                };
                app.copy_to_clipboard(&content);
                Ok(true)
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let content = app.paste_from_clipboard();
                match app.selected_input {
                    0 => app.text_input.push_str(&content),
                    1 => app.emoji_input.push_str(&content),
                    2 => app.hyperlink_input.push_str(&content),
                    _ => {}
                }
                Ok(true)
            }
            _ => Ok(false)
        }
    }

    /// Handle key events in emoji picker mode
    fn handle_emoji_picker_mode(&mut self, key: KeyEvent, app: &mut App) -> Result<bool, InputError> {
        match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                Ok(true)
            }
            KeyCode::Left => {
                if app.emoji_category_index > 0 {
                    app.emoji_category_index -= 1;
                    app.emoji_index = 0;
                }
                Ok(true)
            }
            KeyCode::Right => {
                if app.emoji_category_index < EMOJI_CATEGORIES.len() - 1 {
                    app.emoji_category_index += 1;
                    app.emoji_index = 0;
                }
                Ok(true)
            }
            KeyCode::Up => {
                if app.emoji_index >= 5 {
                    app.emoji_index -= 5;
                }
                Ok(true)
            }
            KeyCode::Down => {
                if let Some(category) = EMOJI_CATEGORIES.get(app.emoji_category_index) {
                    if let Some((_, emoji_data)) = EMOJI_MAP.iter().find(|(cat, _)| *cat == category) {
                        if app.emoji_index + 5 < emoji_data.len() {
                            app.emoji_index += 5;
                        }
                    }
                }
                Ok(true)
            }
            KeyCode::Enter => {
                if let Some(category) = EMOJI_CATEGORIES.get(app.emoji_category_index) {
                    if let Some((_, emoji_data)) = EMOJI_MAP.iter().find(|(cat, _)| *cat == category) {
                        if let Some((emoji, _)) = emoji_data.get(app.emoji_index) {
                            app.emoji_input.push_str(emoji);
                            app.input_mode = InputMode::Normal;
                        }
                    }
                }
                Ok(true)
            }
            _ => Ok(false)
        }
    }

    /// Handle key events in clipboard history mode
    fn handle_clipboard_mode(&mut self, key: KeyEvent, app: &mut App) -> Result<bool, InputError> {
        match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                Ok(true)
            }
            _ => Ok(false)
        }
    }
}

/// Validate if a string is a valid URL
pub fn validate_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("ftp://")
}

/// Format a hyperlink with OSC 8 escape sequences for terminal support
pub fn format_hyperlink(url: &str, text: &str) -> String {
    if validate_url(url) {
        format!("\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\")
    } else {
        text.to_string()
    }
}

/// Render input fields for text, emoji, and hyperlink
pub fn render_input(
    frame: &mut Frame,
    area: Rect,
    app: &App,
) -> IoResult<()> {
    let selected_input = app.selected_input;
    let text_input = &app.text_input;
    let emoji_input = &app.emoji_input;
    let hyperlink_input = &app.hyperlink_input;
    let primary_color = app.primary_color();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(5),
        ])
        .split(area);

    // Text input
    let text_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if selected_input == 0 { primary_color } else { Color::White }))
        .title("Text Input (press 'e' to edit)");
    
    let text = Paragraph::new(text_input.as_str())
        .block(text_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(text, chunks[0]);

    // Emoji input
    let emoji_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if selected_input == 1 { primary_color } else { Color::White }))
        .title("Emoji Input (press 'p' for picker)");
    
    let emoji = Paragraph::new(emoji_input.as_str())
        .block(emoji_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(emoji, chunks[1]);

    // Hyperlink input
    let hyperlink_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if selected_input == 2 { primary_color } else { Color::White }))
        .title("Hyperlink (press Enter to open)");
    
    let hyperlink_text = if validate_url(hyperlink_input) {
        format_hyperlink(hyperlink_input, hyperlink_input)
    } else {
        hyperlink_input.clone()
    };
    
    let hyperlink = Paragraph::new(hyperlink_text)
        .block(hyperlink_block)
        .style(Style::default().fg(if validate_url(hyperlink_input) { Color::Blue } else { Color::Red }));
    frame.render_widget(hyperlink, chunks[2]);

    // Dynamic help text based on input mode
    let help_text = get_help_text(app.input_mode, primary_color);
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);

    Ok(())
}

/// Get help text based on current input mode
fn get_help_text(input_mode: InputMode, primary_color: Color) -> Vec<Line<'static>> {
    match input_mode {
        InputMode::Normal => vec![
            Line::from(vec![
                Span::raw("Press "),
                Span::styled("e", Style::default().fg(primary_color)),
                Span::raw(" to edit, "),
                Span::styled("↑/↓", Style::default().fg(primary_color)),
                Span::raw(" to navigate, "),
                Span::styled("Tab", Style::default().fg(primary_color)),
                Span::raw(" to switch fields"),
            ]),
            Line::from(vec![
                Span::raw("Press "),
                Span::styled("p", Style::default().fg(primary_color)),
                Span::raw(" for emoji picker, "),
                Span::styled("Ctrl+C", Style::default().fg(primary_color)),
                Span::raw(" for clipboard history"),
            ]),
        ],
        InputMode::Editing => vec![
            Line::from(vec![
                Span::raw("Editing mode: Press "),
                Span::styled("Enter/Esc", Style::default().fg(primary_color)),
                Span::raw(" to finish"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl+C", Style::default().fg(primary_color)),
                Span::raw(" to copy, "),
                Span::styled("Ctrl+V", Style::default().fg(primary_color)),
                Span::raw(" to paste, "),
                Span::styled("Backspace", Style::default().fg(primary_color)),
                Span::raw(" to delete"),
            ]),
        ],
        InputMode::EmojiPicker => vec![
            Line::from(vec![
                Span::raw("Emoji Picker: "),
                Span::styled("←/→", Style::default().fg(primary_color)),
                Span::raw(" categories, "),
                Span::styled("↑/↓", Style::default().fg(primary_color)),
                Span::raw(" emojis"),
            ]),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(primary_color)),
                Span::raw(" to select, "),
                Span::styled("Esc", Style::default().fg(primary_color)),
                Span::raw(" to cancel"),
            ]),
        ],
        InputMode::ClipboardHistory => vec![
            Line::from(vec![
                Span::raw("Clipboard History: "),
                Span::styled("↑/↓", Style::default().fg(primary_color)),
                Span::raw(" to navigate"),
            ]),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(primary_color)),
                Span::raw(" to paste, "),
                Span::styled("Esc", Style::default().fg(primary_color)),
                Span::raw(" to cancel"),
            ]),
        ],
    }
}

/// Render emoji picker UI with full functionality
pub fn render_emoji_picker(
    frame: &mut Frame,
    area: Rect,
    app: &App,
) -> IoResult<()> {
    // Clear the area first
    frame.render_widget(Clear, area);
    
    let category_index = app.emoji_category_index;
    let emoji_index = app.emoji_index;
    let primary_color = app.primary_color();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Categories
            Constraint::Min(8),     // Emojis grid
            Constraint::Length(3),  // Description
        ])
        .split(area);

    // Render title
    let title = Paragraph::new("🎨 Emoji Picker")
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(primary_color))
            .title("Emoji Picker"))
        .alignment(Alignment::Center)
        .style(Style::default().fg(primary_color));
    frame.render_widget(title, chunks[0]);

    // Render categories
    let category_items: Vec<Span> = EMOJI_CATEGORIES.iter()
        .enumerate()
        .map(|(i, &cat)| {
            if i == category_index {
                Span::styled(format!("[{cat}]"), Style::default().fg(Color::Black).bg(primary_color))
            } else {
                Span::styled(format!(" {cat} "), Style::default().fg(Color::Gray))
            }
        })
        .collect();
    
    let categories = Paragraph::new(Line::from(category_items))
        .block(Block::default().borders(Borders::ALL).title("Categories"))
        .alignment(Alignment::Center);
    frame.render_widget(categories, chunks[1]);

    // Render emojis for selected category
    if let Some(category) = EMOJI_CATEGORIES.get(category_index) {
        if let Some((_, emoji_data)) = EMOJI_MAP.iter().find(|(cat, _)| *cat == category) {
            render_emoji_grid(frame, chunks[2], emoji_data, emoji_index, primary_color)?;
            
            // Show selected emoji description
            if let Some((emoji, description)) = emoji_data.get(emoji_index) {
                let desc_text = format!("{emoji} - {description}");
                let desc = Paragraph::new(desc_text)
                    .block(Block::default().borders(Borders::ALL).title("Selected"))
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(primary_color));
                frame.render_widget(desc, chunks[3]);
            }
        }
    }

    Ok(())
}

/// Render the emoji grid within the picker
fn render_emoji_grid(
    frame: &mut Frame,
    area: Rect,
    emoji_data: &[(&str, &str)],
    selected_index: usize,
    primary_color: Color,
) -> IoResult<()> {
    let emoji_count = emoji_data.len();
    let cols = 5;
    let rows = emoji_count.div_ceil(cols);
    
    let mut emoji_grid = Vec::new();
    
    for row in 0..rows {
        let mut line = Vec::new();
        for col in 0..cols {
            let index = row * cols + col;
            if index < emoji_count {
                let (emoji, _desc) = emoji_data[index];
                let style = if index == selected_index {
                    Style::default().fg(Color::Black).bg(primary_color)
                } else {
                    Style::default().fg(Color::White)
                };
                line.push(Span::styled(format!(" {emoji} "), style));
            } else {
                line.push(Span::raw("   "));
            }
        }
        emoji_grid.push(Line::from(line));
    }
    
    let emoji_list = Paragraph::new(emoji_grid)
        .block(Block::default().borders(Borders::ALL).title("Emojis"))
        .alignment(Alignment::Left);
    
    frame.render_widget(emoji_list, area);
    Ok(())
}

/// Handle keyboard events for input modes
pub fn handle_key_events(app: &mut App, key: KeyEvent) -> Result<bool, InputError> {
    let mut input_widget = InputWidget::new();
    input_widget.handle_key_event(key, app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com"));
        assert!(validate_url("http://example.com"));
        assert!(validate_url("ftp://example.com"));
        assert!(!validate_url("invalid-url"));
        assert!(!validate_url(""));
    }

    #[test]
    fn test_format_hyperlink() {
        let url = "https://example.com";
        let text = "Example";
        let formatted = format_hyperlink(url, text);
        assert!(formatted.contains(url));
        assert!(formatted.contains(text));
    }

    #[test]
    fn test_input_widget_creation() {
        let widget = InputWidget::new();
        assert_eq!(widget.cursor_position, 0);
        assert!(!widget.focused);
    }

    #[test]
    fn test_emoji_categories() {
        assert!(!EMOJI_CATEGORIES.is_empty());
        assert!(EMOJI_CATEGORIES.contains(&"smileys"));
        assert!(EMOJI_CATEGORIES.contains(&"animals"));
    }

    #[test]
    fn test_emoji_map_structure() {
        assert!(!EMOJI_MAP.is_empty());
        
        // Test that each category has emojis
        for (category, emojis) in EMOJI_MAP {
            assert!(!category.is_empty());
            assert!(!emojis.is_empty());
            
            // Test that each emoji has a description
            for (emoji, description) in emojis.iter() {
                assert!(!emoji.is_empty());
                assert!(!description.is_empty());
            }
        }
    }
