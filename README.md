# Ratatui & Rust Examples

A collection of advanced examples for the Ratatui terminal UI framework.

## Features

- **Table Rendering**: Interactive tables with sorting, filtering, and selection
- **Image Viewer**: Display and convert images to ASCII art in the terminal
- **Video Player**: Play MP4 videos directly in the terminal (requires FFmpeg)
- **Emoji Picker**: Browse and select emojis with category navigation
- **Text Input**: Text input fields with clipboard support
- **Hyperlinks**: Clickable links in terminal applications

## Requirements

### Core Requirements
- Rust 1.60+
- A terminal with RGB color support
- A modern font with emoji support (for the emoji picker)

### Optional Requirements
- FFmpeg (only for video playback)
- pkg-config (for building FFmpeg bindings)

## Running the Examples Without FFmpeg

If you don't have FFmpeg installed, you can still run most examples:

```bash
# Run the table example (works without FFmpeg)
cargo run --example table_example --no-default-features

# Run the text input example (works without FFmpeg)
cargo run --example text_input --no-default-features

# Run the hyperlinks example (works without FFmpeg)
cargo run --example hyperlinks --no-default-features

# Run the emoji picker (works without FFmpeg)
cargo run --example emoji_picker --no-default-features

# Run the dashboard example (works without FFmpeg)
cargo run --example dashboard --no-default-features
```

## Running Video Examples (Requires FFmpeg)

To run examples that use FFmpeg (video player):

1. **Install FFmpeg**:
   
   **Windows**:
   - Option 1: Download from [ffmpeg.org](https://ffmpeg.org/download.html) and add to PATH
   - Option 2: Use Chocolatey: `choco install ffmpeg`
   - Option 3: Use vcpkg: 
     ```
     git clone https://github.com/Microsoft/vcpkg.git
     cd vcpkg
     .\bootstrap-vcpkg.bat
     .\vcpkg.exe install ffmpeg:x64-windows
     .\vcpkg.exe integrate install
     ```

   **macOS**:
   - Use Homebrew: `brew install ffmpeg pkg-config`

   **Linux**:
   - Ubuntu/Debian: `sudo apt install ffmpeg libavcodec-dev libavformat-dev libavutil-dev pkg-config`
   - Fedora: `sudo dnf install ffmpeg-devel pkgconf-pkg-config`

2. **Run the example**:
   ```bash
   cargo run --example video_player --features=video
   ```

## Testing on Different Terminals

This project has been tested on:
- Windows Terminal
- iTerm2 (macOS)
- Alacritty
- Kitty

For the best experience with image and video rendering, use a terminal that supports:
- 24-bit true color
- Kitty graphics protocol or Sixel graphics (for high-resolution images)

## Troubleshooting

If you encounter build errors related to FFmpeg:

1. Make sure pkg-config is installed and in your PATH
2. Set the PKG_CONFIG_PATH environment variable to point to your FFmpeg installation
3. Try running examples without FFmpeg using the `--no-default-features` flag

## License

MIT

## Credits

Created as an open-source contribution to the Ratatui ecosystem.


