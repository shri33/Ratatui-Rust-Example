use std::io;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

struct App {
    input: String,
    message: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input: String::new(),
            message: "Press any key to test input...".to_string(),
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(f.area());
    
    // Instructions
    let instructions = Paragraph::new("KEYBOARD TEST - Type characters, they should appear below. Press Q to quit.")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Instructions"));
    f.render_widget(instructions, chunks[0]);
    
    // Input field
    let input_text = format!("Input: {}", app.input);
    let input = Paragraph::new(input_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Type Here"));
    f.render_widget(input, chunks[1]);
    
    // Debug info
    let debug = Paragraph::new(app.message.as_str())
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Debug"));
    f.render_widget(debug, chunks[2]);
    
    // Set cursor
    f.set_cursor_position(ratatui::layout::Position::new(
        chunks[1].x + app.input.len() as u16 + 8, 
        chunks[1].y + 1
    ));
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();
    
    loop {
        terminal.draw(|f| ui(f, &app))?;
        
        if let Event::Key(key) = event::read()? {
            // Only process key press events
            if key.kind != KeyEventKind::Press {
                continue;
            }
            
            app.message = format!("Last key: {:?}", key.code);
            
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char(c) => {
                    app.input.push(c);
                    app.message = format!("Added character: '{}' - Total: '{}'", c, app.input);
                }
                KeyCode::Backspace => {
                    if !app.input.is_empty() {
                        app.input.pop();
                        app.message = format!("Backspace pressed - Current: '{}'", app.input);
                    }
                }
                KeyCode::Enter => {
                    app.message = format!("Enter pressed - Input was: '{}'", app.input);
                    app.input.clear();
                }
                _ => {
                    app.message = format!("Key pressed: {:?} (not handled)", key.code);
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    println!("Final input was: '{}'", app.input);
    Ok(())
}
