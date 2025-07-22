//! Standalone image viewer example
//!
//! This example demonstrates the image rendering capabilities
//! including high-resolution terminal rendering via viuer and ASCII art conversion.

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
use viuer::Config;

struct ImageViewerApp {
    should_quit: bool,
    status_message: String,
    current_image_path: Option<String>,
    ascii_image: Vec<String>, // Store ASCII representation
    use_high_res: bool,       // Toggle between high-res and ASCII
    image_buffer: Option<image::DynamicImage>, // Store the loaded image
    last_terminal_size: (u16, u16), // Track terminal size for image redrawing
    needs_redraw: bool,       // Flag to indicate if image needs redrawing
}

impl ImageViewerApp {
    fn new() -> Self {
        Self {
            should_quit: false,
            status_message: "Press 'o' to open image, 'h' to toggle high-res mode, 'q' to quit".to_string(),
            current_image_path: None,
            ascii_image: Vec::new(),
            use_high_res: false,
            image_buffer: None,
            last_terminal_size: (0, 0),
            needs_redraw: false,
        }
    }

    fn load_image(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let img = image::open(path)?;
        self.current_image_path = Some(path.to_string());
        let (width, height) = img.dimensions();
        
        // Store the original image for high-res rendering
        self.image_buffer = Some(img.clone());
        
        // Always generate ASCII representation for fallback
        let resized_img = img.resize(80, 40, image::imageops::FilterType::Nearest);
        
        self.ascii_image = Vec::new();
        for y in 0..resized_img.height() {
            let mut line = String::new();
            for x in 0..resized_img.width() {
                let pixel = resized_img.get_pixel(x, y);
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

        if self.use_high_res {
            self.status_message = format!("Loaded image in high-res mode: {} ({}x{})", path, width, height);
            self.needs_redraw = true;
        } else {
            self.status_message = format!("Loaded image in ASCII mode: {} ({}x{})", path, width, height);
        }

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
            KeyCode::Char('h') => {
                // Toggle high-resolution mode
                self.use_high_res = !self.use_high_res;
                
                self.status_message = if self.use_high_res {
                    self.needs_redraw = true;  // Flag for redraw in high-res mode
                    "High-resolution mode enabled".to_string()
                } else {
                    "ASCII mode enabled".to_string()
                };
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

        // Title with mode indicator
        let title_text = if self.use_high_res {
            "🖼️ Image Viewer Example (High-Resolution Mode)"
        } else {
            "🖼️ Image Viewer Example (ASCII Mode)"
        };
        
        let title = Paragraph::new(title_text)
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
                Line::from("  │    Press 'h' to toggle high-res     │"),
                Line::from("  │                                     │"),
                Line::from("  │         Image will appear           │"),
                Line::from("  │         here                        │"),
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
            if self.use_high_res && self.current_image_path.is_some() {
                // For high-resolution mode
                let content_lines = vec![
                    Line::from(""),
                    Line::from("  High-resolution image loaded from:"),
                    Line::from(format!("  {}", self.current_image_path.as_ref().unwrap())),
                    Line::from(""),
                    Line::from("  Controls:"),
                    Line::from("  - Press 'h' to switch to ASCII mode"),
                    Line::from("  - Press 'q' to quit"),
                    Line::from(""),
                ];

                let content = Paragraph::new(content_lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("High-Resolution Image"),
                    )
                    .style(Style::default().fg(Color::Green));
                
                // Render the paragraph widget
                frame.render_widget(content, chunks[1]);
                
                // After rendering the frame, we'll use a special technique to draw the high-res image
                // This won't actually happen inside this function, but will be triggered 
                // after the frame is drawn in the main loop
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
        // Get terminal size to detect changes
        let terminal_size = terminal.size().unwrap_or_default();
        
        // Update the terminal size if it changed
        if app.last_terminal_size != (terminal_size.width, terminal_size.height) {
            app.last_terminal_size = (terminal_size.width, terminal_size.height);
            if app.use_high_res && app.image_buffer.is_some() {
                app.needs_redraw = true;
            }
        }

        // Draw the UI
        terminal.draw(|f| app.render(f))?;

        // Handle events
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key.code);
            }
        }

        // If we're in high-resolution mode and have an image to render
        if app.use_high_res && app.image_buffer.is_some() && app.needs_redraw {
            // Temporarily restore the terminal to normal mode to display the image
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), DisableMouseCapture)?;
            
            // Configure viuer for optimal display
            // Create a configuration for viuer
            let mut config = Config::default();
            config.transparent = true;
            config.absolute_offset = false;  // Using relative positioning instead
            config.width = Some((terminal_size.width / 2) as u32);
            config.height = Some((terminal_size.height / 2) as u32);
            config.truecolor = true;
            config.use_iterm = true;
            
            // Display the image from memory if possible
            if let Some(ref img) = app.image_buffer {
                match viuer::print(img, &config) {
                    Ok(_) => {
                        app.needs_redraw = false;
                    }
                    Err(e) => {
                        app.status_message = format!("Failed to display high-res image: {}", e);
                    }
                }
            }
            
            // Return to alternate screen mode
            enable_raw_mode()?;
            execute!(terminal.backend_mut(), EnableMouseCapture)?;
        }

        if app.should_quit {
            break;
        }
    }

    // Save these values before restoring the terminal
    let high_res_enabled = app.use_high_res;
    let image_buffer = app.image_buffer.take();

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // If we quit while in high-res mode and an image was loaded,
    // display it in full screen after exiting the TUI
    if high_res_enabled && image_buffer.is_some() {
        println!("Displaying high-resolution image...");
        
        // Configure viuer for optimal full-screen display
        let mut config = Config::default();
        config.transparent = true;
        config.absolute_offset = false;
        config.width = None;      // Auto-detect for full screen
        config.height = None;     // Auto-detect for full screen
        config.truecolor = true;  // Use true color
        config.use_iterm = true;  // Try iTerm protocol
        
        // Display the image directly from memory
        if let Err(e) = viuer::print(&image_buffer.unwrap(), &config) {
            println!("Failed to display high-resolution image: {}", e);
        }
    }

    Ok(())
}
