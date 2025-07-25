//! Interactive table example with row and column highlighting
//! 
//! Features:
//! - Arrow key navigation (up/down for rows, left/right for columns)
//! - Shift + Arrow keys for multi-selection
//! - Visual highlighting of selected row and column

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal, Frame,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use tui_image_viewer::widgets::table::{create_demo_table, InteractiveTable};

struct App {
    table: InteractiveTable,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            table: create_demo_table(),
            should_quit: false,
        }
    }

    fn handle_key_event(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            _ => {
                let shift_pressed = modifiers.contains(KeyModifiers::SHIFT);
                self.table.handle_key_with_shift(key, shift_pressed);
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(4),
        ])
        .split(f.area());

    // Render the interactive table
    app.table.render(f, chunks[0]);

    // Instructions
    let selection_count = app.table.get_selection_count();
    let instructions = format!(
        "Navigation: ↑↓ rows, ←→ columns | Shift+Arrow: multi-select ({} selected) | c: clear | q: quit",
        selection_count
    );
    
    let instructions_widget = Paragraph::new(instructions)
        .block(Block::default().borders(Borders::ALL).title("Instructions"))
        .style(Style::default().fg(Color::Yellow));
    
    f.render_widget(instructions_widget, chunks[1]);
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
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key_event(key.code, key.modifiers);
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
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