//! Image and video rendering module
//! 
//! Provides high-resolution image rendering and MP4 video frame extraction/playback
//! Features ASCII art conversion, frame caching, and cross-platform video support.

use std::io::{self, IoResult};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use image::{DynamicImage, ImageBuffer, Rgba, imageops::FilterType};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use log;
use std::collections::HashMap;

/// Error types for image operations
#[derive(Debug)]
pub enum ImageError {
    Io(io::Error),
    ImageError(image::ImageError),
    UnsupportedFormat(String),
    FfmpegError(String),
}

impl From<io::Error> for ImageError {
    fn from(err: io::Error) -> Self {
        ImageError::Io(err)
    }
}

impl From<image::ImageError> for ImageError {
    fn from(err: image::ImageError) -> Self {
        ImageError::ImageError(err)
    }
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::Io(err) => write!(f, "IO error: {err}"),
            ImageError::ImageError(err) => write!(f, "Image error: {err}"),
            ImageError::FfmpegError(msg) => write!(f, "FFmpeg error: {msg}"),
            ImageError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {msg}"),
        }
    }
}

impl std::error::Error for ImageError {}

/// Image quality settings that affect rendering performance and output
#[derive(Debug, Clone, Copy)]
pub enum ImageQuality {
    /// Fast rendering, lower quality
    Low,
    /// Balanced rendering and quality
    Medium,
    /// Slow rendering, highest quality
    High,
}

impl ImageQuality {
    /// Get the appropriate image filter for this quality level
    pub fn get_filter(&self) -> FilterType {
        match self {
            ImageQuality::Low => FilterType::Nearest,
            ImageQuality::Medium => FilterType::Triangle,
            ImageQuality::High => FilterType::Lanczos3,
        }
    }

    /// Get ASCII character set based on quality
    pub fn get_ascii_chars(&self) -> &'static str {
        match self {
            ImageQuality::Low => " .:-=+*#%@",
            ImageQuality::Medium => " .:-=+*#%@█",
            ImageQuality::High => " .'`^\",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$",
        }
    }
}

/// Image rendering widget with video playback support
pub struct ImageWidget {
    /// Current image being displayed
    current_image: Option<DynamicImage>,
    /// Frame cache for video playback
    frame_cache: Vec<DynamicImage>,
    /// Current frame index for video
    current_frame: usize,
    /// Last frame update time
    last_frame_time: Instant,
    /// Target frame rate for video playback
    frame_rate: f32,
    /// Image quality setting
    quality: ImageQuality,
    /// Whether video is playing
    is_playing: bool,
    /// Total video duration estimate
    total_frames: usize,
    /// Playback statistics
    frames_rendered: usize,
}

impl Default for ImageWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageWidget {
    /// Create a new image widget with default settings
    pub fn new() -> Self {
        Self {
            current_image: None,
            frame_cache: Vec::new(),
            current_frame: 0,
            last_frame_time: Instant::now(),
            frame_rate: 10.0,
            quality: ImageQuality::Medium,
            is_playing: false,
            total_frames: 0,
            frames_rendered: 0,
        }
    }

    /// Create a new image widget with custom settings
    pub fn with_quality(quality: ImageQuality) -> Self {
        Self {
            quality,
            ..Self::new()
        }
    }

