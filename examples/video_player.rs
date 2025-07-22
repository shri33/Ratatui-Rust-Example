use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::fs;

// Only include FFmpeg dependencies when the 'video' feature is enabled
#[cfg(feature = "video")]
use ffmpeg_next as ffmpeg;
#[cfg(feature = "video")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "video")]
use std::thread;

// Audio playback support
#[cfg(all(feature = "video", feature = "audio"))]
use rodio::{Decoder, OutputStream, Sink};
#[cfg(all(feature = "video", feature = "audio"))]
use std::sync::mpsc;

#[derive(Debug, Clone)]
pub struct VideoMetadata {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub duration: f64,
    pub total_frames: u64,
    pub format: String,
    pub has_audio: bool,
}

#[cfg(feature = "video")]
pub struct FrameCache {
    frames: HashMap<usize, ffmpeg::frame::Video>,
    max_size: usize,
    access_order: Vec<usize>,
}

#[cfg(feature = "video")]
impl FrameCache {
    fn new(max_size: usize) -> Self {
        Self {
            frames: HashMap::new(),
            max_size,
            access_order: Vec::new(),
        }
    }

    fn get(&mut self, index: usize) -> Option<&ffmpeg::frame::Video> {
        if self.frames.contains_key(&index) {
            // Update access order
            self.access_order.retain(|&x| x != index);
            self.access_order.push(index);
            self.frames.get(&index)
        } else {
            None
        }
    }

    fn insert(&mut self, index: usize, frame: ffmpeg::frame::Video) {
        if self.frames.len() >= self.max_size {
            // Remove least recently used frame
            if let Some(lru_index) = self.access_order.first().copied() {
                self.frames.remove(&lru_index);
                self.access_order.remove(0);
            }
        }
        
        self.frames.insert(index, frame);
        self.access_order.push(index);
    }

    fn clear(&mut self) {
        self.frames.clear();
        self.access_order.clear();
    }
}

#[derive(Debug, Clone)]
pub enum PlayerError {
    FileNotFound(String),
    UnsupportedFormat(String),
    DecodingError(String),
    AudioError(String),
    IoError(String),
}

impl std::fmt::Display for PlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerError::FileNotFound(path) => write!(f, "File not found: {}", path),
            PlayerError::UnsupportedFormat(format) => write!(f, "Unsupported format: {}", format),
            PlayerError::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
            PlayerError::AudioError(msg) => write!(f, "Audio error: {}", msg),
            PlayerError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for PlayerError {}

#[derive(Debug, PartialEq)]
enum AppMode {
    FileBrowser,
    VideoPlayer,
    Help,
}

pub struct FileBrowser {
    current_dir: PathBuf,
    items: Vec<PathBuf>,
    selected: usize,
    list_state: ListState,
}

impl FileBrowser {
    fn new() -> Result<Self, PlayerError> {
        let current_dir = std::env::current_dir()
            .map_err(|e| PlayerError::IoError(format!("Failed to get current directory: {}", e)))?;
        
        let mut browser = Self {
            current_dir,
            items: Vec::new(),
            selected: 0,
            list_state: ListState::default(),
        };
        
        browser.refresh_items()?;
        Ok(browser)
    }

    fn refresh_items(&mut self) -> Result<(), PlayerError> {
        self.items.clear();
        
        // Add parent directory if not at root
        if self.current_dir.parent().is_some() {
            self.items.push(self.current_dir.join(".."));
        }
        
        let entries = fs::read_dir(&self.current_dir)
            .map_err(|e| PlayerError::IoError(format!("Failed to read directory: {}", e)))?;
            
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else if self.is_video_file(&path) {
                files.push(path);
            }
        }
        
        dirs.sort();
        files.sort();
        
        self.items.extend(dirs);
        self.items.extend(files);
        
        self.selected = 0;
        self.list_state.select(Some(0));
        
