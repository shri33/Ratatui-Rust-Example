//! Text input example with clickable links and URL detection
//!
//! Demonstrates text input fields with link detection and interaction.

use arboard::Clipboard;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use regex::Regex;
use std::io;
use std::process::Command;

#[derive(Clone, Debug)]
struct Link {
    text: String,
    url: String,
    start: usize,
    end: usize,
}

#[derive(Clone)]
struct TextField {
    content: String,
    cursor_position: usize,
    label: String,
    is_focused: bool,
    links: Vec<Link>,
}

impl TextField {
    fn new(label: &str) -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            label: label.to_string(),
            is_focused: false,
            links: Vec::new(),
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
        self.detect_links();
    }

    fn delete_char(&mut self) {
        if self.cursor_position < self.content.len() {
            self.content.remove(self.cursor_position);
            self.detect_links();
        }
    }

    fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.content.remove(self.cursor_position);
            self.detect_links();
        }
    }

    fn detect_links(&mut self) {
        self.links.clear();
        
        // URL regex pattern
        let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
        
        for mat in url_regex.find_iter(&self.content) {
            self.links.push(Link {
                text: mat.as_str().to_string(),
                url: mat.as_str().to_string(),
                start: mat.start(),
                end: mat.end(),
            });
        }

        // Email regex pattern
        let email_regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        
        for mat in email_regex.find_iter(&self.content) {
            self.links.push(Link {
                text: mat.as_str().to_string(),
                url: format!("mailto:{}", mat.as_str()),
                start: mat.start(),
                end: mat.end(),
            });
        }

        // Sort links by start position
        self.links.sort_by_key(|link| link.start);
    }

    fn get_link_at_position(&self, pos: usize) -> Option<&Link> {
        self.links.iter().find(|link| pos >= link.start && pos < link.end)
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
        let inner_area = block.inner(area);

        // Create styled content with links highlighted
        let mut spans = Vec::new();
        let mut last_end = 0;

        for link in &self.links {
            // Add text before the link
            if link.start > last_end {
                let text = &self.content[last_end..link.start];
                spans.push(Span::raw(text));
            }

            // Add the link with special styling
            spans.push(Span::styled(
                &link.text,
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::UNDERLINED)
                    .add_modifier(Modifier::BOLD),
            ));

            last_end = link.end;
        }

        // Add remaining text after the last link
        if last_end < self.content.len() {
            spans.push(Span::raw(&self.content[last_end..]));
        }

        // Add cursor if focused
        if self.is_focused {
            // Find where to insert cursor
            let cursor_inserted = false;
            let mut new_spans = Vec::new();
            let mut pos = 0;

            for span in &spans {
                let span_text = span.content.as_ref();
                let span_len = span_text.chars().count();
                
                if pos + span_len > self.cursor_position && !cursor_inserted {
                    // Split the span at cursor position
                    let rel_pos = self.cursor_position - pos;
                    let before: String = span_text.chars().take(rel_pos).collect();
                    let at_cursor = span_text.chars().nth(rel_pos).unwrap_or(' ');
                    let after: String = span_text.chars().skip(rel_pos + 1).collect();

                    if !before.is_empty() {
                        new_spans.push(Span::styled(before, span.style));
                    }
                    
                    new_spans.push(Span::styled(
                        at_cursor.to_string(),
                        Style::default().fg(Color::Black).bg(Color::White),
                    ));
                    
                    if !after.is_empty() {
                        new_spans.push(Span::styled(after, span.style));
                    }
                } else {
                    new_spans.push(span.clone());
                }
                
                pos += span_len;
            }

            // If cursor is at the end
            if self.cursor_position >= self.content.chars().count() {
                new_spans.push(Span::styled(
                    " ",
                    Style::default().fg(Color::Black).bg(Color::White),
                ));
            }

            spans = new_spans;
        }

        let line = Line::from(spans);
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
    detected_links: Vec<String>,
}

impl InputApp {
    fn new() -> Self {
        let mut fields = vec![
            TextField::new("Username"),
            TextField::new("Email"),
            TextField::new("Website (try: https://example.com)"),
            TextField::new("Notes (try adding URLs or emails)"),
        ];
        fields[0].is_focused = true;

        Self {
            fields,
            active_field: 0,
            should_quit: false,
            status_message: String::from(
                "Tab to switch fields, Ctrl+L to open links, Ctrl+C/V clipboard, q to quit"
            ),
            detected_links: Vec::new(),
        }
    }

