//! Charts demo - Histogram and Bar graphs
//! 
//! Features:
//! - Histogram showing data distribution
//! - Bar chart with sample data
//! - Toggle between different chart views

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{BarChart, Block, Borders, Chart, Dataset, GraphType, Axis, Paragraph},
    symbols,
    text::Span,
    Terminal, Frame,
};
use std::{
    error::Error,
    io,
    time::Duration,
};

struct App {
    should_quit: bool,
    current_view: usize,
}

impl App {
    fn new() -> Self {
        Self {
            should_quit: false,
            current_view: 0,
        }
    }

    fn next_view(&mut self) {
        self.current_view = (self.current_view + 1) % 2;
    }

    fn previous_view(&mut self) {
        if self.current_view == 0 {
            self.current_view = 1;
        } else {
            self.current_view -= 1;
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title
    let title = match app.current_view {
        0 => "Bar Chart Demo",
        1 => "Line Chart (Histogram) Demo",
        _ => "Charts Demo",
    };
    
    let title_paragraph = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(title_paragraph, chunks[0]);

    // Charts
    match app.current_view {
        0 => render_bar_chart(f, chunks[1]),
        1 => render_line_chart(f, chunks[1]),
        _ => {}
    }

    // Instructions
    let instructions = Paragraph::new("Tab: Switch view | q: Quit")
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(instructions, chunks[2]);
}

fn render_bar_chart(f: &mut Frame, area: Rect) {
    let data = vec![
        ("Jan", 20),
        ("Feb", 25),
        ("Mar", 30),
        ("Apr", 35),
        ("May", 28),
        ("Jun", 32),
        ("Jul", 40),
        ("Aug", 38),
        ("Sep", 33),
        ("Oct", 29),
        ("Nov", 26),
        ("Dec", 31),
    ];

    let bar_chart = BarChart::default()
        .block(Block::default().borders(Borders::ALL).title("Monthly Sales"))
        .data(&data)
        .bar_width(4)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::Green))
        .value_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    
    f.render_widget(bar_chart, area);
}

fn render_line_chart(f: &mut Frame, area: Rect) {
    // Sample histogram data - frequency distribution
    let data: Vec<(f64, f64)> = vec![
        (0.0, 2.0),
        (1.0, 5.0),
        (2.0, 8.0),
        (3.0, 12.0),
        (4.0, 15.0),
        (5.0, 18.0),
        (6.0, 20.0),
        (7.0, 18.0),
        (8.0, 15.0),
        (9.0, 12.0),
        (10.0, 8.0),
        (11.0, 5.0),
        (12.0, 2.0),
    ];

    let datasets = vec![Dataset::default()
        .name("Distribution")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&data)];

    let chart = Chart::new(datasets)
        .block(Block::default().borders(Borders::ALL).title("Data Distribution (Histogram)"))
        .x_axis(
            Axis::default()
                .title("Value")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 12.0])
                .labels(vec![
                    Span::from("0"),
                    Span::from("6"),
                    Span::from("12"),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("Frequency")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 25.0])
                .labels(vec![
                    Span::from("0"),
                    Span::from("10"),
                    Span::from("20"),
                ]),
        );

    f.render_widget(chart, area);
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if crossterm::event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Tab => {
                            app.next_view();
                        }
                        KeyCode::BackTab => {
                            app.previous_view();
                        }
                        _ => {}
                    }
                }
            }
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