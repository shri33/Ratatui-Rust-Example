//! Standalone emoji picker example
//!
//! Demonstrates the emoji picker functionality with categories and navigation.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;

const EMOJI_CATEGORIES: &[&str] = &["smileys", "animals", "food", "travel"];

const EMOJI_MAP: &[(&str, &[(&str, &str)])] = &[
    (
        "smileys",
        &[
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
        ],
    ),
    (
        "animals",
        &[
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
        ],
    ),
    (
        "food",
        &[
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
        ],
    ),
    (
        "travel",
        &[
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
        ],
    ),
];

struct EmojiPickerApp {
    should_quit: bool,
    category_index: usize,
    emoji_index: usize,
    selected_emoji: String,
}

impl EmojiPickerApp {
    fn new() -> Self {
        Self {
            should_quit: false,
            category_index: 0,
            emoji_index: 0,
            selected_emoji: String::new(),
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Left => {
                if self.category_index > 0 {
                    self.category_index -= 1;
                    self.emoji_index = 0;
                }
            }
            KeyCode::Right => {
                if self.category_index < EMOJI_CATEGORIES.len() - 1 {
                    self.category_index += 1;
                    self.emoji_index = 0;
                }
            }
            KeyCode::Up => {
                if self.emoji_index >= 5 {
                    self.emoji_index -= 5;
                }
            }
            KeyCode::Down => {
                if let Some(&category) = EMOJI_CATEGORIES.get(self.category_index) {
                    if let Some((_, emoji_data)) =
                        EMOJI_MAP.iter().find(|(cat, _)| cat == &category)
                    {
                        if self.emoji_index + 5 < emoji_data.len() {
                            self.emoji_index += 5;
                        }
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(&category) = EMOJI_CATEGORIES.get(self.category_index) {
                    if let Some((_, emoji_data)) =
                        EMOJI_MAP.iter().find(|(cat, _)| cat == &category)
                    {
                        if let Some((emoji, _)) = emoji_data.get(self.emoji_index) {
                            self.selected_emoji = emoji.to_string();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Categories
                Constraint::Min(8),    // Emojis
                Constraint::Length(3), // Description
                Constraint::Length(3), // Selected
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new("🎨 Emoji Picker Example")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Categories
        let category_items: Vec<Span> = EMOJI_CATEGORIES
            .iter()
            .enumerate()
            .map(|(i, &cat)| {
                if i == self.category_index {
                    Span::styled(
                        format!("[{cat}]"),
                        Style::default().fg(Color::Black).bg(Color::Cyan),
                    )
                } else {
                    Span::styled(format!(" {cat} "), Style::default().fg(Color::Gray))
                }
            })
            .collect();

        let categories = Paragraph::new(Line::from(category_items))
            .block(Block::default().borders(Borders::ALL).title("Categories"))
            .alignment(Alignment::Center);
        frame.render_widget(categories, chunks[1]);

        // Emojis grid
        if let Some(&category) = EMOJI_CATEGORIES.get(self.category_index) {
            if let Some((_, emoji_data)) = EMOJI_MAP.iter().find(|(cat, _)| cat == &category) {
                let emoji_count = emoji_data.len();
                let cols = 5;
                let rows = (emoji_count + cols - 1) / cols;

                let mut emoji_grid = Vec::new();

                for row in 0..rows {
                    let mut line = Vec::new();
                    for col in 0..cols {
                        let index = row * cols + col;
                        if index < emoji_count {
                            let (emoji, _) = emoji_data[index];
                            let style = if index == self.emoji_index {
                                Style::default().fg(Color::Black).bg(Color::Cyan)
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
                frame.render_widget(emoji_list, chunks[2]);

                // Description
                if let Some((emoji, description)) = emoji_data.get(self.emoji_index) {
                    let desc_text = format!("{emoji} - {description}");
                    let desc = Paragraph::new(desc_text)
                        .block(Block::default().borders(Borders::ALL).title("Current"))
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Yellow));
                    frame.render_widget(desc, chunks[3]);
                }
            }
        }

        // Selected emoji
        let selected_text = if self.selected_emoji.is_empty() {
            "No emoji selected. Press Enter to select.".to_string()
        } else {
            format!("Selected: {}", self.selected_emoji)
        };

        let selected = Paragraph::new(selected_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Selected Emoji"),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Green));
        frame.render_widget(selected, chunks[4]);
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
    let mut app = EmojiPickerApp::new();

    // Main loop
    loop {
        terminal.draw(|f| app.render(f))?;

        if let Event::Key(key) = event::read()? {
            app.on_key(key.code);
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

    println!("Final selected emoji: {}", app.selected_emoji);

    Ok(())
}
