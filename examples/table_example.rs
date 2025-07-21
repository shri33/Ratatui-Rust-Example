//! Table widget example
//!
//! Demonstrates an interactive table with sorting, filtering, and selection.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use std::io;

#[derive(Copy, Clone, PartialEq)]
enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }
}

struct TableApp {
    state: TableState,
    items: Vec<Vec<String>>,
    headers: Vec<String>,
    sort_index: usize,
    sort_direction: SortDirection,
    should_quit: bool,
}

impl TableApp {
    fn new() -> Self {
        let headers = vec![
            "ID".to_string(),
            "Name".to_string(),
            "Category".to_string(),
            "Price".to_string(),
            "Quantity".to_string(),
        ];

        let items = vec![
            vec![
                "001".to_string(),
                "Laptop".to_string(),
                "Electronics".to_string(),
                "$999.99".to_string(),
                "10".to_string(),
            ],
            vec![
                "002".to_string(),
                "Desk Chair".to_string(),
                "Furniture".to_string(),
                "$189.99".to_string(),
                "25".to_string(),
            ],
            vec![
                "003".to_string(),
                "Coffee Mug".to_string(),
                "Kitchen".to_string(),
                "$12.99".to_string(),
                "100".to_string(),
            ],
            vec![
                "004".to_string(),
                "Headphones".to_string(),
                "Electronics".to_string(),
                "$149.99".to_string(),
                "15".to_string(),
            ],
            vec![
                "005".to_string(),
                "Backpack".to_string(),
                "Accessories".to_string(),
                "$79.99".to_string(),
                "20".to_string(),
            ],
            vec![
                "006".to_string(),
                "Keyboard".to_string(),
                "Electronics".to_string(),
                "$59.99".to_string(),
                "30".to_string(),
            ],
        ];

        let mut state = TableState::default();
        state.select(Some(0));

        Self {
            state,
            items,
            headers,
            sort_index: 0,
            sort_direction: SortDirection::Ascending,
            should_quit: false,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn sort(&mut self) {
        let idx = self.sort_index;
        self.items.sort_by(|a, b| {
            let cmp = a[idx].cmp(&b[idx]);
            match self.sort_direction {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });
    }

    fn next_sort_column(&mut self) {
        self.sort_index = (self.sort_index + 1) % self.headers.len();
        self.sort();
    }

    fn toggle_sort(&mut self) {
        self.sort_direction = self.sort_direction.toggle();
        self.sort();
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Down => {
                self.next();
            }
            KeyCode::Up => {
                self.previous();
            }
            KeyCode::Tab => {
                self.next_sort_column();
            }
            KeyCode::Char('s') => {
                self.toggle_sort();
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new("Interactive Table Example")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Table
        let header_cells = self.headers.iter().enumerate().map(|(i, h)| {
            let cell = Cell::from(h.clone());
            if i == self.sort_index {
                let sort_indicator = match self.sort_direction {
                    SortDirection::Ascending => " ↑",
                    SortDirection::Descending => " ↓",
                };
                Cell::from(format!("{}{}", h, sort_indicator))
                    .style(Style::default().fg(Color::Yellow))
            } else {
                cell.style(Style::default().fg(Color::White))
            }
        });

        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::DarkGray))
            .height(1);

        let rows = self.items.iter().map(|item| {
            let cells = item.iter().map(|c| Cell::from(c.clone()));
            Row::new(cells).height(1)
        });

        let widths = [
            Constraint::Percentage(10),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Inventory"))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, chunks[1], &mut self.state);

        // Help text
        let help = Paragraph::new("Press ↑/↓ to navigate, Tab to change sort column, s to toggle sort direction, q to quit")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(help, chunks[2]);
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
    let mut app = TableApp::new();
    app.sort();

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