    /// Load a single image from file
    pub fn load_image<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ImageError> {
        let path = path.as_ref();
        
        // Validate file extension
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .ok_or_else(|| ImageError::UnsupportedFormat("No file extension".to_string()))?;

        match extension.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "webp" => {
                let img = image::open(path)?;
                self.current_image = Some(img);
                // Clear video-related state when loading a static image
                self.frame_cache.clear();
                self.is_playing = false;
                self.current_frame = 0;
                Ok(())
            }
            _ => Err(ImageError::UnsupportedFormat(format!("Unsupported format: {extension}"))),
        }
    }

    /// Extract frames from MP4 video using FFmpeg
    pub fn load_mp4_frames<P: AsRef<Path>>(&mut self, mp4_path: P) -> Result<(), ImageError> {
        let mp4_path = mp4_path.as_ref();
        let output_dir = "frames";
        
        // Validate MP4 file exists
        if !mp4_path.exists() {
            return Err(ImageError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("MP4 file not found: {}", mp4_path.display())
            )));
        }

        // Create frames directory
        fs::create_dir_all(output_dir)?;

        // Clean existing frames
        if let Ok(entries) = fs::read_dir(output_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map_or(false, |ext| ext == "png") {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }

        // Extract frames using FFmpeg
        let fps_filter = format!("fps={}", self.frame_rate);
        let output_pattern = format!("{output_dir}/frame_%04d.png");

        log::info!("Extracting frames from {} at {} FPS", mp4_path.display(), self.frame_rate);

        let output = Command::new("ffmpeg")
            .args([
                "-i", mp4_path.to_str().unwrap(),
                "-vf", &fps_filter,
                "-y", // Overwrite existing files
                &output_pattern
            ])
            .output()
            .map_err(|e| ImageError::FfmpegError(format!("FFmpeg not found: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ImageError::FfmpegError(format!("FFmpeg failed: {stderr}")));
        }

        // Load extracted frames
        self.load_frame_sequence(output_dir)?;
        self.is_playing = true;
        self.total_frames = self.frame_cache.len();

        log::info!("Successfully loaded {} frames", self.total_frames);
        Ok(())
    }

    /// Load a sequence of frame images from a directory
    pub fn load_frame_sequence<P: AsRef<Path>>(&mut self, frames_dir: P) -> Result<(), ImageError> {
        let mut frames = Vec::new();
        let dir = fs::read_dir(frames_dir)?;
        
        let mut frame_files: Vec<_> = dir
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("png"))
                    .unwrap_or(false)
            })
            .collect();

        // Sort by filename to ensure correct frame order
        frame_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        for entry in frame_files {
            match image::open(entry.path()) {
                Ok(img) => frames.push(img),
                Err(e) => log::warn!("Failed to load frame {}: {e}", entry.path().display()),
            }
        }

        if frames.is_empty() {
            return Err(ImageError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "No valid frame images found"
            )));
        }

        self.frame_cache = frames;
        self.current_frame = 0;
        self.is_playing = true;
        Ok(())
    }

    /// Update animation frame if enough time has passed
    pub fn update_animation(&mut self) {
        if !self.is_playing || self.frame_cache.is_empty() {
            return;
        }

        let frame_duration = Duration::from_secs_f32(1.0 / self.frame_rate);
        
        if self.last_frame_time.elapsed() >= frame_duration {
            self.current_frame = (self.current_frame + 1) % self.frame_cache.len();
            self.last_frame_time = Instant::now();
            self.frames_rendered = self.frames_rendered.saturating_add(1);
        }
    }

    /// Get the current frame for rendering
    pub fn get_current_frame(&self) -> Option<&DynamicImage> {
        if !self.frame_cache.is_empty() {
            self.frame_cache.get(self.current_frame)
        } else {
            self.current_image.as_ref()
        }
    }

    /// Convert image to ASCII art with advanced character mapping
    pub fn image_to_ascii(&self, img: &DynamicImage, width: usize, height: usize) -> Vec<String> {
        let ascii_chars = self.quality.get_ascii_chars();
        let filter = self.quality.get_filter();
        
        // Resize image to target dimensions
        let img = img.resize_exact(width as u32, height as u32, filter);
        let img = img.to_luma8();
        
        let mut result = Vec::with_capacity(height);
        
        for y in 0..height {
            let mut line = String::with_capacity(width);
            for x in 0..width {
                let pixel = img.get_pixel(x as u32, y as u32);
                let brightness = pixel[0] as f32 / 255.0;
                
                // Apply gamma correction for better visual mapping
                let gamma_corrected = brightness.powf(0.45);
                
                let char_index = (gamma_corrected * (ascii_chars.len() - 1) as f32) as usize;
                let char_index = char_index.min(ascii_chars.len() - 1);
                
                line.push(ascii_chars.chars().nth(char_index).unwrap_or(' '));
            }
            result.push(line);
        }
        
        result
    }

    /// Set playback controls
    pub fn play(&mut self) {
        self.is_playing = true;
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    /// Toggle playback state
    pub fn toggle_playback(&mut self) {
        self.is_playing = !self.is_playing;
    }

    /// Set frame rate
    pub fn set_frame_rate(&mut self, fps: f32) {
        self.frame_rate = fps.clamp(1.0, 60.0);
    }

    /// Set image quality
    pub fn set_quality(&mut self, quality: ImageQuality) {
        self.quality = quality;
    }

    /// Get playback statistics
    pub fn get_stats(&self) -> PlaybackStats {
        PlaybackStats {
            total_frames: self.total_frames,
            current_frame: self.current_frame + 1,
            frame_rate: self.frame_rate,
            is_playing: self.is_playing,
            frames_rendered: self.frames_rendered,
        }
    }

    /// Render the image widget to the terminal
    pub fn render(&mut self, frame: &mut Frame, area: Rect) -> IoResult<()> {
        // Update animation state
        self.update_animation();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(10),    // Image area  
                Constraint::Length(5),  // Controls and info
            ])
            .split(area);

        // Title with current file info
        let title_text = if self.frame_cache.is_empty() {
            "🖼️ Image Viewer".to_string()
        } else {
            format!("🎬 Video Player (Frame {}/{})", 
                self.current_frame + 1, self.total_frames)
        };

        let title = Paragraph::new(title_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Image display area
        if let Some(img) = self.get_current_frame() {
            let img_area = chunks[1];
            let ascii_width = (img_area.width as usize).saturating_sub(4);
            let ascii_height = (img_area.height as usize).saturating_sub(2);
            
            if ascii_width > 0 && ascii_height > 0 {
                let ascii_art = self.image_to_ascii(img, ascii_width, ascii_height);
                let ascii_lines: Vec<Line> = ascii_art.into_iter()
                    .map(|line| Line::from(Span::raw(line)))
                    .collect();

                let image_display = Paragraph::new(ascii_lines)
                    .block(Block::default().borders(Borders::ALL).title("Display"))
                    .alignment(Alignment::Left);
                frame.render_widget(image_display, chunks[1]);
            }
        } else {
            self.render_placeholder(frame, chunks[1]);
        }

        // Controls and info panel
        self.render_info_panel(frame, chunks[2]);

        Ok(())
    }

    /// Render placeholder when no image is loaded
    fn render_placeholder(&self, frame: &mut Frame, area: Rect) {
        let placeholder_lines = vec![
            Line::from(""),
            Line::from("  ╭─────────────────────────────────────╮"),
            Line::from("  │                                     │"),
            Line::from("  │         No Media Loaded             │"),
            Line::from("  │                                     │"),
            Line::from("  │    Supported formats:               │"),
            Line::from("  │    • Images: PNG, JPEG, GIF, BMP    │"),
            Line::from("  │    • Videos: MP4 (via FFmpeg)       │"),
            Line::from("  │                                     │"),
            Line::from("  │    Press 'o' to load media          │"),
            Line::from("  │    Press 'h' for help               │"),
            Line::from("  │                                     │"),
            Line::from("  ╰─────────────────────────────────────╯"),
        ];

        let placeholder = Paragraph::new(placeholder_lines)
            .block(Block::default().borders(Borders::ALL).title("Media Display"))
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(placeholder, area);
    }

    /// Render information panel with controls and statistics
    fn render_info_panel(&self, frame: &mut Frame, area: Rect) {
        let info_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left panel: Media info
        let info_text = if let Some(img) = self.get_current_frame() {
            let (width, height) = img.dimensions();
            if self.is_playing {
                format!(
                    "Video: {}x{}\nFrame Rate: {:.1} FPS\nQuality: {:?}\nStatus: {}",
                    width, height, self.frame_rate, self.quality,
                    if self.is_playing { "Playing" } else { "Paused" }
                )
            } else {
                format!("Image: {}x{}\nQuality: {:?}\nType: Static", width, height, self.quality)
            }
        } else {
            "No media loaded\n\nSupported formats:\n• PNG, JPEG, GIF, BMP\n• MP4 (requires FFmpeg)".to_string()
        };

        let info = Paragraph::new(info_text)
            .block(Block::default().borders(Borders::ALL).title("Media Info"))
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(info, info_chunks[0]);

        // Right panel: Controls
        let controls_text = vec![
            Line::from(vec![
                Span::styled("Space", Style::default().fg(Color::Cyan)),
                Span::raw(" - Play/Pause"),
            ]),
            Line::from(vec![
                Span::styled("←/→", Style::default().fg(Color::Cyan)),
                Span::raw(" - Previous/Next frame"),
            ]),
            Line::from(vec![
                Span::styled("+/-", Style::default().fg(Color::Cyan)),
                Span::raw(" - Adjust frame rate"),
            ]),
            Line::from(vec![
                Span::styled("q", Style::default().fg(Color::Cyan)),
                Span::raw(" - Quality toggle"),
            ]),
        ];

        let controls = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::Green));
        frame.render_widget(controls, info_chunks[1]);
    }
}

