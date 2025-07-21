//! ASCII art animation viewer
//!
//! Displays a sequence of images as ASCII art animation

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
use std::{
    fs, io,
    path::Path,
    time::{Duration, Instant},
};

struct AnimationApp {
    frame_files: Vec<String>,
    current_frame: usize,
    ascii_art: String,
    scale: f32,
    should_quit: bool,
    playing: bool,
    last_frame_time: Instant,
    frame_duration: Duration,
}

impl AnimationApp {
    fn new(frames_dir: &str, fps: u32) -> Self {
        // List all image files in the frames directory
        let path = Path::new(frames_dir);
        let mut frame_files = Vec::new();

        if path.exists() && path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if let Some(extension) = path.extension() {
                            if extension == "jpg" || extension == "jpeg" || extension == "png" {
                                frame_files.push(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }

        // Sort the frame files to ensure they're in sequence
        frame_files.sort();

        Self {
            frame_files,
            current_frame: 0,
            ascii_art: String::new(),
            scale: 0.05, // Start with a small scale for performance
            should_quit: false,
            playing: false,
            last_frame_time: Instant::now(),
            frame_duration: Duration::from_secs_f64(1.0 / fps as f64),
        }
    }

    fn load_current_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.frame_files.is_empty() {
            self.ascii_art = "No frames found in the specified directory.\n\nMake sure you have extracted frames using:\nffmpeg -i assets/sample.mp4 -vf \"fps=10\" frames/output_%03d.jpg".to_string();
            return Ok(());
        }

        let frame_path = &self.frame_files[self.current_frame];

        // Load the image
        let img = image::open(frame_path)?;

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

    fn next_frame(&mut self) {
        if !self.frame_files.is_empty() {
            self.current_frame = (self.current_frame + 1) % self.frame_files.len();
            if let Err(e) = self.load_current_frame() {
                eprintln!("Error loading frame: {}", e);
            }
        }
    }

    fn increase_scale(&mut self) {
        self.scale *= 1.1;
        if let Err(e) = self.load_current_frame() {
            eprintln!("Error: {}", e);
        }
    }

    fn decrease_scale(&mut self) {
        self.scale *= 0.9;
        if let Err(e) = self.load_current_frame() {
            eprintln!("Error: {}", e);
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char(' ') => {
                self.playing = !self.playing;
                self.last_frame_time = Instant::now();
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.increase_scale();
            }
            KeyCode::Char('-') => {
                self.decrease_scale();
            }
            KeyCode::Right => {
                self.next_frame();
            }
            KeyCode::Left => {
                if !self.frame_files.is_empty() {
                    self.current_frame = if self.current_frame == 0 {
                        self.frame_files.len() - 1
                    } else {
                        self.current_frame - 1
                    };
                    if let Err(e) = self.load_current_frame() {
                        eprintln!("Error loading frame: {}", e);
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
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(frame.area());

        // Title with frame info
        let frame_info = if !self.frame_files.is_empty() {
            format!(
                "Frame {}/{}",
                self.current_frame + 1,
                self.frame_files.len()
            )
        } else {
            "No frames loaded".to_string()
        };

        let title = Paragraph::new(format!("ASCII Art Animation - {}", frame_info))
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Animation frame
        let image_block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Scale: {:.2}", self.scale));

        let ascii_paragraph = Paragraph::new(Text::from(self.ascii_art.clone()))
            .block(image_block)
            .style(Style::default().fg(Color::White));
        frame.render_widget(ascii_paragraph, chunks[1]);

        // Controls
        let status = if self.playing { "Playing" } else { "Paused" };
        let help = Paragraph::new(format!("Status: {} | Press SPACE to play/pause, LEFT/RIGHT to change frame, +/- to zoom, q to quit", status))
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(help, chunks[2]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create frames directory if it doesn't exist
    let frames_dir = "frames";
    if !Path::new(frames_dir).exists() {
        fs::create_dir_all(frames_dir)?;
        println!("Created frames directory. Please extract frames using:");
        println!("ffmpeg -i assets/sample.mp4 -vf \"fps=10\" frames/output_%03d.jpg");
        println!("Then run this example again.");
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and load first frame
    let mut app = AnimationApp::new(frames_dir, 10); // 10 FPS
    if let Err(e) = app.load_current_frame() {
        eprintln!("Error loading first frame: {}", e);
    }

    // Main loop
    loop {
        terminal.draw(|f| app.render(f))?;

        // Check for key events or advance frames if playing
        let timeout = Duration::from_millis(50);
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key.code);
            }
        } else if app.playing && app.last_frame_time.elapsed() >= app.frame_duration {
            app.next_frame();
            app.last_frame_time = Instant::now();
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
