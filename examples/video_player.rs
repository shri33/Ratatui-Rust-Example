use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;

// Only include FFmpeg dependencies when the 'video' feature is enabled
#[cfg(feature = "video")]
use ffmpeg_next as ffmpeg;
#[cfg(feature = "video")]
use std::path::Path;
#[cfg(feature = "video")]
use std::time::Instant;

#[cfg(feature = "video")]
fn display_frame_with_viuer(
    frame: &ffmpeg::frame::Video,
) -> Result<(), Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Rgb};
    use std::path::Path;

    let width = frame.width();
    let height = frame.height();

    // Convert the frame to RGB24 if not already
    let mut rgb_frame = ffmpeg::frame::Video::empty();
    if frame.format() != ffmpeg::format::Pixel::RGB24 {
        let mut scaler = ffmpeg::software::scaling::Context::get(
            frame.format(),
            width,
            height,
            ffmpeg::format::Pixel::RGB24,
            width,
            height,
            ffmpeg::software::scaling::flag::Flags::BILINEAR,
        )?;
        scaler.run(frame, &mut rgb_frame)?;
    } else {
        rgb_frame.clone_from(frame);
    }

    // Get the raw RGB data
    let data = rgb_frame.data(0);
    let img: ImageBuffer<Rgb<u8>, _> = ImageBuffer::from_raw(width, height, data.to_vec())
        .ok_or("Failed to create image buffer from frame data")?;

    // Save to a temporary file
    let tmp_path = Path::new("frame_tmp.png");
    img.save(&tmp_path)?;

    // Display with viuer
    let config = viuer::Config::default();
    viuer::print_from_file(&tmp_path, &config)?;

    // Optionally, remove the temp file
    std::fs::remove_file(&tmp_path).ok();

    Ok(())
}

struct VideoPlayerApp {
    should_quit: bool,
    status_message: String,
    #[cfg(feature = "video")]
    current_video_path: Option<String>,
    #[cfg(feature = "video")]
    is_playing: bool,
    #[cfg(feature = "video")]
    current_frame: Option<ffmpeg::frame::Video>,
    #[cfg(feature = "video")]
    frame_index: usize,
    #[cfg(feature = "video")]
    fps: f64,
    #[cfg(feature = "video")]
    last_frame_time: Instant,
}

impl VideoPlayerApp {
    fn new() -> Self {
        #[cfg(feature = "video")]
        {
            // Initialize FFmpeg
            ffmpeg::init().unwrap_or_else(|e| {
                eprintln!("FFmpeg initialization error: {}", e);
            });
            Self {
                should_quit: false,
                status_message:
                    "Press 'o' to open video, Space to play/pause, d to display frame, q to quit"
                        .to_string(),
                current_video_path: None,
                is_playing: false,
                current_frame: None,
                frame_index: 0,
                fps: 24.0,
                last_frame_time: Instant::now(),
            }
        }
        #[cfg(not(feature = "video"))]
        {
            Self {
                should_quit: false,
                status_message: "FFmpeg support is not enabled. Compile with --features=video"
                    .to_string(),
            }
        }
    }

