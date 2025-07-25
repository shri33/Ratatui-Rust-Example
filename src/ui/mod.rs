use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::App;
use crate::widgets::image::{ImageWidget, ImageQuality};
use crate::widgets::input::InputWidget;

/// Main UI rendering function
pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
        ])
        .split(frame.area());

    // Get theme colors - use app.active_tab if it exists, otherwise default to 0
    let active_tab = app.active_tab; // Direct access if it's usize
    let primary_color = app.primary_color();

    // Render tabs
    let titles = ["Table", "Image", "User Input"];
    let tab_titles: Vec<Line> = titles.iter().map(|t| Line::from(*t)).collect();
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL))
        .select(active_tab)
        .highlight_style(Style::default().fg(Color::Black).bg(primary_color));
    frame.render_widget(tabs, chunks[0]);

    // Render content based on active tab
    match active_tab {
        0 => render_table(frame, chunks[1]),
        1 => render_image(frame, chunks[1]),
        2 => render_input(frame, chunks[1]),
        _ => {}
    }
}

/// Render table widget
fn render_table(frame: &mut Frame, area: Rect) {
    let paragraph = Paragraph::new("Table view - Coming soon!")
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .style(Style::default().fg(Color::White));
    frame.render_widget(paragraph, area);
}

/// Render image widget  
fn render_image(frame: &mut Frame, area: Rect) {
    let mut widget = ImageWidget::with_quality(ImageQuality::Medium);
    let _ = widget.render(frame, area);
}

/// Render input widget
fn render_input(frame: &mut Frame, area: Rect) {
    let input_widget = InputWidget::new();
    let _ = input_widget.render(frame, area);
}
