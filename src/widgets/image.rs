//! Image and video rendering module
//! 
//! Provides high-resolution image rendering and MP4 video frame extraction/playback
//! Features ASCII art conversion, frame caching, and cross-platform video support.

use std::io::{self, Result as IoResult};
use std::path::Path;
use std::time::Instant;
use image::{DynamicImage, imageops::FilterType};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

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

/// Image quality settings
#[derive(Debug, Clone, Copy)]
pub enum ImageQuality {
    Low,
    Medium,
    High,
}

impl ImageQuality {
    pub fn get_filter(&self) -> FilterType {
        match self {
            ImageQuality::Low => FilterType::Nearest,
            ImageQuality::Medium => FilterType::Triangle,
            ImageQuality::High => FilterType::Lanczos3,
        }
    }

    pub fn get_ascii_chars(&self) -> &'static str {
        match self {
            ImageQuality::Low => " .:-=+*#%@",
            ImageQuality::Medium => " .:-=+*#%@█",
            ImageQuality::High => " .'`^\",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$",
        }
    }
}

/// Image rendering widget
pub struct ImageWidget {
    current_image: Option<DynamicImage>,
    frame_cache: Vec<DynamicImage>,
    current_frame: usize,
    quality: ImageQuality,
    is_playing: bool,
    // Video-related fields for future use
    #[allow(dead_code)]
    last_frame_time: Instant,
    #[allow(dead_code)]
    frame_rate: f32,
    #[allow(dead_code)]
    total_frames: usize,
    #[allow(dead_code)]
    frames_rendered: usize,
}

impl Default for ImageWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageWidget {
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

    pub fn with_quality(quality: ImageQuality) -> Self {
        Self {
            quality,
            ..Self::new()
        }
    }

    pub fn load_image<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ImageError> {
        let img = image::open(path)?;
        self.current_image = Some(img);
        self.frame_cache.clear();
        self.is_playing = false;
        self.current_frame = 0;
        self.total_frames = 1; // Single image has 1 frame
        self.frames_rendered = 0;
        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) -> IoResult<()> {
        if let Some(img) = &self.current_image {
            let ascii_art = self.image_to_ascii(img, area.width as usize, area.height as usize);
            
            let lines: Vec<Line> = ascii_art.into_iter()
                .map(|line| Line::from(line))
                .collect();

            // Add frame info to title for video playback info
            let title = if self.total_frames > 1 {
                format!("Image ({}/{})", self.current_frame + 1, self.total_frames)
            } else {
                "Image".to_string()
            };

            let paragraph = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title(title))
                .style(Style::default().fg(Color::White));
            frame.render_widget(paragraph, area);
            
            self.frames_rendered += 1;
        } else {
            let placeholder = Paragraph::new("No image loaded")
                .block(Block::default().borders(Borders::ALL).title("Image"))
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(placeholder, area);
        }
        Ok(())
    }

    /// Get current playback information
    pub fn get_playback_info(&self) -> (usize, usize, bool) {
        (self.current_frame, self.total_frames, self.is_playing)
    }

    /// Set frame rate for video playback
    pub fn set_frame_rate(&mut self, fps: f32) {
        self.frame_rate = fps;
    }

    /// Check if enough time has passed for next frame
    pub fn should_advance_frame(&self) -> bool {
        if !self.is_playing || self.total_frames <= 1 {
            return false;
        }
        
        let frame_duration = std::time::Duration::from_secs_f32(1.0 / self.frame_rate);
        self.last_frame_time.elapsed() >= frame_duration
    }

    /// Advance to next frame
    pub fn next_frame(&mut self) {
        if self.frame_cache.len() > 1 {
            self.current_frame = (self.current_frame + 1) % self.frame_cache.len();
            self.last_frame_time = Instant::now();
        }
    }

    /// Toggle playback
    pub fn toggle_playback(&mut self) {
        self.is_playing = !self.is_playing;
        if self.is_playing {
            self.last_frame_time = Instant::now();
        }
    }

    fn image_to_ascii(&self, img: &DynamicImage, width: usize, height: usize) -> Vec<String> {
        let ascii_chars = self.quality.get_ascii_chars();
        let filter = self.quality.get_filter();
        
        let img = img.resize_exact(width as u32, height as u32, filter);
        let img = img.to_luma8();
        
        let mut result = Vec::with_capacity(height);
        
        for y in 0..height {
            let mut line = String::with_capacity(width);
            for x in 0..width {
                let pixel = img.get_pixel(x as u32, y as u32);
                let brightness = pixel[0] as f32 / 255.0;
                let char_index = (brightness * (ascii_chars.len() - 1) as f32) as usize;
                let char_index = char_index.min(ascii_chars.len() - 1);
                
                line.push(ascii_chars.chars().nth(char_index).unwrap_or(' '));
            }
            result.push(line);
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_playback_info() {
        let widget = ImageWidget::new();
        let (current, total, playing) = widget.get_playback_info();
        assert_eq!(current, 0);
        assert_eq!(total, 0);
        assert!(!playing);
    }

    #[test]
    fn test_frame_rate_setting() {
        let mut widget = ImageWidget::new();
        widget.set_frame_rate(30.0);
        assert_eq!(widget.frame_rate, 30.0);
    }
}
