//! Dashboard example showing multiple widgets

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};
use std::io;

struct DashboardApp {
    tabs: Vec<String>,
    tab_index: usize,
    should_quit: bool,
    items: Vec<String>,
    progress: u16,
}

impl DashboardApp {
    fn new() -> Self {
        Self {
            tabs: vec!["Overview".to_string(), "Details".to_string()],
            tab_index: 0,
            should_quit: false,
            items: vec![
                "Item 1".to_string(),
                "Item 2".to_string(),
                "Item 3".to_string(),
            ],
            progress: 45,
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => {
                self.tab_index = (self.tab_index + 1) % self.tabs.len();
            }
            KeyCode::BackTab => {
                if self.tab_index > 0 {
                    self.tab_index -= 1;
                } else {
                    self.tab_index = self.tabs.len() - 1;
                }
            }
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(frame.area());

        // Render tabs
        let tab_items: Vec<Line> = self
            .tabs
            .iter()
            .map(|t| Line::from(Span::styled(t, Style::default().fg(Color::White))))
            .collect();

        let tabs = Tabs::new(tab_items)
            .select(self.tab_index)
            .block(Block::default().borders(Borders::ALL).title("Tabs"));
        frame.render_widget(tabs, chunks[0]);

        // Render content based on selected tab
        match self.tab_index {
            0 => self.render_overview(frame, chunks[1]),
            1 => self.render_details(frame, chunks[1]),
            _ => {}
        }
    }

    fn render_overview(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Render gauge
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Progress"))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(self.progress as f64 / 100.0);
        frame.render_widget(gauge, chunks[0]);

        // Render paragraph
        let text = Text::from("This is the overview tab. It shows a high-level dashboard with key metrics and status information.");
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Summary"))
            .style(Style::default().fg(Color::White));
        frame.render_widget(paragraph, chunks[1]);
    }

    fn render_details(&self, frame: &mut Frame, area: Rect) {
        // Render list
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|i| ListItem::new(i.as_str()))
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        frame.render_widget(list, area);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = DashboardApp::new();

    // Main loop
    loop {
        terminal.draw(|f| app.render(f))?;

        if let Event::Key(key) = event::read()? {
            app.on_key(key.code);
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