        Ok(())
    }

    fn is_video_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            matches!(ext.as_str(), "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v")
        } else {
            false
        }
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
        }
    }

    fn move_down(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
            self.list_state.select(Some(self.selected));
        }
    }

    fn select_current(&mut self) -> Result<Option<PathBuf>, PlayerError> {
        if let Some(path) = self.items.get(self.selected) {
            if path.file_name() == Some(std::ffi::OsStr::new("..")) {
                // Go to parent directory
                if let Some(parent) = self.current_dir.parent() {
                    self.current_dir = parent.to_path_buf();
                    self.refresh_items()?;
                }
                Ok(None)
            } else if path.is_dir() {
                // Enter directory
                self.current_dir = path.clone();
                self.refresh_items()?;
                Ok(None)
            } else if self.is_video_file(path) {
                // Select video file
                Ok(Some(path.clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn get_display_items(&self) -> Vec<ListItem> {
        self.items.iter().map(|path| {
            let display_name = if path.file_name() == Some(std::ffi::OsStr::new("..")) {
                "📁 ..".to_string()
            } else if path.is_dir() {
                format!("📁 {}", path.file_name().unwrap_or_default().to_string_lossy())
            } else {
                format!("🎬 {}", path.file_name().unwrap_or_default().to_string_lossy())
            };
            ListItem::new(display_name)
        }).collect()
    }
}

#[cfg(feature = "video")]
fn display_frame_with_viuer(
    frame: &ffmpeg::frame::Video,
) -> Result<(), PlayerError> {
    use image::{ImageBuffer, Rgb};

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
        ).map_err(|e| PlayerError::DecodingError(format!("Failed to create scaler: {}", e)))?;
        
        scaler.run(frame, &mut rgb_frame)
            .map_err(|e| PlayerError::DecodingError(format!("Failed to scale frame: {}", e)))?;
    } else {
        rgb_frame.clone_from(frame);
    }

    // Get the raw RGB data
    let data = rgb_frame.data(0);
    let img: ImageBuffer<Rgb<u8>, _> = ImageBuffer::from_raw(width, height, data.to_vec())
        .ok_or_else(|| PlayerError::DecodingError("Failed to create image buffer from frame data".to_string()))?;

    // Save to a temporary file
    let tmp_path = Path::new("frame_tmp.png");
    img.save(tmp_path)
        .map_err(|e| PlayerError::IoError(format!("Failed to save temporary frame: {}", e)))?;

    // Display with viuer
    let config = viuer::Config::default();
    viuer::print_from_file(tmp_path, &config)
        .map_err(|e| PlayerError::IoError(format!("Failed to display frame: {}", e)))?;

    // Remove the temp file
    std::fs::remove_file(tmp_path).ok();

    Ok(())
}

struct VideoPlayerApp {
    mode: AppMode,
    should_quit: bool,
    status_message: String,
    error_message: Option<String>,
    file_browser: Option<FileBrowser>,
    
    #[cfg(feature = "video")]
    current_video_path: Option<PathBuf>,
    #[cfg(feature = "video")]
    video_metadata: Option<VideoMetadata>,
    #[cfg(feature = "video")]
    is_playing: bool,
    #[cfg(feature = "video")]
    current_frame: Option<ffmpeg::frame::Video>,
    #[cfg(feature = "video")]
    frame_index: usize,
    #[cfg(feature = "video")]
    last_frame_time: Instant,
    #[cfg(feature = "video")]
    frame_cache: FrameCache,
    #[cfg(feature = "video")]
    video_context: Option<(ffmpeg::format::context::Input, usize)>, // (input, stream_index)
    
    #[cfg(all(feature = "video", feature = "audio"))]
    audio_sink: Option<Arc<Mutex<Sink>>>,
    #[cfg(all(feature = "video", feature = "audio"))]
    _audio_stream: Option<OutputStream>,
    #[cfg(all(feature = "video", feature = "audio"))]
    volume: f32,
    #[cfg(all(feature = "video", feature = "audio"))]
    is_muted: bool,
}

impl VideoPlayerApp {
    fn new() -> Self {
        #[cfg(feature = "video")]
        {
            // Initialize FFmpeg
            ffmpeg::init().unwrap_or_else(|e| {
                eprintln!("FFmpeg initialization error: {}", e);
            });
        }

        let file_browser = match FileBrowser::new() {
            Ok(browser) => Some(browser),
            Err(e) => {
                eprintln!("Failed to initialize file browser: {}", e);
                None
            }
        };

        Self {
            mode: AppMode::FileBrowser,
            should_quit: false,
            status_message: "Press 'h' for help, Enter to select file/directory, 'q' to quit".to_string(),
            error_message: None,
            file_browser,
            
            #[cfg(feature = "video")]
            current_video_path: None,
            #[cfg(feature = "video")]
            video_metadata: None,
            #[cfg(feature = "video")]
            is_playing: false,
            #[cfg(feature = "video")]
            current_frame: None,
            #[cfg(feature = "video")]
            frame_index: 0,
            #[cfg(feature = "video")]
            last_frame_time: Instant::now(),
            #[cfg(feature = "video")]
            frame_cache: FrameCache::new(50), // Cache up to 50 frames
            #[cfg(feature = "video")]
            video_context: None,
            
            #[cfg(all(feature = "video", feature = "audio"))]
            audio_sink: None,
            #[cfg(all(feature = "video", feature = "audio"))]
            _audio_stream: None,
            #[cfg(all(feature = "video", feature = "audio"))]
            volume: 0.5,
            #[cfg(all(feature = "video", feature = "audio"))]
            is_muted: false,
        }
    }

    #[cfg(feature = "video")]
    fn load_video(&mut self, path: &Path) -> Result<(), PlayerError> {
        if !path.exists() {
            return Err(PlayerError::FileNotFound(path.display().to_string()));
        }

        // Clear previous video state
        self.frame_cache.clear();
        self.video_context = None;
        self.current_frame = None;
        self.frame_index = 0;

        let input = ffmpeg::format::input(path)
            .map_err(|e| PlayerError::DecodingError(format!("Failed to open video: {}", e)))?;
        
        let video_stream = input
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or_else(|| PlayerError::UnsupportedFormat("No video stream found".to_string()))?;
        
        let stream_index = video_stream.index();
        let context = ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())
            .map_err(|e| PlayerError::DecodingError(format!("Failed to create codec context: {}", e)))?;
        
        let decoder = context.decoder().video()
            .map_err(|e| PlayerError::DecodingError(format!("Failed to create video decoder: {}", e)))?;

        // Extract metadata
        let fps = if let Some(rate) = decoder.frame_rate() {
            f64::from(rate.numerator()) / f64::from(rate.denominator())
        } else {
            25.0 // Default FPS
        };

        let duration = if let Some(duration) = input.duration() {
            duration as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE)
        } else {
            0.0
        };

        let total_frames = if duration > 0.0 && fps > 0.0 {
            (duration * fps) as u64
        } else {
            0
        };

        let has_audio = input.streams().best(ffmpeg::media::Type::Audio).is_some();

        self.video_metadata = Some(VideoMetadata {
            width: decoder.width(),
            height: decoder.height(),
            fps,
            duration,
            total_frames,
            format: decoder.format().descriptor().unwrap_or("Unknown").name().to_string(),
            has_audio,
        });

        // Load first frame
        self.video_context = Some((input, stream_index));
        self.seek_to_frame(0)?;
        
        self.current_video_path = Some(path.to_path_buf());
        self.is_playing = false;
        self.last_frame_time = Instant::now();

        // Load audio if available
        #[cfg(all(feature = "video", feature = "audio"))]
        if has_audio {
            self.load_audio(path)?;
        }

        self.status_message = format!(
            "Loaded: {} ({}x{}, {:.2} fps, {:.1}s)", 
            path.file_name().unwrap_or_default().to_string_lossy(),
            self.video_metadata.as_ref().unwrap().width,
            self.video_metadata.as_ref().unwrap().height,
            fps,
            duration
        );
        
        self.error_message = None;
        self.mode = AppMode::VideoPlayer;
        Ok(())
    }

    #[cfg(all(feature = "video", feature = "audio"))]
    fn load_audio(&mut self, _path: &Path) -> Result<(), PlayerError> {
        // Note: This is a simplified audio implementation
        // In a real implementation, you'd need to extract audio from the video file
        // and synchronize it with video playback
        
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| PlayerError::AudioError(format!("Failed to create audio stream: {}", e)))?;
        
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| PlayerError::AudioError(format!("Failed to create audio sink: {}", e)))?;
        
        sink.set_volume(self.volume);
        if self.is_muted {
            sink.pause();
        }
        
        self.audio_sink = Some(Arc::new(Mutex::new(sink)));
        self._audio_stream = Some(_stream);
        
        Ok(())
    }

    #[cfg(feature = "video")]
    fn seek_to_frame(&mut self, target_frame: usize) -> Result<(), PlayerError> {
        // Check cache first
        if let Some(cached_frame) = self.frame_cache.get(target_frame).cloned() {
            self.current_frame = Some(cached_frame);
            self.frame_index = target_frame;
            return Ok(());
        }

        if let Some((ref input, stream_index)) = self.video_context.as_ref() {
            // For seeking, we need to create a new input context due to FFmpeg limitations
            let path = self.current_video_path.as_ref().unwrap();
            let input = ffmpeg::format::input(path)
                .map_err(|e| PlayerError::DecodingError(format!("Failed to reopen video for seeking: {}", e)))?;
            
            let stream = input.streams().get(*stream_index)
                .ok_or_else(|| PlayerError::DecodingError("Stream not found".to_string()))?;
            
            let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())
                .map_err(|e| PlayerError::DecodingError(format!("Failed to create codec context: {}", e)))?;
            
            let mut decoder = context.decoder().video()
                .map_err(|e| PlayerError::DecodingError(format!("Failed to create video decoder: {}", e)))?;

            let mut packet_stream = input.packets();
            let mut current_index = 0;
            
            while let Some((stream, packet)) = packet_stream.next() {
                if stream.index() == *stream_index {
                    decoder.send_packet(&packet)
                        .map_err(|e| PlayerError::DecodingError(format!("Failed to send packet: {}", e)))?;
                    
                    let mut frame = ffmpeg::frame::Video::empty();
                    while decoder.receive_frame(&mut frame).is_ok() {
                        if current_index == target_frame {
                            // Cache the frame
                            let frame_clone = frame.clone();
                            self.frame_cache.insert(target_frame, frame_clone);
                            self.current_frame = Some(frame);
                            self.frame_index = target_frame;
                            return Ok(());
                        }
                        current_index += 1;
                    }
                }
            }
        }

        Err(PlayerError::DecodingError("Failed to seek to frame".to_string()))
    }

    #[cfg(feature = "video")]
    fn next_frame(&mut self) -> Result<(), PlayerError> {
        if let Some(metadata) = &self.video_metadata {
            if (self.frame_index as u64) < metadata.total_frames {
                self.seek_to_frame(self.frame_index + 1)
            } else {
                // End of video
                self.is_playing = false;
                self.status_message = "End of video reached".to_string();
                Ok(())
            }
        } else {
            Err(PlayerError::DecodingError("No video loaded".to_string()))
        }
    }

    #[cfg(feature = "video")]
    fn previous_frame(&mut self) -> Result<(), PlayerError> {
        if self.frame_index > 0 {
            self.seek_to_frame(self.frame_index - 1)
        } else {
            Ok(())
        }
    }

    #[cfg(all(feature = "video", feature = "audio"))]
    fn adjust_volume(&mut self, delta: f32) {
        self.volume = (self.volume + delta).clamp(0.0, 1.0);
        if let Some(ref sink) = self.audio_sink {
            if let Ok(sink) = sink.lock() {
                sink.set_volume(self.volume);
            }
        }
        self.status_message = format!("Volume: {:.0}%", self.volume * 100.0);
    }

    #[cfg(all(feature = "video", feature = "audio"))]
    fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
        if let Some(ref sink) = self.audio_sink {
            if let Ok(sink) = sink.lock() {
                if self.is_muted {
                    sink.pause();
                } else {
                    sink.play();
                }
            }
        }
        self.status_message = if self.is_muted {
            "Muted".to_string()
        } else {
            format!("Volume: {:.0}%", self.volume * 100.0)
        };
    }

    fn on_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match self.mode {
            AppMode::FileBrowser => self.handle_browser_key(key, modifiers),
            AppMode::VideoPlayer => self.handle_player_key(key, modifiers),
            AppMode::Help => self.handle_help_key(key, modifiers),
        }
    }

    fn handle_browser_key(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('h') => {
                self.mode = AppMode::Help;
            }
            KeyCode::Up => {
                if let Some(ref mut browser) = self.file_browser {
                    browser.move_up();
                }
            }
            KeyCode::Down => {
                if let Some(ref mut browser) = self.file_browser {
                    browser.move_down();
                }
            }
            KeyCode::Enter => {
                if let Some(ref mut browser) = self.file_browser {
                    match browser.select_current() {
                        Ok(Some(path)) => {
                            #[cfg(feature = "video")]
                            {
                                if let Err(e) = self.load_video(&path) {
                                    self.error_message = Some(e.to_string());
                                }
                            }
                            #[cfg(not(feature = "video"))]
                            {
                                self.error_message = Some("Video support not enabled. Compile with --features=video".to_string());
                            }
                        }
                        Ok(None) => {
                            // Directory navigation handled internally
                        }
                        Err(e) => {
                            self.error_message = Some(e.to_string());
                        }
                    }
                }
            }
            KeyCode::Esc => {
                self.error_message = None;
            }
            _ => {}
        }
    }

    fn handle_player_key(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('h') => {
                self.mode = AppMode::Help;
            }
            KeyCode::Char('b') => {
                self.mode = AppMode::FileBrowser;
                self.status_message = "File browser - Press Enter to select, 'h' for help".to_string();
            }
            #[cfg(feature = "video")]
            KeyCode::Char(' ') => {
                self.is_playing = !self.is_playing;
                self.status_message = if self.is_playing {
                    "Playing".to_string()
                } else {
                    "Paused".to_string()
                };

                #[cfg(all(feature = "video", feature = "audio"))]
                if let Some(ref sink) = self.audio_sink {
                    if let Ok(sink) = sink.lock() {
                        if self.is_playing && !self.is_muted {
                            sink.play();
                        } else {
                            sink.pause();
                        }
                    }
                }
            }
            #[cfg(feature = "video")]
            KeyCode::Right => {
                if let Err(e) = self.next_frame() {
                    self.error_message = Some(e.to_string());
                }
            }
            #[cfg(feature = "video")]
            KeyCode::Left => {
                if let Err(e) = self.previous_frame() {
                    self.error_message = Some(e.to_string());
                }
            }
            #[cfg(feature = "video")]
            KeyCode::Char('d') => {
                if let Some(ref frame) = self.current_frame {
                    if let Err(e) = display_frame_with_viuer(frame) {
                        self.error_message = Some(e.to_string());
                    }
                } else {
                    self.error_message = Some("No frame to display".to_string());
                }
            }
            #[cfg(all(feature = "video", feature = "audio"))]
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.adjust_volume(0.1);
            }
            #[cfg(all(feature = "video", feature = "audio"))]
            KeyCode::Char('-') => {
                self.adjust_volume(-0.1);
            }
            #[cfg(all(feature = "video", feature = "audio"))]
            KeyCode::Char('m') => {
                self.toggle_mute();
            }
            KeyCode::Esc => {
                self.error_message = None;
            }
            _ => {}
        }
    }

    fn handle_help_key(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Esc | KeyCode::Char('h') => {
                self.mode = if self.current_video_path.is_some() {
                    AppMode::VideoPlayer
                } else {
                    AppMode::FileBrowser
                };
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        match self.mode {
            AppMode::FileBrowser => self.render_file_browser(frame),
            AppMode::VideoPlayer => self.render_video_player(frame),
            AppMode::Help => self.render_help(frame),
        }

        // Render error popup if present
        if let Some(ref error) = self.error_message {
            self.render_error_popup(frame, error);
        }
    }

    fn render_file_browser(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // File list
                Constraint::Length(3), // Status
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new("📁 File Browser - Select a Video File")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        frame.render_widget(title, chunks[0]);

        // File list
        if let Some(ref mut browser) = self.file_browser {
            let items = browser.get_display_items();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Files"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
                .highlight_symbol("► ");
            frame.render_stateful_widget(list, chunks[1], &mut browser.list_state);
        } else {
            let error_text = Paragraph::new("Failed to initialize file browser")
                .block(Block::default().borders(Borders::ALL).title("Error"))
                .style(Style::default().fg(Color::Red));
            frame.render_widget(error_text, chunks[1]);
        }

        // Status
        let status = Paragraph::new(self.status_message.as_str())
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(status, chunks[2]);
    }

    fn render_video_player(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(8),    // Video info
                Constraint::Length(3), // Progress bar
                Constraint::Length(3), // Controls
                Constraint::Length(3), // Status
            ])
            .split(frame.area());

        // Title
        let title_text = if let Some(ref path) = self.current_video_path {
            format!("🎬 Video Player - {}", path.file_name().unwrap_or_default().to_string_lossy())
        } else {
            "🎬 Video Player".to_string()
        };
        
        let title = Paragraph::new(title_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        frame.render_widget(title, chunks[0]);

        // Video info
        #[cfg(feature = "video")]
        let content_text = if let Some(ref metadata) = self.video_metadata {
            format!(
                "Resolution: {}x{}\nFPS: {:.2}\nDuration: {:.1}s\nFormat: {}\nAudio: {}\nFrame: {}/{}",
                metadata.width,
                metadata.height,
                metadata.fps,
                metadata.duration,
                metadata.format,
                if metadata.has_audio { "Yes" } else { "No" },
                self.frame_index,
                metadata.total_frames
            )
        } else {
            "No video loaded. Press 'b' to browse for files.".to_string()
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
        │    Then run: cargo run --features=video,audio              │
        │                                                             │
        ╰─────────────────────────────────────────────────────────────╯
        ".to_string();

        let content = Paragraph::new(content_text)
            .block(Block::default().borders(Borders::ALL).title("Video Info"))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        frame.render_widget(content, chunks[1]);

        // Progress bar
        #[cfg(feature = "video")]
        if let Some(ref metadata) = self.video_metadata {
            let progress = if metadata.total_frames > 0 {
                (self.frame_index as f64 / metadata.total_frames as f64 * 100.0) as u16
            } else {
                0
            };
            
            let progress_bar = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Progress"))
                .gauge_style(Style::default().fg(Color::Green))
                .percent(progress)
                .label(format!("{:.1}%", progress as f64));
            frame.render_widget(progress_bar, chunks[2]);
        } else {
            let empty_bar = Paragraph::new("")
                .block(Block::default().borders(Borders::ALL).title("Progress"));
            frame.render_widget(empty_bar, chunks[2]);
        }

        #[cfg(not(feature = "video"))]
        {
            let empty_bar = Paragraph::new("")
                .block(Block::default().borders(Borders::ALL).title("Progress"));
            frame.render_widget(empty_bar, chunks[2]);
        }

        // Controls
        let controls_text = 
            "Space: Play/Pause │ ←/→: Previous/Next Frame │ D: Display Frame │ B: Browse Files │ H: Help";
        
        #[cfg(all(feature = "video", feature = "audio"))]
        let controls_text = format!("{} │ +/-: Volume │ M: Mute", controls_text);

        let controls = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        frame.render_widget(controls, chunks[3]);

        // Status
        let mut status_text = self.status_message.clone();
        
        #[cfg(all(feature = "video", feature = "audio"))]
        if self.audio_sink.is_some() {
            if self.is_muted {
                status_text = format!("{} │ 🔇 MUTED", status_text);
            } else {
                status_text = format!("{} │ 🔊 {:.0}%", status_text, self.volume * 100.0);
            }
        }

        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(status, chunks[4]);
    }

    fn render_help(&self, frame: &mut Frame) {
        let help_text = "
╭─────────────────────────────────────────────────────────────╮
│                        HELP - VIDEO PLAYER                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  FILE BROWSER CONTROLS:                                     │
│    ↑/↓         Navigate files and directories               │
│    Enter       Select file or enter directory               │
│    Esc         Clear error messages                         │
│                                                             │
│  VIDEO PLAYER CONTROLS:                                     │
│    Space       Play/Pause video                             │
│    ←/→         Previous/Next frame                          │
│    D           Display current frame in terminal            │
│    B           Return to file browser                       │
│                                                             │
│  AUDIO CONTROLS (if audio feature enabled):                 │
│    +/=         Increase volume                              │
│    -           Decrease volume                              │
│    M           Toggle mute                                  │
│                                                             │
│  GENERAL CONTROLS:                                          │
│    H           Show/hide this help screen                   │
│    Q           Quit application                             │
│    Esc         Close error dialogs                          │
│                                                             │
│  SUPPORTED VIDEO FORMATS:                                   │
│    MP4, AVI, MKV, MOV, WMV, FLV, WebM, M4V                 │
│                                                             │
│  FEATURES:                                                  │
│    • Frame-by-frame navigation with caching                │
│    • Video metadata display                                │
│    • Progress tracking                                      │
│    • Error handling and user feedback                      │
│    • Audio playback (with audio feature)                   │
│                                                             │
╰─────────────────────────────────────────────────────────────╯

                    Press H or Esc to return
        ";

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(help, frame.area());
    }

    fn render_error_popup(&self, frame: &mut Frame, error: &str) {
        let area = frame.area();
        let popup_width = std::cmp::min(60, area.width.saturating_sub(4));
        let popup_height = std::cmp::min(8, area.height.saturating_sub(4));
        
        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        frame.render_widget(Clear, popup_area);
        
        let error_text = format!("❌ Error:\n\n{}\n\nPress Esc to dismiss", error);
        let error_popup = Paragraph::new(error_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Error")
                .style(Style::default().fg(Color::Red)))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);
        
        frame.render_widget(error_popup, popup_area);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for required features
    #[cfg(not(feature = "video"))]
    {
        println!("⚠️  Video support is not enabled!");
        println!("   Compile with: cargo run --features=video");
        println!("   For audio support: cargo run --features=video,audio");
        println!();
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = VideoPlayerApp::new();

    #[cfg(feature = "video")]
    let mut last_frame = Instant::now();
    #[cfg(feature = "video")]
    let frame_duration = Duration::from_secs_f64(1.0 / 30.0); // 30 FPS for UI updates

    // Main application loop
    loop {
        terminal.draw(|f| app.render(f))?;

        // Handle input with appropriate timeout
        let timeout = if cfg!(feature = "video") && app.is_playing { 16 } else { 100 };
        
        if event::poll(Duration::from_millis(timeout))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key.code, key.modifiers);
            }
        }

        // Update video playback
        #[cfg(feature = "video")]
        if app.is_playing && app.current_frame.is_some() {
            let now = Instant::now();
            let target_fps = app.video_metadata.as_ref().map(|m| m.fps).unwrap_or(24.0);
            let target_duration = Duration::from_secs_f64(1.0 / target_fps);
            
            if now.duration_since(last_frame) >= target_duration {
                if let Err(e) = app.next_frame() {
                    app.error_message = Some(e.to_string());
                    app.is_playing = false;
                }
                last_frame = now;
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}