//! ASCII art image renderer
//!
//! A simplified image viewer that works without FFmpeg dependencies

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
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;

struct ImageViewerApp {
    image_path: String,
    ascii_art: String,
    scale: f32,
    should_quit: bool,
}

impl ImageViewerApp {
    fn new(image_path: &str) -> Self {
        Self {
            image_path: image_path.to_string(),
            ascii_art: String::new(),
            scale: 0.1,
            should_quit: false,
        }
    }

    fn load_image(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Load the image
        let img = image::open(&self.image_path)?;

        // Get image dimensions
        let (width, height) = img.dimensions();

        // Calculate new dimensions based on scale
        let new_width = (width as f32 * self.scale) as u32;
        let new_height = (height as f32 * self.scale) as u32;

        // Resize image
        let resized = img.resize_exact(new_width, new_height, image::imageops::FilterType::Nearest);

        // Convert to grayscale
        let grayscale = resized.grayscale();

        // Generate ASCII art
        let ascii_chars = " .:-=+*#%@";
        let mut ascii = String::new();

        for y in 0..new_height {
            for x in 0..new_width {
                let pixel = grayscale.get_pixel(x, y);
                let intensity = pixel[0] as usize;
                let index = intensity * (ascii_chars.len() - 1) / 255;
                ascii.push(ascii_chars.chars().nth(index).unwrap_or(' '));
            }
            ascii.push('\n');
        }

        self.ascii_art = ascii;
        Ok(())
    }

    fn increase_scale(&mut self) {
        self.scale *= 1.1;
        if let Err(e) = self.load_image() {
            eprintln!("Error: {}", e);
        }
    }

    fn decrease_scale(&mut self) {
        self.scale *= 0.9;
        if let Err(e) = self.load_image() {
            eprintln!("Error: {}", e);
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.increase_scale();
            }
            KeyCode::Char('-') => {
                self.decrease_scale();
            }
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(frame.area());

        let title = Paragraph::new("ASCII Art Image Viewer")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        let image_block = Block::default().borders(Borders::ALL).title(format!(
            "Image: {} (Scale: {:.2})",
            self.image_path, self.scale
        ));

        let ascii_paragraph = Paragraph::new(Text::from(self.ascii_art.clone()))
            .block(image_block)
            .style(Style::default().fg(Color::White));
        frame.render_widget(ascii_paragraph, chunks[1]);

        let help = Paragraph::new("Press + to zoom in, - to zoom out, q to quit")
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(help, chunks[2]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and load image
    // Replace with your own image path or use a sample image
    let sample_image = "assets/sample.jpg";
    let mut app = ImageViewerApp::new(sample_image);
    if let Err(e) = app.load_image() {
        eprintln!("Error loading image: {}", e);
        app.ascii_art = format!(
            "Error loading image: {}\n\nMake sure the image exists at the specified path.",
            e
        );
    }

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
