//! Terminal hyperlinks example
//!
//! Demonstrates clickable hyperlinks in the terminal using OSC 8 escape sequences.

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

// OSC 8 sequence for hyperlinks
fn hyperlink(text: &str, url: &str) -> String {
    format!("\x1B]8;;{}\x1B\\{}\x1B]8;;\x1B\\", url, text)
}

struct HyperlinkApp {
    should_quit: bool,
    selected_link: usize,
    links: Vec<(String, String)>, // (Text, URL)
    status_message: String,
}

impl HyperlinkApp {
    fn new() -> Self {
        Self {
            should_quit: false,
            selected_link: 0,
            links: vec![
                (
                    "Ratatui GitHub".to_string(),
                    "https://github.com/ratatui-org/ratatui".to_string(),
                ),
                (
                    "Rust Homepage".to_string(),
                    "https://www.rust-lang.org".to_string(),
                ),
                ("Crates.io".to_string(), "https://crates.io".to_string()),
                (
                    "Rust Docs".to_string(),
                    "https://doc.rust-lang.org".to_string(),
                ),
                (
                    "This Project".to_string(),
                    "https://github.com/yourusername/ratatui-rust-example".to_string(),
                ),
            ],
            status_message: String::from("Use ↑/↓ to select, Enter to open in browser, q to quit"),
        }
    }

    fn next(&mut self) {
        if self.selected_link < self.links.len() - 1 {
            self.selected_link += 1;
        } else {
            self.selected_link = 0;
        }
    }

    fn previous(&mut self) {
        if self.selected_link > 0 {
            self.selected_link -= 1;
        } else {
            self.selected_link = self.links.len() - 1;
        }
    }

    fn open_selected_link(&mut self) {
        if let Some((text, url)) = self.links.get(self.selected_link) {
            self.status_message = format!("Opening: {} ({})", text, url);

            // On Unix-like systems, we could use:
            // std::process::Command::new("xdg-open").arg(url).spawn().ok();

            // On Windows:
            #[cfg(target_os = "windows")]
            {
                std::process::Command::new("cmd")
                    .args(&["/C", "start", "", &url])
                    .spawn()
                    .ok();
            }

            // On macOS:
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open").arg(&url).spawn().ok();
            }

            // On other Unix-like systems:
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                std::process::Command::new("xdg-open")
                    .arg(&url)
                    .spawn()
                    .ok();
            }
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Down => {
                self.next();
            }
            KeyCode::Up => {
                self.previous();
            }
            KeyCode::Enter => {
                self.open_selected_link();
            }
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(8), // Instructions
                Constraint::Min(0),    // Links
                Constraint::Length(3), // Status
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new("🔗 Terminal Hyperlinks Example")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Instructions
        let instructions_text = Text::from(vec![
            Line::from("This example demonstrates clickable hyperlinks in the terminal."),
            Line::from(""),
            Line::from(
                "If your terminal supports OSC 8 sequences (like iTerm2, Windows Terminal, etc.),",
            ),
            Line::from("you should be able to click on the links directly."),
            Line::from(""),
            Line::from(
                "You can also use ↑/↓ to select a link and Enter to open it in your browser.",
            ),
        ]);

        let instructions = Paragraph::new(instructions_text)
            .block(Block::default().borders(Borders::ALL).title("Instructions"))
            .style(Style::default().fg(Color::White));
        frame.render_widget(instructions, chunks[1]);

        // Links
        let items: Vec<ListItem> = self
            .links
            .iter()
            .enumerate()
            .map(|(i, (text, url))| {
                let style = if i == self.selected_link {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED)
                };

                let content = format!("➤ {}", text);
                let hyperlinked_text = hyperlink(&content, url);

                ListItem::new(hyperlinked_text).style(style)
            })
            .collect();

        let links_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Clickable Links"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        frame.render_widget(links_list, chunks[2]);

        // Status bar
        let status = Paragraph::new(self.status_message.as_str())
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(status, chunks[3]);
    }

    #[allow(dead_code)]    #[allow(dead_code)]
    fn validate_email(&mut self) {
        // More comprehensive email validation
        let email_regex = regex::Regex::new(
            r"^([a-z0-9_+]([a-z0-9_\-\.])+[a-z0-9_]+@([a-z0-9_\-\.])+\.[a-z]{2,5}$",
        )
        .unwrap();

        for (text, _url) in &self.links {
            if email_regex.is_match(text) {
                self.status_message = format!("Valid email: {}", text);
            } else {
                self.status_message = format!("Invalid email: {}", text);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = HyperlinkApp::new();

    loop {
        terminal.draw(|f| app.render(f))?;

        // Fix: Use proper event polling
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key.code);
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
