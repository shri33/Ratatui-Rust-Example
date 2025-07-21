//! Standalone image viewer example
//!
//! This example demonstrates the image rendering capabilities
//! including MP4 video frame extraction and ASCII art conversion.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use image::GenericImageView;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;

struct ImageViewerApp {
    should_quit: bool,
    status_message: String,
    current_image_path: Option<String>,
    ascii_image: Vec<String>, // Store ASCII representation
}

impl ImageViewerApp {
    fn new() -> Self {
        Self {
            should_quit: false,
            status_message: "Press 'o' to open image, 'v' for video, 'q' to quit".to_string(),
            current_image_path: None,
            ascii_image: Vec::new(),
        }
    }

    fn load_image(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let img = image::open(path)?;
        self.current_image_path = Some(path.to_string());

        // Convert to ASCII art (simplified example)
        let (width, height) = img.dimensions();
        let img = img.resize(80, 40, image::imageops::FilterType::Nearest);

        // Create ASCII representation
        self.ascii_image = Vec::new();
        // Simple ASCII conversion (real implementation would be more sophisticated)
        for y in 0..img.height() {
            let mut line = String::new();
            for x in 0..img.width() {
                let pixel = img.get_pixel(x, y);
                let brightness =
                    0.3 * pixel[0] as f32 + 0.59 * pixel[1] as f32 + 0.11 * pixel[2] as f32;
                let ascii_char = match brightness as u8 {
                    0..=50 => ' ',
                    51..=100 => '.',
                    101..=150 => '*',
                    151..=200 => '#',
                    _ => '@',
                };
                line.push(ascii_char);
            }
            self.ascii_image.push(line);
        }

        self.status_message = format!("Loaded image: {} ({}x{})", path, width, height);
        Ok(())
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('o') => {
                // In a real app, you would open a file dialog here
                // For this example, we'll just hardcode a path
                if let Err(e) = self.load_image("sample.jpg") {
                    self.status_message = format!("Error loading image: {}", e);
                }
            }
            KeyCode::Char('v') => {
                self.status_message = "Video loading would be implemented here...".to_string();
            }
            KeyCode::Esc => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Status
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new("🖼️ Image Viewer Example")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Content area
        if self.ascii_image.is_empty() {
            // No image loaded yet - show placeholder
            let content_lines = vec![
                Line::from(""),
                Line::from("  ╭─────────────────────────────────────╮"),
                Line::from("  │                                     │"),
                Line::from("  │         No Image Loaded             │"),
                Line::from("  │                                     │"),
                Line::from("  │    Press 'o' to load an image       │"),
                Line::from("  │    Press 'v' to load a video        │"),
                Line::from("  │                                     │"),
                Line::from("  │         ASCII art will              │"),
                Line::from("  │         appear here                 │"),
                Line::from("  │                                     │"),
                Line::from("  ╰─────────────────────────────────────╯"),
                Line::from(""),
            ];

            let content = Paragraph::new(content_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Image Display"),
                )
                .style(Style::default().fg(Color::White));
            frame.render_widget(content, chunks[1]);
        } else {
            // Show ASCII image
            let content_lines: Vec<Line> = self
                .ascii_image
                .iter()
                .map(|line| Line::from(line.clone()))
                .collect();

            let content = Paragraph::new(content_lines)
                .block(Block::default().borders(Borders::ALL).title("ASCII Image"))
                .style(Style::default().fg(Color::White));
            frame.render_widget(content, chunks[1]);
        }

        // Status bar
        let status = Paragraph::new(self.status_message.as_str())
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(status, chunks[2]);
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
    let mut app = ImageViewerApp::new();

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

    Ok(())
}
