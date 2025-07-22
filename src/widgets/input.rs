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
        ("😉", "winking face"),
        ("😊", "smiling face with smiling eyes"),
        ("😋", "face savoring food"),
        ("😎", "smiling face with sunglasses"),
        ("🤗", "hugging face"),
        ("🤔", "thinking face"),
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
        ("🐯", "tiger face"),
        ("🦁", "lion"),
        ("🐸", "frog"),
        ("🐵", "monkey face"),
        ("🐔", "chicken"),
        ("🐧", "penguin"),
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
        ("🥭", "mango"),
        ("🍑", "peach"),
        ("🥑", "avocado"),
        ("🍅", "tomato"),
        ("🥕", "carrot"),
        ("🌽", "ear of corn"),
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
        ("🚐", "minibus"),
        ("🚚", "delivery truck"),
        ("🚛", "articulated lorry"),
        ("🚜", "tractor"),
        ("🏍️", "motorcycle"),
        ("🛵", "motor scooter"),
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
        ("🏸", "badminton"),
        ("🎯", "direct hit"),
        ("🎮", "video game"),
        ("🎲", "game die"),
        ("🎪", "circus tent"),
        ("🎨", "artist palette"),
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
        ("🌼", "daisy"),
        ("🌿", "herb"),
        ("🍀", "four leaf clover"),
        ("🌾", "sheaf of rice"),
        ("🌱", "seedling"),
        ("🌊", "water wave"),
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
        ("✏️", "pencil"),
        ("📝", "memo"),
        ("📎", "paperclip"),
        ("📌", "pushpin"),
        ("📍", "round pushpin"),
        ("🔍", "magnifying glass"),
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
        ("❣️", "heavy heart exclamation"),
        ("💕", "two hearts"),
        ("💖", "sparkling heart"),
        ("✨", "sparkles"),
        ("⭐", "star"),
        ("🌟", "glowing star"),
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

