// main.rs - Entry point for the Ratatui & Rust Example application
// This file contains the main event loop and UI logic for the demo app.

use std::io::{self, Write};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell},
    Terminal,
};
use arboard::Clipboard;
use open;
use std::fs;

/// Application modes
enum Mode {
    Table,
    Image,
    Input,
}

/// Main application state
struct App {
    mode: Mode,
    status: String,
    input: String,
    clipboard: Clipboard,
    emoji_picker: Vec<&'static str>,
    emoji_index: usize,
    link: String,
}

impl App {
    /// Create a new App instance
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            mode: Mode::Table,
            status: "Press 1:Table 2:Image 3:Input | q:Quit".to_string(),
            input: String::new(),
            clipboard: Clipboard::new()?,
            emoji_picker: vec!["😀", "😂", "🥳", "🚀", "❤️", "👍", "🔥", "🎉"],
            emoji_index: 0,
            link: "https://ratatui.rs".to_string(),
        })
    }

    /// Handle key events and update app state
    fn on_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Table => match key.code {
                KeyCode::Char('2') => self.mode = Mode::Image,
                KeyCode::Char('3') => self.mode = Mode::Input,
                KeyCode::Char('q') => self.status = "quit".to_string(),
                _ => {}
            },
            Mode::Image => match key.code {
                KeyCode::Char('1') => self.mode = Mode::Table,
                KeyCode::Char('3') => self.mode = Mode::Input,
                KeyCode::Char('h') => {
                    if let Err(e) = display_high_res_image("sample.jpg") {
                        self.status = format!("Error: {e}");
                    } else {
                        self.status = "Returned from high-res image view".to_string();
                    }
                }
                KeyCode::Char('q') => self.status = "quit".to_string(),
                _ => {}
            },
            Mode::Input => match key.code {
                KeyCode::Char('1') => self.mode = Mode::Table,
                KeyCode::Char('2') => self.mode = Mode::Image,
                KeyCode::Char('q') => self.status = "quit".to_string(),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let _ = self.clipboard.set_text(self.input.clone());
                    self.status = "Copied to clipboard!".to_string();
                }
                KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Ok(text) = self.clipboard.get_text() {
                        self.input.push_str(&text);
                        self.status = "Pasted from clipboard!".to_string();
                    }
                }
                KeyCode::Char('e') => {
                    // Insert emoji
                    self.input.push_str(self.emoji_picker[self.emoji_index]);
                    self.emoji_index = (self.emoji_index + 1) % self.emoji_picker.len();
                }
                KeyCode::Char('l') => {
                    // Open link
                    if let Err(e) = open::that(&self.link) {
                        self.status = format!("Failed to open link: {e}");
                    } else {
                        self.status = "Opened link in browser".to_string();
                    }
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                _ => {}
            },
        }
    }
}

/// Helper to display a high-res image using viuer
fn display_high_res_image(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !fs::metadata(path).is_ok() {
        println!("Image file '{}' not found. Please provide a valid image.", path);
        return Ok(());
    }
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    let config = viuer::Config { ..Default::default() };
    viuer::print_from_file(path, &config)?;
    print!("\nPress Enter to return to the TUI...");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    Ok(())
}

/// Main entry point
fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new()?;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(size);

            // Title
            let title = Paragraph::new("🦀 Ratatui All-in-One Demo")
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(title, chunks[0]);

            match app.mode {
                Mode::Table => {
                    let rows = vec![
                        Row::new(vec!["Name", "Type", "Supports"]).style(Style::default().add_modifier(Modifier::BOLD)),
                        Row::new(vec!["Table", "Widget", "Sorting, Selection"]),
                        Row::new(vec!["Image", "High-Res", "Kitty/Sixel"]),
                        Row::new(vec!["Input", "Text/Emoji", "Clipboard, Links"]),
                    ];
                    let column_widths = [
                        Constraint::Length(10),
                        Constraint::Length(15),
                        Constraint::Length(25),
                    ];
                    let table = Table::new(rows, &column_widths)
                        .block(Block::default().borders(Borders::ALL).title("Feature Table"));
                    f.render_widget(table, chunks[1]);
                }
                Mode::Image => {
                    let content = Paragraph::new("Press 'h' to show the image in high resolution using viuer.\n\nMake sure your terminal supports Kitty or Sixel graphics protocols for best results.\n\nPress 1:Table 3:Input q:Quit")
                        .block(Block::default().borders(Borders::ALL).title("Image Mode"));
                    f.render_widget(content, chunks[1]);
                }
                Mode::Input => {
                    let mut text = Text::from(vec![
                        Line::from(vec![
                            Span::styled("Input: ", Style::default().fg(Color::Green)),
                            Span::raw(&app.input),
                        ]),
                        Line::from(vec![
                            Span::styled("Emoji: ", Style::default().fg(Color::Magenta)),
                            Span::raw(app.emoji_picker[app.emoji_index]),
                        ]),
                        Line::from(vec![
                            Span::styled("Link: ", Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED)),
                            Span::raw(&app.link),
                            Span::raw(" (press 'l' to open)"),
                        ]),
                        Line::from("Ctrl+C:Copy  Ctrl+V:Paste  e:Insert Emoji  1:Table 2:Image q:Quit"),
                    ]);
                    let input_box = Paragraph::new(text)
                        .block(Block::default().borders(Borders::ALL).title("Input Mode"));
                    f.render_widget(input_box, chunks[1]);
                }
            }

            let status = Paragraph::new(app.status.as_str())
                .block(Block::default().borders(Borders::ALL).title("Status"))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(status, chunks[2]);
        })?;

        if app.status == "quit" {
            break;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key);
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}