    #[cfg(feature = "video")]
    fn load_video(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(format!("File not found: {}", path.display()).into());
        }
        let input = ffmpeg::format::input(&path)?;
        let stream = input
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or("No video stream found")?;
        let stream_index = stream.index();
        let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
        let mut decoder = context.decoder().video()?;
        if let Some(rate) = decoder.frame_rate() {
            self.fps = f64::from(rate.numerator()) / f64::from(rate.denominator());
        }
        self.current_video_path = Some(path.to_string_lossy().to_string());
        self.frame_index = 0;
        self.is_playing = true;
        self.last_frame_time = Instant::now();
        let mut packet_stream = input.packets();
        while let Some((stream, packet)) = packet_stream.next() {
            if stream.index() == stream_index {
                decoder.send_packet(&packet)?;
                let mut frame = ffmpeg::frame::Video::empty();
                if decoder.receive_frame(&mut frame).is_ok() {
                    self.current_frame = Some(frame);
                    break;
                }
            }
        }
        self.status_message = format!("Loaded video: {} ({:.2} fps)", path.display(), self.fps);
        Ok(())
    }

    #[cfg(feature = "video")]
    fn next_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = &self.current_video_path {
            let input = ffmpeg::format::input(&path)?;
            let stream = input
                .streams()
                .best(ffmpeg::media::Type::Video)
                .ok_or("No video stream found")?;
            let stream_index = stream.index();
            let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
            let mut decoder = context.decoder().video()?;
            let mut packet_stream = input.packets();
            let mut current_index = 0;
            while let Some((stream, packet)) = packet_stream.next() {
                if stream.index() == stream_index {
                    decoder.send_packet(&packet)?;
                    let mut frame = ffmpeg::frame::Video::empty();
                    if decoder.receive_frame(&mut frame).is_ok() {
                        if current_index == self.frame_index {
                            self.current_frame = Some(frame);
                            self.frame_index += 1;
                            break;
                        }
                        current_index += 1;
                    }
                }
            }
            self.status_message = format!("Playing frame: {}", self.frame_index);
        }
        Ok(())
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            #[cfg(feature = "video")]
            KeyCode::Char('o') => {
                if let Err(e) = self.load_video("assets/demo.mp4") {
                    self.status_message = format!("Error loading video: {}", e);
                }
            }
            #[cfg(feature = "video")]
            KeyCode::Char(' ') => {
                self.is_playing = !self.is_playing;
                self.status_message = if self.is_playing {
                    "Playing".to_string()
                } else {
                    "Paused".to_string()
                };
            }
            #[cfg(feature = "video")]
            KeyCode::Right => {
                self.frame_index += 10;
                if let Err(e) = self.next_frame() {
                    self.status_message = format!("Error advancing frame: {}", e);
                }
            }
            #[cfg(feature = "video")]
            KeyCode::Left => {
                if self.frame_index > 10 {
                    self.frame_index -= 10;
                } else {
                    self.frame_index = 0;
                }
                if let Err(e) = self.next_frame() {
                    self.status_message = format!("Error going back: {}", e);
                }
            }
            #[cfg(feature = "video")]
            KeyCode::Char('d') => {
                if let Some(ref frame) = self.current_frame {
                    if let Err(e) = display_frame_with_viuer(frame) {
                        self.status_message = format!("Error displaying frame: {}", e);
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
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Status
            ])
            .split(frame.area());
        let title = Paragraph::new("🎬 Video Player Example")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);
        #[cfg(feature = "video")]
        let content_text = if let Some(video_frame) = &self.current_frame {
            let width = video_frame.width();
            let height = video_frame.height();
            format!(
                "Video frame: {}x{} (Frame #{})",
                width, height, self.frame_index
            )
        } else {
            "No video loaded. Press 'o' to open a video file.".to_string()
        };
        #[cfg(not(feature = "video"))]
        let content_text = "
        ╭─────────────────────────────────────────────────────────────╮
        │                                                             │
        │               FFmpeg support is not enabled                 │
        │                                                             │
        │    This example requires FFmpeg to be properly installed    │
        │    and the project to be compiled with --features=video     │
        │                                                             │
        │    Installation steps:                                      │
        │                                                             │
        │    Windows:                                                 │
        │    1. Download from ffmpeg.org                              │
        │    2. Add to PATH                                           │
        │    3. Set PKG_CONFIG_PATH environment variable              │
        │                                                             │
        │    OR use vcpkg:                                            │
        │    vcpkg install ffmpeg:x64-windows                         │
        │    vcpkg integrate install                                  │
        │                                                             │
        │    Then run: cargo run --example video_player --features=video │
        │                                                             │
        ╰─────────────────────────────────────────────────────────────╯
        ".to_string();
        let content = Paragraph::new(content_text)
            .block(Block::default().borders(Borders::ALL).title("Video"))
            .style(Style::default().fg(Color::White));
        frame.render_widget(content, chunks[1]);
        let status = Paragraph::new(self.status_message.as_str())
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(status, chunks[2]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = VideoPlayerApp::new();
    #[cfg(feature = "video")]
    let mut last_frame = Instant::now();
    #[cfg(feature = "video")]
    let frame_duration = Duration::from_secs_f64(1.0 / 24.0); // 24 FPS default
    loop {
        terminal.draw(|f| app.render(f))?;
        let timeout = if cfg!(feature = "video") { 100 } else { 1000 };
        if event::poll(Duration::from_millis(timeout))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key.code);
            }
        }
        #[cfg(feature = "video")]
        if app.is_playing && app.current_frame.is_some() {
            let now = Instant::now();
            if now.duration_since(last_frame) >= frame_duration {
                if let Err(e) = app.next_frame() {
                    app.status_message = format!("Error: {}", e);
                    app.is_playing = false;
                }
                last_frame = now;
            }
        }
        if app.should_quit {
            break;
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
