# 🦀 Ratatui Advanced UI Demo

A comprehensive terminal user interface demonstration built with Rust and Ratatui, featuring advanced UI components inspired by Vue.js forms and modern CLI patterns.

## 🌟 Features

### ✅ **All Goals Completed:**

- **📥 Interactive Forms**: Vue.js-style input with real-time validation and history scrollback
- **✅ Select Components**: Dropdown, radio buttons, and checkboxes with keyboard navigation  
- **📊 Terminal Charts**: Bar graphs and histograms with toggleable views
- **🔼 Table Navigation**: Row highlighting with Up/Down arrow keys
- **➡️ Column Selection**: Multi-column selection with Shift+Arrow keys
- **🎛️ Modular Architecture**: Clean component separation and reusable widgets

## 🚀 Quick Start

### Run the Main Menu:
```bash
cargo run --bin main_menu
```

### Run Individual Examples:
```bash
# Interactive form with validation
cargo run --bin interactive_form

# Select components (dropdown, radio, checkbox)
cargo run --bin table_example  

# Charts and graphs
cargo run --bin charts_demo

# Advanced table navigation
cargo run --bin interactive_table

# Multi-tab dashboard
cargo run --bin dashboard

# Image viewer
cargo run --bin image_viewer

# Video player
cargo run --bin video_player
```

## 📁 Project Structure

```
src/
├── examples/
│   ├── main_menu.rs          # 🎯 Main showcase menu
│   ├── interactive_form.rs   # 📥 Vue.js-style forms
│   ├── table_example.rs      # ✅ Select components  
│   ├── charts_demo.rs        # 📊 Charts and graphs
│   ├── interactive_table.rs  # 🔼 Advanced table navigation
│   ├── dashboard.rs          # 🎛️ Multi-widget dashboard
│   ├── image_viewer.rs       # 🖼️ High-res image display
│   ├── video_player.rs       # 🎬 Video playback
│   └── emoji_picker.rs       # 😀 Emoji selection
├── widgets/
│   ├── table.rs             # Table widget implementation
│   ├── input.rs             # Input widget with validation
│   └── image.rs             # Image rendering widget
└── ui/
    └── mod.rs               # UI rendering logic
```

## 🎯 Key Features Demonstrated

### 1. **Interactive Forms** (`interactive_form.rs`)
- ✅ Sequence of user prompts (name, email, etc.)
- ✅ Real-time validation with visual feedback
- ✅ Scrollable history pane (like CLI wizards)
- ✅ Form wizard navigation

### 2. **Select Components** (`table_example.rs`)
- ✅ Dropdown selection
- ✅ Radio button groups  
- ✅ Checkbox lists
- ✅ Arrow key navigation
- ✅ Enter to select

### 3. **Terminal Charts** (`charts_demo.rs`)
- ✅ Bar charts with sample data
- ✅ Line charts (histogram style)
- ✅ Toggle between chart types
- ✅ Labeled axes and legends

### 4. **Advanced Table Navigation** (`interactive_table.rs`)
- ✅ Row highlighting (Up/Down arrows)
- ✅ Column highlighting (Left/Right arrows)
- ✅ Multi-selection with Shift+Arrow keys
- ✅ Visual styling for selected cells

### 5. **Dashboard Interface** (`dashboard.rs`)
- ✅ Multi-tab layout
- ✅ Progress gauges
- ✅ List widgets
- ✅ Tab switching with keyboard

## 🎮 Controls

### Global Controls:
- `q` - Quit application
- `Tab` - Switch tabs/views
- `↑↓←→` - Navigation
- `Enter` - Select/Confirm
- `Esc` - Cancel/Back

### Advanced Controls:
- `Shift + ↑↓←→` - Multi-selection (tables)
- `h` - Help/Instructions
- `Space` - Toggle selection (checkboxes)

## 🏗️ Architecture

The project demonstrates clean modular architecture:

- **Widgets**: Reusable UI components
- **Examples**: Standalone demonstrations  
- **UI Logic**: Centralized rendering
- **Event Handling**: Keyboard and mouse input
- **State Management**: Application state patterns

## 🛠️ Technologies

- **Rust** - Systems programming language
- **Ratatui** - Terminal user interface library
- **Crossterm** - Cross-platform terminal manipulation
- **Image** - Image processing and display
- **Tokio** - Async runtime (for video features)

## 📸 Screenshots

Run the examples to see:
- Beautiful terminal-based forms
- Interactive charts and graphs  
- Advanced table navigation
- Modern CLI user experience

## 🤝 Contributing

This project serves as a comprehensive example of Ratatui capabilities. Feel free to:

- Study the code for learning Ratatui
- Use components in your own projects
- Submit improvements and bug fixes
- Add new example demonstrations

## 📜 License

MIT License - feel free to use this code in your projects!

---

**Built with ❤️ using Rust and Ratatui**


