//! Text input example with clipboard support
//!
//! Demonstrates text input fields with editing capabilities.

use arboard::Clipboard;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;

#[derive(Clone)]
struct TextField {
    content: String,
    cursor_position: usize,
    label: String,
    is_focused: bool,
}

impl TextField {
    fn new(label: &str) -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            label: label.to_string(),
            is_focused: false,
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.content.len() {
            self.cursor_position += 1;
        }
    }

    fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.content.len();
    }

    fn insert_char(&mut self, c: char) {
        self.content.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor_position < self.content.len() {
            self.content.remove(self.cursor_position);
        }
    }

    fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.content.remove(self.cursor_position);
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.label.clone());

        let block_style = if self.is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = block.style(block_style);

        // Calculate visible content
        let inner_area = block.inner(area);
        let width = inner_area.width as usize;

        // Create visible text with cursor
        let mut display_content = self.content.clone();

        // Ensure we display the part with the cursor if content is longer than width
        let content_len = display_content.chars().count();
        let start_pos = if width > 0 && content_len > width - 1 {
            let start = self.cursor_position.saturating_sub(width / 2);
            let end = start + width - 1;
            if end >= content_len {
                content_len.saturating_sub(width - 1)
            } else {
                start
            }
        } else {
            0
        };

        // Slice the content for display
        if start_pos > 0 {
            display_content = display_content
                .chars()
                .skip(start_pos)
                .take(width - 1)
                .collect();
        } else if content_len > width - 1 {
            display_content = display_content.chars().take(width - 1).collect();
        }

        // Calculate cursor position in the visible area
        let cursor_pos_visible = self.cursor_position.saturating_sub(start_pos);

        // Create line with cursor
        let mut line_content = Vec::new();
        for (i, c) in display_content.chars().enumerate() {
            if i == cursor_pos_visible && self.is_focused {
                line_content.push(Span::styled(
                    c.to_string(),
                    Style::default().fg(Color::Black).bg(Color::White),
                ));
            } else {
                line_content.push(Span::raw(c.to_string()));
            }
        }

        // Add cursor at the end if needed
        if cursor_pos_visible == display_content.len() && self.is_focused {
            line_content.push(Span::styled(
                " ",
                Style::default().fg(Color::Black).bg(Color::White),
            ));
        }

        let line = Line::from(line_content);
        let text = Paragraph::new(line).style(Style::default().fg(Color::White));

        frame.render_widget(block, area);
        frame.render_widget(text, inner_area);
    }

    fn paste_from_clipboard(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut clipboard = Clipboard::new()?;
        if let Ok(text) = clipboard.get_text() {
            for c in text.chars() {
                self.insert_char(c);
            }
        }
        Ok(())
    }

    fn copy_to_clipboard(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(self.content.clone())?;
        Ok(())
    }
}

struct InputApp {
    fields: Vec<TextField>,
    active_field: usize,
    should_quit: bool,
    status_message: String,
}

impl InputApp {
    fn new() -> Self {
        let mut fields = vec![
            TextField::new("Username"),
            TextField::new("Email"),
            TextField::new("Password"),
            TextField::new("Notes"),
        ];
        fields[0].is_focused = true;

        Self {
            fields,
            active_field: 0,
            should_quit: false,
            status_message: String::from(
                "Tab to switch fields, Enter to submit, Ctrl+C/Ctrl+V for clipboard, q to quit",
            ),
        }
    }

    fn next_field(&mut self) {
        self.fields[self.active_field].is_focused = false;
        self.active_field = (self.active_field + 1) % self.fields.len();
        self.fields[self.active_field].is_focused = true;
    }

    fn previous_field(&mut self) {
        self.fields[self.active_field].is_focused = false;
        if self.active_field > 0 {
            self.active_field -= 1;
        } else {
            self.active_field = self.fields.len() - 1;
        }
        self.fields[self.active_field].is_focused = true;
    }

    fn submit(&mut self) {
        let mut values = Vec::new();
        for field in &self.fields {
            values.push(format!("{}: {}", field.label, field.content));
        }
        self.status_message = format!("Submitted: {}", values.join(", "));
    }

    fn on_key(&mut self, key: event::KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), _) => {
                self.should_quit = true;
            }
            (KeyCode::Tab, _) => {
                self.next_field();
            }
            (KeyCode::BackTab, _) => {
                self.previous_field();
            }
            (KeyCode::Enter, _) => {
                self.submit();
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.fields[self.active_field].copy_to_clipboard()?;
                self.status_message = "Copied to clipboard".to_string();
            }
            (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                self.fields[self.active_field].paste_from_clipboard()?;
                self.status_message = "Pasted from clipboard".to_string();
            }
            (KeyCode::Char(c), _) => {
                self.fields[self.active_field].insert_char(c);
            }
            (KeyCode::Left, _) => {
                self.fields[self.active_field].move_cursor_left();
            }
            (KeyCode::Right, _) => {
                self.fields[self.active_field].move_cursor_right();
            }
            (KeyCode::Home, _) => {
                self.fields[self.active_field].move_cursor_to_start();
            }
            (KeyCode::End, _) => {
                self.fields[self.active_field].move_cursor_to_end();
            }
            (KeyCode::Backspace, _) => {
                self.fields[self.active_field].backspace();
            }
            (KeyCode::Delete, _) => {
                self.fields[self.active_field].delete_char();
            }
            _ => {}
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Field 1
                Constraint::Length(3), // Field 2
                Constraint::Length(3), // Field 3
                Constraint::Length(6), // Field 4 (notes, taller)
                Constraint::Min(0),    // Spacing
                Constraint::Length(3), // Status
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new("✏️ Text Input Example with Clipboard Support")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Render fields
        for (i, field) in self.fields.iter().enumerate() {
            field.render(frame, chunks[i + 1]);
        }

        // Status bar
        let status = Paragraph::new(self.status_message.as_str())
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(status, chunks[6]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = InputApp::new();

    // Main loop
    loop {
        terminal.draw(|f| app.render(f))?;

        if let Event::Key(key) = event::read()? {
            if let Err(e) = app.on_key(key) {
                app.status_message = format!("Error: {}", e);
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