    fn next_field(&mut self) {
        self.fields[self.active_field].is_focused = false;
        self.active_field = (self.active_field + 1) % self.fields.len();
        self.fields[self.active_field].is_focused = true;
        self.update_detected_links();
    }

    fn previous_field(&mut self) {
        self.fields[self.active_field].is_focused = false;
        if self.active_field > 0 {
            self.active_field -= 1;
        } else {
            self.active_field = self.fields.len() - 1;
        }
        self.fields[self.active_field].is_focused = true;
        self.update_detected_links();
    }

    fn update_detected_links(&mut self) {
        self.detected_links.clear();
        for field in &self.fields {
            for link in &field.links {
                self.detected_links.push(format!("{}: {}", field.label, link.url));
            }
        }
    }

    fn open_links(&mut self) {
        let field = &self.fields[self.active_field];
        if field.links.is_empty() {
            self.status_message = "No links found in current field".to_string();
            return;
        }

        let link = &field.links[0]; // Open first link found
        
        #[cfg(target_os = "windows")]
        let result = Command::new("cmd").args(&["/c", "start", &link.url]).spawn();
        
        #[cfg(target_os = "macos")]
        let result = Command::new("open").arg(&link.url).spawn();
        
        #[cfg(target_os = "linux")]
        let result = Command::new("xdg-open").arg(&link.url).spawn();

        match result {
            Ok(_) => self.status_message = format!("Opening: {}", link.url),
            Err(e) => self.status_message = format!("Failed to open link: {}", e),
        }
    }

    fn handle_mouse_click(&mut self, x: u16, y: u16) {
        // Simple click handling - in a real implementation you'd need to map
        // screen coordinates to text positions more accurately
        self.status_message = format!("Mouse clicked at ({}, {})", x, y);
        
        // For demonstration, try to activate the field that was clicked
        if y >= 4 && y <= 6 && self.fields.len() > 0 {
            self.fields[self.active_field].is_focused = false;
            self.active_field = 0;
            self.fields[self.active_field].is_focused = true;
        } else if y >= 7 && y <= 9 && self.fields.len() > 1 {
            self.fields[self.active_field].is_focused = false;
            self.active_field = 1;
            self.fields[self.active_field].is_focused = true;
        }
        // Add more field mappings as needed
    }

    fn submit(&mut self) {
        let mut values = Vec::new();
        for field in &self.fields {
            values.push(format!("{}: {}", field.label, field.content));
        }
        self.status_message = format!("Submitted: {}", values.join(", "));
        self.update_detected_links();
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
            (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                self.open_links();
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.fields[self.active_field].copy_to_clipboard()?;
                self.status_message = "Copied to clipboard".to_string();
            }
            (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                self.fields[self.active_field].paste_from_clipboard()?;
                self.status_message = "Pasted from clipboard".to_string();
                self.update_detected_links();
            }
            (KeyCode::Char(c), _) => {
                self.fields[self.active_field].insert_char(c);
                self.update_detected_links();
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
                self.update_detected_links();
            }
            (KeyCode::Delete, _) => {
                self.fields[self.active_field].delete_char();
                self.update_detected_links();
            }
            _ => {}
        }
        Ok(())
    }

    fn on_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_mouse_click(mouse.column, mouse.row);
            }
            _ => {}
        }
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
                Constraint::Min(3),    // Links panel
                Constraint::Length(3), // Status
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new("🔗 Text Input with Clickable Links")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Render fields
        for (i, field) in self.fields.iter().enumerate() {
            field.render(frame, chunks[i + 1]);
        }

        // Links panel
        let link_items: Vec<ListItem> = self.detected_links
            .iter()
            .map(|link| ListItem::new(link.as_str()))
            .collect();

        let links_list = List::new(link_items)
            .block(Block::default().borders(Borders::ALL).title("Detected Links"))
            .style(Style::default().fg(Color::Green));
        frame.render_widget(links_list, chunks[5]);

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

        match event::read()? {
            Event::Key(key) => {
                if let Err(e) = app.on_key(key) {
                    app.status_message = format!("Error: {}", e);
                }
            }
            Event::Mouse(mouse) => {
                app.on_mouse(mouse);
            }
            _ => {}
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