/// Clipboard manager for handling copy/paste operations
pub struct ClipboardManager {
    /// In-memory clipboard storage as fallback
    clipboard_content: String,
    /// Clipboard history
    history: Vec<String>,
    /// Maximum history size
    max_history_size: usize,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardManager {
    /// Create a new clipboard manager
    pub fn new() -> Self {
        Self {
            clipboard_content: String::new(),
            history: Vec::new(),
            max_history_size: 10,
        }
    }

    /// Copy text to clipboard
    pub fn copy(&mut self, text: &str) -> Result<(), InputError> {
        // Try system clipboard first, fall back to internal storage
        match self.copy_to_system_clipboard(text) {
            Ok(_) => {},
            Err(_) => {
                // Fallback to internal clipboard
                self.clipboard_content = text.to_string();
            }
        }
        
        // Add to history if not empty and not duplicate of last entry
        if !text.is_empty() && self.history.last().map_or(true, |last| last != text) {
            self.history.push(text.to_string());
            if self.history.len() > self.max_history_size {
                self.history.remove(0);
            }
        }
        
        Ok(())
    }

    /// Paste text from clipboard
    pub fn paste(&self) -> Result<String, InputError> {
        // Try system clipboard first, fall back to internal storage
        match self.paste_from_system_clipboard() {
            Ok(content) => Ok(content),
            Err(_) => Ok(self.clipboard_content.clone()),
        }
    }

    /// Get clipboard history
    pub fn get_history(&self) -> &[String] {
        &self.history
    }

    /// Clear clipboard history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Copy to system clipboard (fallback implementation)
    fn copy_to_system_clipboard(&self, text: &str) -> Result<(), InputError> {
        // This would typically use a crate like `clipboard` or `arboard`
        // For now, we'll simulate it
        log::info!("Copying to system clipboard: {}", text);
        Ok(())
    }

    /// Paste from system clipboard (fallback implementation)
    fn paste_from_system_clipboard(&self) -> Result<String, InputError> {
        // This would typically use a crate like `clipboard` or `arboard`
        // For now, return empty string to indicate fallback to internal clipboard
        Err(InputError::ClipboardError("System clipboard not available".to_string()))
    }
}

/// Input widget for handling various input types
pub struct InputWidget {
    /// Current cursor position in the input field
    cursor_position: usize,
    /// Whether the widget is in focus
    focused: bool,
    /// Clipboard manager
    clipboard_manager: ClipboardManager,
    /// Selected clipboard history item
    clipboard_history_index: usize,
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
            clipboard_manager: ClipboardManager::new(),
            clipboard_history_index: 0,
        }
    }

    /// Set focus state of the widget
    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Get cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Set cursor position
    pub fn set_cursor_position(&mut self, position: usize) {
        self.cursor_position = position;
    }

    /// Render the input widget
    pub fn render(&self, frame: &mut Frame, area: Rect, app: &App) -> IoResult<()> {
        let primary_color = app.primary_color();
        
        // Process keybindings and emojis for demonstration
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
            ("Ctrl+C", "Copy"),
            ("Ctrl+X", "Cut"),
        ];
        let keybindings: HashMap<&str, &[(&str, &str)]> = HashMap::from([
            ("Input", &keybindings_data[..]),
        ]);

        // Process keybindings
        if let Some(bindings) = keybindings.get("Input") {
            for (key, description) in bindings.iter() {
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
                // Set cursor to end of current field
                let current_text = match app.selected_input {
                    0 => &app.text_input,
                    1 => &app.emoji_input,
                    2 => &app.hyperlink_input,
                    _ => "",
                };
                self.cursor_position = current_text.len();
                Ok(true)
            }
            KeyCode::Char('p') => {
                app.input_mode = InputMode::EmojiPicker;
                app.emoji_category_index = 0;
                app.emoji_index = 0;
                Ok(true)
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.input_mode = InputMode::ClipboardHistory;
                self.clipboard_history_index = 0;
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
            KeyCode::Enter => {
                // If hyperlink field is selected and contains valid URL, handle it
                if app.selected_input == 2 && validate_url(&app.hyperlink_input) {
                    // In a real application, you might open the URL
                    log::info!("Opening URL: {}", app.hyperlink_input);
                }
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
                let current_field = match app.selected_input {
                    0 => &mut app.text_input,
                    1 => &mut app.emoji_input,
                    2 => &mut app.hyperlink_input,
                    _ => return Ok(false),
                };
                
                // Insert character at cursor position
                if self.cursor_position <= current_field.len() {
                    current_field.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                } else {
                    current_field.push(c);
                    self.cursor_position = current_field.len();
                }
                Ok(true)
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    let current_field = match app.selected_input {
                        0 => &mut app.text_input,
                        1 => &mut app.emoji_input,
                        2 => &mut app.hyperlink_input,
                        _ => return Ok(false),
                    };
                    
                    if self.cursor_position <= current_field.len() {
                        current_field.remove(self.cursor_position - 1);
                        self.cursor_position -= 1;
                    }
                }
                Ok(true)
            }
            KeyCode::Delete => {
                let current_field = match app.selected_input {
                    0 => &mut app.text_input,
                    1 => &mut app.emoji_input,
                    2 => &mut app.hyperlink_input,
                    _ => return Ok(false),
                };
                
                if self.cursor_position < current_field.len() {
                    current_field.remove(self.cursor_position);
                }
                Ok(true)
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                Ok(true)
            }
            KeyCode::Right => {
                let current_field = match app.selected_input {
                    0 => &app.text_input,
                    1 => &app.emoji_input,
                    2 => &app.hyperlink_input,
                    _ => return Ok(false),
                };
                
                if self.cursor_position < current_field.len() {
                    self.cursor_position += 1;
                }
                Ok(true)
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                Ok(true)
            }
            KeyCode::End => {
                let current_field = match app.selected_input {
                    0 => &app.text_input,
                    1 => &app.emoji_input,
                    2 => &app.hyperlink_input,
                    _ => return Ok(false),
                };
                self.cursor_position = current_field.len();
                Ok(true)
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let content = match app.selected_input {
                    0 => app.text_input.clone(),
                    1 => app.emoji_input.clone(),
                    2 => app.hyperlink_input.clone(),
                    _ => return Ok(false),
                };
                self.clipboard_manager.copy(&content)?;
                Ok(true)
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let content = self.clipboard_manager.paste()?;
                let current_field = match app.selected_input {
                    0 => &mut app.text_input,
                    1 => &mut app.emoji_input,
                    2 => &mut app.hyperlink_input,
                    _ => return Ok(false),
                };
                
                // Insert at cursor position
                for c in content.chars() {
                    current_field.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                }
                Ok(true)
            }
            KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let content = match app.selected_input {
                    0 => {
                        let content = app.text_input.clone();
                        app.text_input.clear();
                        content
                    }
                    1 => {
                        let content = app.emoji_input.clone();
                        app.emoji_input.clear();
                        content
                    }
                    2 => {
                        let content = app.hyperlink_input.clone();
                        app.hyperlink_input.clear();
                        content
                    }
                    _ => return Ok(false),
                };
                self.clipboard_manager.copy(&content)?;
                self.cursor_position = 0;
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
                } else {
                    app.emoji_index = 0;
                }
                Ok(true)
            }
            KeyCode::Down => {
                if let Some(category) = EMOJI_CATEGORIES.get(app.emoji_category_index) {
                    if let Some((_, emoji_data)) = EMOJI_MAP.iter().find(|(cat, _)| *cat == category) {
                        if app.emoji_index + 5 < emoji_data.len() {
                            app.emoji_index += 5;
                        } else {
                            // Go to last row
                            let last_row_start = (emoji_data.len() - 1) / 5 * 5;
                            app.emoji_index = std::cmp::max(last_row_start, app.emoji_index);
                        }
                    }
                }
                Ok(true)
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
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
            KeyCode::Char(c) if c.is_ascii_digit() => {
                // Quick number selection (0-9)
                if let Some(digit) = c.to_digit(10) {
                    let index = digit as usize;
                    if let Some(category) = EMOJI_CATEGORIES.get(app.emoji_category_index) {
                        if let Some((_, emoji_data)) = EMOJI_MAP.iter().find(|(cat, _)| *cat == category) {
                            if let Some((emoji, _)) = emoji_data.get(index) {
                                app.emoji_input.push_str(emoji);
                                app.input_mode = InputMode::Normal;
                            }
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
        let history = self.clipboard_manager.get_history();
        
        match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                Ok(true)
            }
            KeyCode::Up => {
                if self.clipboard_history_index > 0 {
                    self.clipboard_history_index -= 1;
                }
                Ok(true)
            }
            KeyCode::Down => {
                if !history.is_empty() && self.clipboard_history_index < history.len() - 1 {
                    self.clipboard_history_index += 1;
                }
                Ok(true)
            }
            KeyCode::Enter => {
                if let Some(selected_content) = history.get(self.clipboard_history_index) {
                    let current_field = match app.selected_input {
                        0 => &mut app.text_input,
                        1 => &mut app.emoji_input,
                        2 => &mut app.hyperlink_input,
                        _ => return Ok(false),
                    };
                    current_field.push_str(selected_content);
                    app.input_mode = InputMode::Normal;
                }
                Ok(true)
            }
            KeyCode::Delete => {
                // Clear clipboard history
                self.clipboard_manager.clear_history();
                self.clipboard_history_index = 0;
                Ok(true)
            }
            _ => Ok(false)
        }
    }

    /// Get clipboard manager reference
    pub fn clipboard_manager(&self) -> &ClipboardManager {
        &self.clipboard_manager
    }

    /// Get mutable clipboard manager reference
    pub fn clipboard_manager_mut(&mut self) -> &mut ClipboardManager {
        &mut self.clipboard_manager
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

/// Render input fields for text, emoji, and hyperlink
pub fn render_input(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    input_widget: &InputWidget,
) -> IoResult<()> {
    let selected_input = app.selected_input;
    let text_input = &app.text_input;
    let emoji_input = &app.emoji_input;
    let hyperlink_input = &app.hyperlink_input;
    let primary_color = app.primary_color();
    let cursor_pos = input_widget.cursor_position();
    
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
    let text_title = if matches!(app.input_mode, InputMode::Editing) && selected_input == 0 {
        format!("Text Input (editing - pos: {})", cursor_pos)
    } else {
        "Text Input (press 'e' to edit)".to_string()
    };
    
    let text_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if selected_input == 0 { primary_color } else { Color::White }))
        .title(text_title);
    
    let text_display = if matches!(app.input_mode, InputMode::Editing) && selected_input == 0 {
        // Show cursor in editing mode
        let mut display = text_input.clone();
        if cursor_pos <= display.len() {
            display.insert(cursor_pos, '|');
        }
        display
    } else {
        text_input.clone()
    };
    
    let text = Paragraph::new(text_display)
        .block(text_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(text, chunks[0]);

    // Emoji input
    let emoji_title = if matches!(app.input_mode, InputMode::Editing) && selected_input == 1 {
        format!("Emoji Input (editing - pos: {})", cursor_pos)
    } else {
        "Emoji Input (press 'p' for picker)".to_string()
    };
    
    let emoji_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if selected_input == 1 { primary_color } else { Color::White }))
        .title(emoji_title);
    
    let emoji_display = if matches!(app.input_mode, InputMode::Editing) && selected_input == 1 {
        // Show cursor in editing mode
        let mut display = emoji_input.clone();
        if cursor_pos <= display.len() {
            display.insert(cursor_pos, '|');
        }
        display
    } else {
        emoji_input.clone()
    };
    
    let emoji = Paragraph::new(emoji_display)
        .block(emoji_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(emoji, chunks[1]);

    // Hyperlink input
    let hyperlink_title = if matches!(app.input_mode, InputMode::Editing) && selected_input == 2 {
        format!("Hyperlink (editing - pos: {})", cursor_pos)
    } else {
        "Hyperlink (press Enter to open)".to_string()
    };
    
    let hyperlink_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if selected_input == 2 { primary_color } else { Color::White }))
        .title(hyperlink_title);
    
    let hyperlink_display = if matches!(app.input_mode, InputMode::Editing) && selected_input == 2 {
        // Show cursor in editing mode
        let mut display = hyperlink_input.clone();
        if cursor_pos <= display.len() {
            display.insert(cursor_pos, '|');
        }
        display
    } else {
        hyperlink_input.clone()
    };
    
    let hyperlink_text = if validate_url(&hyperlink_display) {
        format_hyperlink(&hyperlink_display, &hyperlink_display)
    } else {
        hyperlink_display
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
                Span::raw(" for clipboard history, "),
                Span::styled("Enter", Style::default().fg(primary_color)),
                Span::raw(" to open URL"),
            ]),
        ],
        InputMode::Editing => vec![
            Line::from(vec![
                Span::raw("Editing mode: "),
                Span::styled("Enter/Esc", Style::default().fg(primary_color)),
                Span::raw(" to finish, "),
                Span::styled("←/→", Style::default().fg(primary_color)),
                Span::raw(" move cursor"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl+C", Style::default().fg(primary_color)),
                Span::raw(" copy, "),
                Span::styled("Ctrl+V", Style::default().fg(primary_color)),
                Span::raw(" paste, "),
                Span::styled("Ctrl+X", Style::default().fg(primary_color)),
                Span::raw(" cut, "),
                Span::styled("Backspace/Del", Style::default().fg(primary_color)),
                Span::raw(" delete"),
            ]),
        ],
        InputMode::EmojiPicker => vec![
            Line::from(vec![
                Span::raw("Emoji Picker: "),
                Span::styled("←/→", Style::default().fg(primary_color)),
                Span::raw(" categories, "),
                Span::styled("↑/↓", Style::default().fg(primary_color)),
                Span::raw(" emojis, "),
                Span::styled("0-9", Style::default().fg(primary_color)),
                Span::raw(" quick select"),
            ]),
            Line::from(vec![
                Span::styled("Enter/Space", Style::default().fg(primary_color)),
                Span::raw(" to select, "),
                Span::styled("Esc", Style::default().fg(primary_color)),
                Span::raw(" to cancel"),
            ]),
        ],
        InputMode::ClipboardHistory => vec![
            Line::from(vec![
                Span::raw("Clipboard History: "),
                Span::styled("↑/↓", Style::default().fg(primary_color)),
                Span::raw(" to navigate, "),
                Span::styled("Del", Style::default().fg(primary_color)),
                Span::raw(" to clear"),
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
                Span::styled(format!("[{}]", cat), Style::default().fg(Color::Black).bg(primary_color))
            } else {
                Span::styled(format!(" {} ", cat), Style::default().fg(Color::Gray))
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
                let desc_text = format!("{} - {}", emoji, description);
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
    let rows = (emoji_count + cols - 1) / cols; // Ceiling division
    
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
                line.push(Span::styled(format!(" {} ", emoji), style));
                line.push(Span::raw(" ")); // Add spacing between emojis
            } else {
                line.push(Span::raw("     ")); // Empty space
            }
        }
        emoji_grid.push(Line::from(line));
    }
    
    let emoji_list = Paragraph::new(emoji_grid)
        .block(Block::default().borders(Borders::ALL).title("Emojis (0-9 for quick select)"))
        .alignment(Alignment::Left);
    
    frame.render_widget(emoji_list, area);
    Ok(())
}