/// Playback statistics
#[derive(Debug, Clone)]
pub struct PlaybackStats {
    pub total_frames: usize,
    pub current_frame: usize,
    pub frame_rate: f32,
    pub is_playing: bool,
    pub frames_rendered: usize,
}

/// High-level function to render a static image
pub fn render_static_image<P: AsRef<Path>>(
    frame: &mut Frame,
    area: Rect,
    image_path: P,
    quality: ImageQuality,
) -> Result<(), ImageError> {
    let mut widget = ImageWidget::with_quality(quality);
    widget.load_image(image_path)?;
    widget.render(frame, area)?;
    Ok(())
}

/// High-level function to setup and render MP4 video
pub fn render_mp4_video<P: AsRef<Path>>(
    frame: &mut Frame,
    area: Rect,
    mp4_path: P,
    quality: ImageQuality,
    frame_rate: f32,
) -> Result<ImageWidget, ImageError> {
    let mut widget = ImageWidget::with_quality(quality);
    widget.set_frame_rate(frame_rate);
    widget.load_mp4_frames(mp4_path)?;
    widget.render(frame, area)?;
    Ok(widget)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_image_widget_creation() {
        let widget = ImageWidget::new();
        assert!(widget.current_image.is_none());
        assert!(widget.frame_cache.is_empty());
        assert_eq!(widget.current_frame, 0);
        assert!(!widget.is_playing);
    }

    #[test]
    fn test_quality_settings() {
        assert!(matches!(ImageQuality::Low.get_filter(), FilterType::Nearest));
        assert!(matches!(ImageQuality::High.get_filter(), FilterType::Lanczos3));
        
        assert!(ImageQuality::High.get_ascii_chars().len() > ImageQuality::Low.get_ascii_chars().len());
    }

    #[test]
    fn test_ascii_conversion() {
        let widget = ImageWidget::new();
        
        // Create a test image with white pixels
        let img = DynamicImage::ImageRgba8(
            ImageBuffer::from_fn(10, 10, |_x, _y| Rgba([255u8, 255u8, 255u8, 255u8]))
        );
        
        let ascii = widget.image_to_ascii(&img, 5, 5);
        assert_eq!(ascii.len(), 5);
        assert!(ascii.iter().all(|line| line.len() == 5));
        
        // White pixels should map to the brightest character
        let brightest_char = widget.quality.get_ascii_chars().chars().last().unwrap();
        assert!(ascii.iter().all(|line| line.chars().all(|c| c == brightest_char)));
    }

    #[test]
    fn test_playback_controls() {
        let mut widget = ImageWidget::new();
        
        assert!(!widget.is_playing);
        
        widget.play();
        assert!(widget.is_playing);
        
        widget.pause();
        assert!(!widget.is_playing);
        
        widget.toggle_playback();
        assert!(widget.is_playing);
    }

    #[test]
    fn test_frame_rate_clamping() {
        let mut widget = ImageWidget::new();
        
        widget.set_frame_rate(0.5); // Below minimum
        assert_eq!(widget.frame_rate, 1.0);
        
        widget.set_frame_rate(100.0); // Above maximum
        assert_eq!(widget.frame_rate, 60.0);
        
        widget.set_frame_rate(15.0); // Within range
        assert_eq!(widget.frame_rate, 15.0);
    }

    #[test]
    fn test_unsupported_format_error() {
        let mut widget = ImageWidget::new();
        
        let temp_dir = tempdir().unwrap();
        let unsupported_file = temp_dir.path().join("test.txt");
        File::create(&unsupported_file).unwrap().write_all(b"not an image").unwrap();
        
        let result = widget.load_image(&unsupported_file);
        assert!(matches!(result, Err(ImageError::UnsupportedFormat(_))));
    }

    #[test]
    fn test_stats() {
        let widget = ImageWidget::new();
        let stats = widget.get_stats();
        
        assert_eq!(stats.total_frames, 0);
        assert_eq!(stats.current_frame, 1); // Frame numbers are 1-indexed for display
        assert!(!stats.is_playing);
        assert_eq!(stats.frames_rendered, 0);
    }
}

