# 🦀 Ratatui Advanced UI & CLI Demo

A comprehensive terminal user interface demonstration built with Rust and Ratatui, featuring Vue.js-style forms, modern CLI patterns, and advanced TUI components.

## 🌟 Features Overview

### ✅ **Complete Implementation - All Goals Achieved:**

- **📥 Vue.js-Style Interactive Forms**: Real-time validation, history scrollback, and professional styling
- **✅ Select Components**: Yes/No/Maybe/Other options with keyboard navigation
- **📊 Embedded Charts**: Bar graphs and histograms integrated into TUI
- **🔼 Advanced Table Navigation**: Row highlighting with Arrow keys
- **➡️ Multi-Column Selection**: Shift+Arrow key multi-selection
- **🖥️ CLI-Style Interface**: Sequential prompts like `npm create vue@latest`
- **⏳ Loading Animations**: Professional spinners and progress indicators
- **� File System Integration**: Campaign generation with file explorer opening
- **🌐 Web Service Simulation**: API call logging and structured data output

## 🚀 Quick Start Guide

### **🎯 CLI-Style Interface** (Vue.js npm Experience)
```bash
# Basic CLI with sequential prompts
cargo run --bin journal_cli authenticate login

# Enhanced CLI with web service simulation
cargo run --bin journal_cli_enhanced authenticate login

# Debug keyboard input issues
cargo run --bin keyboard_test
cargo run --bin no_filter_test
```

### **🎨 TUI-Style Interface** (Rich Visual Components)
```bash
# Interactive form with Vue.js styling
cargo run --bin interactive_form

# Advanced table with multi-selection
cargo run --bin interactive_table

# Charts and visualization demo
cargo run --bin charts_demo

# Complete dashboard
cargo run --bin dashboard

# Main menu showcase
cargo run --bin main_menu
```

### **🖼️ Media and Advanced Features**
```bash
# High-resolution image viewer
cargo run --bin image_viewer

# Video player with controls
cargo run --bin video_player

# Text input examples
cargo run --bin text_input
```
cargo run --bin image_viewer

# Video player
cargo run --bin video_player
```

## 📁 Project Structure

```
src/
├── bin/                      # 🎯 Main Applications
│   ├── journal_cli.rs        # CLI-style sequential prompts
│   ├── journal_cli_enhanced.rs # Enhanced CLI with web services
│   ├── interactive_form.rs   # Vue.js-style TUI form
│   ├── simple_form_test.rs   # Simplified form for debugging
│   ├── keyboard_test.rs      # Basic keyboard input test
│   └── no_filter_test.rs     # Alternative input handling
├── examples/                 # 🎨 UI Component Examples
│   ├── main_menu.rs          # Main showcase menu
│   ├── interactive_table.rs  # Advanced table navigation
│   ├── charts_demo.rs        # Charts and graphs
│   ├── dashboard.rs          # Multi-widget dashboard
│   ├── image_viewer.rs       # High-res image display
│   ├── video_player.rs       # Video playback
│   ├── text_input.rs         # Input field examples
│   ├── table_example.rs      # Basic table rendering
│   ├── emoji_picker.rs       # Emoji selection UI
│   ├── hyperlinks.rs         # Clickable links
│   └── ascii_art.rs          # ASCII art display
├── widgets/                  # 🛠️ Reusable Components
│   ├── table.rs             # Interactive table widget
│   ├── input.rs             # Input widget with validation
│   ├── image.rs             # Image rendering widget
│   ├── clipboard.rs         # Clipboard integration
│   └── mod.rs               # Widget module exports
├── ui/                      # 🎨 UI Logic
│   └── mod.rs               # UI rendering utilities
└── app/                     # ⚙️ Application Logic
    ├── config.rs            # Configuration management
    └── mod.rs               # App module exports
