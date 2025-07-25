use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;
use std::collections::VecDeque;

/// Render clipboard history in a list
pub fn render_clipboard_history(
    frame: &mut Frame,
    area: Rect,
    clipboard_history: &VecDeque<String>,
    selected_index: usize,
    primary_color: Color,
) {
    // Create block for clipboard history
    let block = Block::default()
        .title("Clipboard History")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(primary_color));
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    
    if clipboard_history.is_empty() {
        // Show message when there's no history
        let empty_msg = Paragraph::new("No clipboard history items")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, inner_area);
        return;
    }
    
    // Create list items from clipboard history
    let items: Vec<ListItem> = clipboard_history
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let content = if text.len() > 40 {
                format!("{}...", &text[..37])
            } else {
                text.clone()
            };
            
            let style = if i == selected_index {
                Style::default().fg(Color::Black).bg(primary_color)
            } else {
                Style::default()
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("{}: ", i + 1), style),
                Span::styled(content, style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().fg(Color::Black).bg(primary_color));
    
    frame.render_widget(list, inner_area);
    
    // Instructions are defined but not used currently - can be rendered later if needed
    let _instructions = Line::from(vec![
        Span::raw("Press "),
        Span::styled("↑/↓", Style::default().fg(primary_color)),
        Span::raw(" to navigate, "),
        Span::styled("Enter", Style::default().fg(primary_color)),
        Span::raw(" to use, "),
        Span::styled("Esc", Style::default().fg(primary_color)),
        Span::raw(" to cancel"),
    ]);
}