/// Emoji mapping for different categories
const EMOJI_MAP: &[(&str, &[(&str, &str)])] = &[
    ("smileys", &[]),
    ("animals", &[]),
    ("food", &[]),
    ("activities", &[]),
    ("places", &[]),
    ("objects", &[]),
    ("symbols", &[]),
    ("flags", &[]),
];

/// High-level function to render an emoji
pub fn render_emoji<P: AsRef<Path>>(
    frame: &mut Frame,
    area: Rect,
    emoji_name: &str,
    category: &str,
) -> Result<(), ImageError> {
    // Find the emoji data by name and category
    if let Some((_, emoji_data)) = EMOJI_MAP.iter().find(|(cat, _)| **cat == category) {
        for (name, unicode) in *emoji_data {
            if *name == emoji_name {
                // Render the emoji using the unicode point
                let unicode_char = char::from_u32(*unicode as u32).ok_or_else(|| {
                    ImageError::UnsupportedFormat(format!("Invalid unicode point for emoji: {emoji_name}"))
                })?;
                
                let emoji_lines = vec![
                    Line::from(format!("  {unicode_char}  ")),
                    Line::from(""),
                ];

                let emoji_display = Paragraph::new(emoji_lines)
                    .block(Block::default().borders(Borders::ALL).title("Emoji Display"))
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Yellow));
                frame.render_widget(emoji_display, area);

                return Ok(());
            }
        }
    }

    // Emoji not found, render a placeholder
    let placeholder_lines = vec![
        Line::from(""),
        Line::from("  ╭───────────────────────────╮"),
        Line::from("  │                           │"),
        Line::from("  │      Emoji Not Found     │"),
        Line::from("  │                           │"),
        Line::from("  │  Supported categories:    │"),
        Line::from("  │  - smileys                │"),
        Line::from("  │  - animals                │"),
        Line::from("  │  - food                   │"),
        Line::from("  │  - activities             │"),
        Line::from("  │  - places                 │"),
        Line::from("  │  - objects                │"),
        Line::from("  │  - symbols                │"),
        Line::from("  │  - flags                  │"),
        Line::from("  │                           │"),
        Line::from("  ╰───────────────────────────╯"),
    ];

    let placeholder = Paragraph::new(placeholder_lines)
        .block(Block::default().borders(Borders::ALL).title("Emoji Display"))
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(placeholder, area);

    Ok(())
}