/// Render clipboard history UI
pub fn render_clipboard_history(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    input_widget: &InputWidget,
) -> IoResult<()> {
    // Clear the area first
    frame.render_widget(Clear, area);
    
    let primary_color = app.primary_color();
    let history = input_widget.clipboard_manager().get_history();
    let selected_index = input_widget.clipboard_history_index;
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(5),     // History items
            Constraint::Length(3),  // Help
        ])
        .split(area);

    // Render title
    let title = Paragraph::new("📋 Clipboard History")
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(primary_color))
            .title("Clipboard History"))
        .alignment(Alignment::Center)
        .style(Style::default().fg(primary_color));
    frame.render_widget(title, chunks[0]);

    // Render history items
    if history.is_empty() {
        let empty_msg = Paragraph::new("No clipboard history available")
            .block(Block::default().borders(Borders::ALL).title("History"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(empty_msg, chunks[1]);
    } else {
        let mut history_lines = Vec::new();
        
        for (i, item) in history.iter().enumerate() {
            let truncated_item = if item.len() > 50 {
                format!("{}...", &item[..47])
            } else {
                item.clone()
            };
            
            let style = if i == selected_index {
                Style::default().fg(Color::Black).bg(primary_color)
            } else {
                Style::default().fg(Color::White)
            };
            
            history_lines.push(Line::from(vec![
                Span::styled(format!(" {} ", i + 1), Style::default().fg(Color::Gray)),
                Span::styled(truncated_item, style),
            ]));
        }
        
        let history_widget = Paragraph::new(history_lines)
            .block(Block::default().borders(Borders::ALL).title("History"))
            .alignment(Alignment::Left);
        frame.render_widget(history_widget, chunks[1]);
    }

    // Render help
    let help_text = vec![
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(primary_color)),
            Span::raw(" navigate, "),
            Span::styled("Enter", Style::default().fg(primary_color)),
            Span::raw(" paste, "),
            Span::styled("Del", Style::default().fg(primary_color)),
            Span::raw(" clear, "),
            Span::styled("Esc", Style::default().fg(primary_color)),
            Span::raw(" cancel"),
        ])
    ];
    
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);

    Ok(())
}

