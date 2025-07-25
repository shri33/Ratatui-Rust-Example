# Enhanced Terminal Video Player

A feature-rich terminal-based video player built with Rust, featuring file browsing, frame-by-frame navigation, audio playback, and comprehensive error handling.

## Features

### ✅ Implemented Improvements
- **File Browser**: Navigate directories and select video files
- **Proper Seeking/Indexing**: Frame caching system for efficient navigation
- **Audio Support**: Volume controls and mute functionality (optional feature)
- **Better Error Handling**: Comprehensive error types and user feedback
- **Enhanced UI**: Progress bars, metadata display, help system
- **Frame Caching**: LRU cache for smooth playback
- **Multiple Format Support**: MP4, AVI, MKV, MOV, WMV, FLV, WebM, M4V

### 🎮 Controls

#### File Browser
- `↑/↓` - Navigate files and directories
- `Enter` - Select file or enter directory
- `Esc` - Clear error messages

#### Video Player
- `Space` - Play/Pause video
- `←/→` - Previous/Next frame
- `D` - Display current frame in terminal
- `B` - Return to file browser

#### Audio Controls (with audio feature)
- `+/=` - Increase volume
- `-` - Decrease volume  
- `M` - Toggle mute

#### General
- `H` - Show/hide help screen
- `Q` - Quit application
- `Esc` - Close error dialogs

## Installation & Setup

### Prerequisites

#### FFmpeg Installation

**Windows:**
```bash
# Option 1: Direct download
1. Download FFmpeg from https://ffmpeg.org/download.html
2. Extract to C:\ffmpeg
3. Add C:\ffmpeg\bin to your PATH
4. Set PKG_CONFIG_PATH=C:\ffmpeg\lib\pkgconfig

# Option 2: Using vcpkg (recommended)
vcpkg install ffmpeg:x64-windows
vcpkg integrate install
```

**macOS:**
```bash
brew install ffmpeg pkg-config
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install ffmpeg libavcodec-dev libavformat-dev libavutil-dev libavfilter-dev libavdevice-dev libswscale-dev libswresample-dev pkg-config
```

**Linux (Arch):**
```bash
sudo pacman -S ffmpeg pkg-config
```

### Building

#### Basic version (file browser only):
```bash
cargo build --release
cargo run
```

#### With video support:
```bash
cargo build --release --features=video
cargo run --features=video
```

#### With video and audio support:
```bash
cargo build --release --features=video,audio
cargo run --features=video,audio
```

## Project Structure

```
src/
├── main.rs              # Main application with all features
└── lib.rs               # Optional library structure

Cargo.toml               # Dependencies and features
README.md               # This file
```

## Architecture

### Core Components

1. **VideoPlayerApp**: Main application state manager
2. **FileBrowser**: File system navigation with video file filtering
3. **FrameCache**: LRU cache for efficient frame storage
4. **VideoMetadata**: Comprehensive video information
5. **PlayerError**: Structured error handling

### Feature Flags

- `video`: Enables FFmpeg integration and video playback
- `audio`: Enables audio playback controls (requires `video`)

### Error Handling

The application uses structured error types:
- `FileNotFound`: File system errors
- `UnsupportedFormat`: Video format issues  
- `DecodingError`: FFmpeg/codec problems
- `AudioError`: Audio system failures
- `IoError`: General I/O problems

## Usage Examples

### Basic File Navigation
1. Start the application
2. Use arrow keys to navigate directories
3. Press Enter on video files to load them
4. Use 'B' key to return to browser from player

### Video Playback
1. Load a video file from the browser
2. Press Space to play/pause
3. Use left/right arrows for frame navigation
4. Press 'D' to display current frame in terminal

### Audio Control (if enabled)
1. Use +/- keys to adjust volume
2. Press 'M' to mute/unmute
3. Volume and mute status shown in status bar

## Performance Considerations

- **Frame Caching**: Default cache size is 50 frames (adjustable)
- **Memory Usage**: Cached frames consume significant memory
- **Seeking Performance**: Initial seeking may be slow for large files
- **Terminal Display**: Frame display via `viuer` requires terminal image support

## Supported Formats

- **Video**: MP4, AVI, MKV, MOV, WMV, FLV, WebM, M4V
- **Codecs**: All FFmpeg-supported codecs
- **Audio**: PCM, MP3, AAC, etc. (with audio feature)

## Troubleshooting

### Common Issues

**"FFmpeg not found" error:**
- Ensure FFmpeg is properly installed and in PATH
- Verify PKG_CONFIG_PATH is set correctly on Windows
- Try using vcpkg on Windows for easier setup

**"No video stream found" error:**
- File may be corrupted or unsupported format
- Try with a known-good MP4 file first

**Audio not working:**
- Ensure compiled with `--features=video,audio`
- Check audio system availability
- Some video files may not contain audio tracks

**Poor performance:**
- Reduce frame cache size in code if memory constrained
- Use lower resolution videos for better terminal display
- Ensure SSD storage for better seeking performance

### Debug Mode

For additional debugging information:
```bash
RUST_LOG=debug cargo run --features=video,audio
```

## Development

### Adding New Features

1. **New Video Formats**: Extend `is_video_file()` method
2. **Additional Controls**: Add key handlers in `handle_*_key()` methods
3. **UI Improvements**: Modify render methods in `VideoPlayerApp`
4. **Performance**: Adjust cache size or implement smarter caching

### Code Organization

- **Conditional Compilation**: Features are properly gated with `#[cfg(feature = "...")]`
- **Error Propagation**: Uses `Result<T, PlayerError>` throughout
- **State Management**: Clean separation between browser and player states
- **Memory Management**: RAII and explicit cleanup for FFmpeg resources

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all features compile and run
5. Submit a pull request

## License

This project is open source. Choose an appropriate license for your needs.

## Dependencies

### Required
- `crossterm`: Cross-platform terminal manipulation
- `ratatui`: Terminal user interface framework

### Optional (with features)
- `ffmpeg-next`: FFmpeg Rust bindings
- `image`: Image processing
- `viuer`: Terminal image display
- `rodio`: Audio playback

## Future Enhancements

Potential improvements for future versions:
- Subtitle support
- Playlist management
- Video filters and effects
- Network streaming support
- Multiple audio track selection
- Thumbnail generation
- Video information export