/// High-level function to render text with emoji support
pub fn render_text_with_emojis<P: AsRef<Path>>(
    frame: &mut Frame,
    area: Rect,
    text: &str,
    emojis: &[(&str, &str)],
) -> Result<(), ImageError> {
    let lines: Vec<Line> = text.lines()
        .map(|line| {
            let mut spans = Vec::new();
            let mut last_pos = 0;

            // Find and highlight emojis in the text
            for (emoji_name, category) in emojis {
                if let Some((_, emoji_data)) = EMOJI_MAP.iter().find(|(cat, _)| **cat == *category) {
                    for (name, unicode) in *emoji_data {
                        if *name == *emoji_name {
                            // Add text before the emoji
                            if last_pos < line.len() {
                                spans.push(Span::raw(&line[last_pos..]));
                            }

                            // Add the emoji span
                            let unicode_char = char::from_u32(*unicode as u32).unwrap_or('�');
                            spans.push(Span::styled(format!(" {unicode_char} "), Style::default().fg(Color::Yellow)));

                            last_pos = line.len();
                        }
                    }
                }
            }

            // Add any remaining text after the last emoji
            if last_pos < line.len() {
                spans.push(Span::raw(&line[last_pos..]));
            }

            Line::from(spans)
        })
        .collect();

    let text_display = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Text with Emojis"))
        .alignment(Alignment::Left);
    frame.render_widget(text_display, area);

    Ok(())
}