/// Handle keyboard events for input modes
pub fn handle_key_events(app: &mut App, key: KeyEvent, input_widget: &mut InputWidget) -> Result<bool, InputError> {
    input_widget.handle_key_event(key, app)
}

/// Create a centered popup area
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Utility function to get emoji by category and index
pub fn get_emoji_by_index(category_index: usize, emoji_index: usize) -> Option<(&'static str, &'static str)> {
    EMOJI_CATEGORIES
        .get(category_index)
        .and_then(|&category| {
            EMOJI_MAP
                .iter()
                .find(|(cat, _)| *cat == category)
                .and_then(|(_, emojis)| emojis.get(emoji_index))
                .copied()
        })
}

/// Utility function to search emojis by description
pub fn search_emojis(query: &str) -> Vec<(usize, usize, &'static str, &'static str)> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();
    
    for (cat_idx, &category) in EMOJI_CATEGORIES.iter().enumerate() {
        if let Some((_, emojis)) = EMOJI_MAP.iter().find(|(cat, _)| *cat == category) {
            for (emoji_idx, &(emoji, description)) in emojis.iter().enumerate() {
                if description.to_lowercase().contains(&query_lower) {
                    results.push((cat_idx, emoji_idx, emoji, description));
                }
            }
        }
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com"));
        assert!(validate_url("http://example.com"));
        assert!(validate_url("ftp://example.com"));
        assert!(validate_url("ftps://example.com"));
        assert!(validate_url("file:///path/to/file"));
        assert!(!validate_url("invalid-url"));
        assert!(!validate_url(""));
        assert!(!validate_url("example.com"));
    }

    #[test]
    fn test_format_hyperlink() {
        let url = "https://example.com";
        let text = "Example";
        let formatted = format_hyperlink(url, text);
        assert!(formatted.contains(url));
        assert!(formatted.contains(text));
        
        // Test invalid URL
        let invalid_formatted = format_hyperlink("invalid", text);
        assert_eq!(invalid_formatted, text);
    }

    #[test]
    fn test_input_widget_creation() {
        let widget = InputWidget::new();
        assert_eq!(widget.cursor_position, 0);
        assert!(!widget.focused);
        assert!(widget.clipboard_manager().get_history().is_empty());
    }

    #[test]
    fn test_emoji_categories() {
        assert!(!EMOJI_CATEGORIES.is_empty());
        assert!(EMOJI_CATEGORIES.contains(&"smileys"));
        assert!(EMOJI_CATEGORIES.contains(&"animals"));
        assert!(EMOJI_CATEGORIES.contains(&"food"));
        assert!(EMOJI_CATEGORIES.contains(&"travel"));
        assert!(EMOJI_CATEGORIES.contains(&"activities"));
        assert!(EMOJI_CATEGORIES.contains(&"nature"));
        assert!(EMOJI_CATEGORIES.contains(&"objects"));
        assert!(EMOJI_CATEGORIES.contains(&"symbols"));
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
                // Basic check that emoji is actually a unicode character
                assert!(emoji.chars().count() >= 1);
            }
        }
    }

    #[test]
    fn test_get_emoji_by_index() {
        // Test valid indices
        let (emoji, desc) = get_emoji_by_index(0, 0).expect("Should find emoji");
        assert!(!emoji.is_empty());
        assert!(!desc.is_empty());
        
        // Test invalid category index
        assert!(get_emoji_by_index(999, 0).is_none());
        
        // Test invalid emoji index
        assert!(get_emoji_by_index(0, 999).is_none());
    }

    #[test]
    fn test_search_emojis() {
        let results = search_emojis("face");
        assert!(!results.is_empty());
        
        // All results should contain "face" in description
        for (_, _, _, description) in &results {
            assert!(description.to_lowercase().contains("face"));
        }
        
        // Test case insensitive search
        let results_upper = search_emojis("FACE");
        assert_eq!(results.len(), results_upper.len());
        
        // Test empty query
        let empty_results = search_emojis("");
        assert!(!empty_results.is_empty()); // Should return all emojis
    }

    #[test]
    fn test_clipboard_manager() {
        let mut clipboard = ClipboardManager::new();
        
        // Test copy and paste
        clipboard.copy("test content").unwrap();
        let pasted = clipboard.paste().unwrap();
        assert_eq!(pasted, "test content");
        
        // Test history
        clipboard.copy("first").unwrap();
        clipboard.copy("second").unwrap();
        clipboard.copy("third").unwrap();
        
        let history = clipboard.get_history();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0], "first");
        assert_eq!(history[1], "second");
        assert_eq!(history[2], "third");
        
        // Test clear history
        clipboard.clear_history();
        assert!(clipboard.get_history().is_empty());
    }

    #[test]
    fn test_centered_rect() {
        let parent = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(50, 50, parent);
        
        // Should be centered
        assert_eq!(centered.width, 50);
        assert_eq!(centered.height, 50);
        assert_eq!(centered.x, 25);
        assert_eq!(centered.y, 25);
    }
}