```

## 🎯 Key Features Demonstrated

### **🖥️ CLI-Style Interface** - Sequential Prompt Experience

#### **Basic CLI** (`journal_cli.rs`)
- ✅ Clean terminal interface (no TUI boxes)
- ✅ Sequential email → name → confirmation prompts
- ✅ Real-time input validation with error messages
- ✅ Loading animations with professional spinners
- ✅ Campaign file generation (JSON, HTML, README)
- ✅ Cross-platform file explorer integration
- ✅ Web service simulation with API logging

#### **Enhanced CLI** (`journal_cli_enhanced.rs`)
- ✅ Advanced command parsing
- ✅ Professional HTML email templates
- ✅ Structured JSON data output
- ✅ Comprehensive logging system
- ✅ Multi-step campaign creation workflow

### **🎨 TUI-Style Interface** - Rich Visual Components

#### **Interactive Forms** (`interactive_form.rs`)
- ✅ Vue.js-inspired input styling
- ✅ Real-time validation with ✓/✗ indicators
- ✅ 10-item history scrollback buffer
- ✅ Embedded bar charts and histograms
- ✅ Multi-field navigation with Tab/Arrow keys
- ✅ Professional color-coded feedback

#### **Advanced Table Navigation** (`interactive_table.rs`)
- ✅ Row highlighting with Up/Down arrows
- ✅ Column highlighting with Left/Right arrows
- ✅ Multi-cell selection with Shift+Arrow keys
- ✅ Visual styling for selected regions
- ✅ Keyboard-only navigation

#### **Charts & Visualization** (`charts_demo.rs`)
- ✅ Bar charts with labeled data
- ✅ Histogram-style visualizations
- ✅ Toggle between chart types
- ✅ Responsive layout and styling
- ✅ Interactive data highlighting

#### **Select Components** (Multiple Files)
- ✅ Yes/No/Maybe/Other selection tabs
- ✅ Arrow key navigation between options
- ✅ Visual highlighting of selected items
- ✅ Color-coded selection states

### **🛠️ Debug and Testing Tools**

#### **Input Debugging** (`keyboard_test.rs`, `no_filter_test.rs`)
- ✅ Keyboard input validation
- ✅ Event filtering troubleshooting
- ✅ Terminal compatibility testing
- ✅ Real-time debug information

## 🎮 Control Schemes

### **CLI Interface Controls:**
- **Text Input**: Type naturally, Enter to proceed
- **Email Validation**: Automatic with retry on invalid format
- **Yes/No Prompts**: Y/n with Enter confirmation
- **Navigation**: Sequential flow with back/forward options
- **Exit**: Q or Esc to quit at any time

### **TUI Interface Controls:**
- **Field Navigation**: Tab, Shift+Tab, Arrow keys
- **Text Editing**: Enter to start/stop editing
- **Multi-Selection**: Shift+Arrow keys for ranges
- **Chart Interaction**: Space to toggle views
- **Global**: Q/Esc to quit, H for help

### **Advanced Shortcuts:**
- **Forms**: Ctrl+C to copy, validation on blur
- **Tables**: Page Up/Down for quick navigation
- **Charts**: Number keys to jump to specific data points
- **Debug**: D to toggle debug information

## 🏗️ Architecture Patterns

This project demonstrates **two complementary interface paradigms**:

### **1. CLI-Style Pattern** (Sequential Workflow)
```rust
// Clean terminal output with sequential prompts
email = prompt_with_validation("Email: ")?;
name = prompt_with_validation("Name: ")?; 
campaign = prompt_yes_no("Generate campaign?")?;
```

**Benefits:**
- Familiar to users of npm, Vue CLI, create-react-app
- Clean, distraction-free interface
- Perfect for setup wizards and authentication
- Works well in CI/CD and scripted environments

### **2. TUI-Style Pattern** (Rich Visual Interface)
```rust
// Rich visual components with real-time updates
render_form_with_validation(&mut frame, &app_state)?;
render_embedded_charts(&mut frame, &chart_data)?;
handle_multi_selection_input(&key_event, &mut table_state)?;
```

**Benefits:**
- High information density
- Real-time visual feedback
- Complex data manipulation
- Professional application feel

## 🛠️ Technologies & Dependencies

### **Core Technologies:**
- **Rust 2021** - Systems programming language
- **Ratatui 0.28** - Terminal user interface framework
- **Crossterm 0.28** - Cross-platform terminal manipulation

### **Feature Libraries:**
- **Chrono** - Date/time handling for timestamps
- **Serde JSON** - Data serialization for API simulation
- **Regex** - Email validation and text processing
- **Arboard** - Clipboard integration
- **Viuer** - High-quality image display in terminal

### **Development Tools:**
- **Tokio** - Async runtime for video features
- **Anyhow** - Error handling
- **Clap** - Command-line argument parsing

## 📊 Generated Artifacts

The applications create structured output files:

### **Campaign Files** (from CLI tools):
```
campaigns/
├── journal_campaign_user_at_example_com_20241231_123456/
│   ├── campaign.json          # Campaign configuration
│   ├── templates/
│   │   └── welcome_email.html # Professional HTML email
│   └── README.md              # Documentation and next steps
```

### **API Logs** (from web service simulation):
```
logs/
└── api_call_1735684800.json   # Structured API call logging
```

## 🔧 Troubleshooting

### **Input Issues:**
If keyboard input doesn't work in forms:
1. Try: `cargo run --bin keyboard_test`
2. Try: `cargo run --bin no_filter_test`
3. Use Windows Terminal instead of PowerShell
4. Check terminal raw mode support

### **Build Issues:**
```bash
# Clean build
cargo clean && cargo build

