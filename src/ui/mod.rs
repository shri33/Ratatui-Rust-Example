use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Tabs};
use ratatui::Frame;

use crate::app::App;
use crate::widgets::image::{render_image, ImageWidget, ImageQuality};
use crate::widgets::input::{render_input, render_emoji_picker, InputMode};
use crate::widgets::table::render_table;
use crate::widgets::clipboard::render_clipboard_history;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
        ])
        .split(frame.size());

    // Get theme colors
    let (primary_color, _secondary_color, _accent_color) = app.get_theme_colors();

    // Render tabs with theme colors
    let titles = ["Table", "Image", "User Input"];
    
    let tab_titles: Vec<Line> = titles.iter().map(|t| Line::from(*t)).collect();
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(primary_color)))
        .select(app.active_tab)
        .highlight_style(Style::default().fg(Color::Black).bg(primary_color));
    frame.render_widget(tabs, chunks[0]);

    // Render content based on active tab
    match app.active_tab {
        0 => {
            // Table tab
            render_table(frame, chunks[1], &mut app.table_state);
        }
        1 => {
            // Image tab
            render_image(frame, chunks[1], primary_color);
        }
        2 => {
            // User input tab - handles different input modes
            match app.input_mode {
                InputMode::EmojiPicker => {
                    render_emoji_picker(
                        frame, 
                        chunks[1], 
                        app.emoji_category_index,
                        app.emoji_index,
                        primary_color
                    );
                },
                InputMode::ClipboardHistory => {
                    render_clipboard_history(
                        frame,
                        chunks[1],
                        &app.clipboard_history,
                        app.clipboard_history_index,
                        primary_color
                    );
                },
                _ => {
                    render_input(
                        frame,
                        chunks[1],
                        app.input_mode,
                        app.selected_input,
                        &app.text_input,
                        &app.emoji_input,
                        &app.hyperlink_input,
                        primary_color
                    );
                }
            }
        }
        _ => {}
    }
}

/// Render image widget with primary color theming
pub fn render_image(frame: &mut Frame, area: Rect, primary_color: Color) -> IoResult<()> {
    let mut widget = ImageWidget::with_quality(ImageQuality::Medium);
    
    // Try to load a demo image if available
    if std::path::Path::new("assets/demo.jpg").exists() {
        let _ = widget.load_image("assets/demo.jpg");
    } else if std::path::Path::new("demo.png").exists() {
        let _ = widget.load_image("demo.png");
    }
    
    widget.render(frame, area)
}

/// Fixed render_input function signature to match usage
pub fn render_input(frame: &mut Frame, area: Rect, app: &crate::app::App) -> IoResult<()> {
    // Implementation should use the existing InputWidget
    let mut input_widget = crate::widgets::input::InputWidget::new();
    input_widget.render(frame, area)
}