/// High-level function to render a list of items with emoji support
pub fn render_emoji_list<P: AsRef<Path>>(
    frame: &mut Frame,
    area: Rect,
    items: &[(&str, &str)],
) -> Result<(), ImageError> {
    let lines: Vec<Line> = items.iter()
        .map(|(emoji_name, category)| {
            let mut spans = Vec::new();

            // Add the emoji span
            if let Some((_, emoji_data)) = EMOJI_MAP.iter().find(|(cat, _)| **cat == *category) {
                for (name, unicode) in *emoji_data {
                    if *name == *emoji_name {
                        let unicode_char = char::from_u32(*unicode as u32).unwrap_or('�');
                        spans.push(Span::styled(format!(" {unicode_char} "), Style::default().fg(Color::Yellow)));
                        break;
                    }
                }
            }

            // Add the item text
            spans.push(Span::raw(format!("{}{}", spans.len(), emoji_name)));

            Line::from(spans)
        })
        .collect();

    let list_display = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Emoji List"))
        .alignment(Alignment::Left);
    frame.render_widget(list_display, area);

    Ok(())
}

/// High-level function to render a grid of emojis
pub fn render_emoji_grid<P: AsRef<Path>>(
    frame: &mut Frame,
    area: Rect,
    category: &str,
) -> Result<(), ImageError> {
    // Find the emoji data by category
    if let Some((_, emoji_data)) = EMOJI_MAP.iter().find(|(cat, _)| **cat == category) {
        let mut lines = Vec::new();
        let mut current_line = Vec::new();
        let max_width = 8; // Max emojis per row
        
        for (name, unicode) in *emoji_data {
            // Add the emoji span
            let unicode_char = char::from_u32(*unicode as u32).unwrap_or('�');
            current_line.push(Span::styled(format!(" {unicode_char} "), Style::default().fg(Color::Yellow)));

            // Break the line if it reaches the maximum width
            if current_line.len() == max_width {
                lines.push(Line::from(current_line));
                current_line = Vec::new();
            }
        }

        // Add any remaining emojis in the last line
        if !current_line.is_empty() {
            lines.push(Line::from(current_line));
        }

        let grid_display = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Emoji Grid"))
            .alignment(Alignment::Left);
        frame.render_widget(grid_display, area);

        return Ok(());
    }

    // Emoji category not found, render an empty grid
    let empty_grid = vec![
        Line::from(""),
        Line::from("  ╭───────────────────────────╮"),
        Line::from("  │                           │"),
        Line::from("  │      No Emojis Found     │"),
        Line::from("  │                           │"),
        Line::from("  │  Supported categories:    │"),
        Line::from("  │  - smileys                │"),
        Line::from("  │  - animals                │"),
        Line::from("  │  - food                   │"),
        Line::from("  │  - activities             │"),
        Line::from("  │  - places                 │"),
        Line::from("  │  - objects                │"),
        Line::from("  │  - symbols                │"),
        Line::from("  │  - flags                  │"),
        Line::from("  │                           │"),
        Line::from("  ╰───────────────────────────╯"),
    ];

    let empty_display = Paragraph::new(empty_grid)
        .block(Block::default().borders(Borders::ALL).title("Emoji Grid"))
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(empty_display, area);

    Ok(())
}