# Check specific component
cargo build --bin journal_cli
```

### **Terminal Compatibility:**
- **Recommended**: Windows Terminal, iTerm2, GNOME Terminal
- **Works**: PowerShell, CMD, VS Code Terminal
- **Limited**: Basic terminal emulators

## 📸 Interface Examples

### CLI-Style Output:
```
╔══════════════════════════════════════════════════╗
║     📰 Welcome to Journal                        ║
║     Your personal journaling companion           ║
╚══════════════════════════════════════════════════╝

Welcome to Journal, please provide your Email to create an account
Email: user@example.com
Email: user@example.com ✅
Name: John Doe  
Name: John Doe ✅

Thank you, Journal is creating an account for you. Standby...
⠋ Creating your account
✅ Your account has been created.
```

### TUI-Style Interface:
- Rich bordered components
- Real-time validation indicators
- Embedded charts and graphs
- Multi-column data tables
- Professional color schemes

## 🤝 Contributing

This project serves as a comprehensive showcase of Ratatui capabilities and modern terminal interface patterns. Contributions welcome:

### **Learning Resources:**
- Study the CLI vs TUI pattern implementations
- Use widget components in your own projects
- Reference the Vue.js-style form validation approach
- Learn from the multi-selection table navigation

### **Contributing:**
- Submit bug fixes and improvements
- Add new example demonstrations
- Enhance existing widget functionality
- Improve documentation and examples

### **Project Goals:**
- Demonstrate best practices for terminal UIs
- Provide reusable components for Rust developers
- Showcase both CLI and TUI interface patterns
- Maintain high code quality and documentation

## 📈 Development Status

### **✅ Completed Features:**
- ✅ CLI-style sequential prompt interface
- ✅ Vue.js-inspired interactive forms
- ✅ Advanced table navigation with multi-selection
- ✅ Embedded charts and visualizations
- ✅ File system integration and campaign generation
- ✅ Cross-platform compatibility
- ✅ Comprehensive error handling and validation
- ✅ Debug tools for input troubleshooting

### **🔧 Technical Achievements:**
- ✅ Zero external UI dependencies (pure terminal)
- ✅ Professional loading animations and spinners
- ✅ Real-time input validation
- ✅ Structured data output (JSON, HTML)
- ✅ Cross-platform file explorer integration
- ✅ Memory-efficient widget rendering
- ✅ Keyboard-only navigation throughout

## 🎓 Learning Outcomes

This project demonstrates:

### **Rust Concepts:**
- Advanced pattern matching for UI state management
- Error handling with `Result` types
- Modular architecture with clean separation
- Cross-platform development techniques

### **Terminal UI Concepts:**
- Event-driven programming in terminal environments
- Real-time rendering and state updates
- Keyboard input handling and validation
- Layout management and responsive design

### **Interface Design Patterns:**
- CLI-style sequential workflows
- TUI-style rich visual interfaces
- Form validation and user feedback
- Multi-selection and navigation patterns

## 📜 License

MIT License - feel free to use this code in your projects!

## 🏆 Acknowledgments

- **Ratatui Team** - For the excellent terminal UI framework
- **Crossterm Team** - For cross-platform terminal support
- **Rust Community** - For the amazing ecosystem

---

**Built with ❤️ using Rust and Ratatui**

*This project showcases the power of terminal-based user interfaces and demonstrates that CLI applications can be both beautiful and functional.*


