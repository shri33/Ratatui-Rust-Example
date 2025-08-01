use std::io;
use crossterm::{
    event::{self, Event, KeyCode},
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
    name: String,
    email: String,
    selection: usize,
    editing_field: Option<usize>, // 0 = name, 1 = email, None = navigation
    message: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            name: String::new(),
            email: String::new(),
            selection: 0,
            editing_field: None,
            message: "Use Tab to navigate, Enter to edit, Arrow keys for selection".to_string(),
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
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(f.area());
    
    // Instructions
    let instructions = Paragraph::new("NO KEY FILTERING - Tab=navigate, Enter=edit, Q=quit")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Instructions"));
    f.render_widget(instructions, chunks[0]);
    
    // Name field
    let name_style = if app.editing_field == Some(0) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    let name = Paragraph::new(format!("Name: {}", app.name))
        .style(name_style)
        .block(Block::default().borders(Borders::ALL).title("Name Field"));
    f.render_widget(name, chunks[1]);
    
    // Email field
    let email_style = if app.editing_field == Some(1) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    let email = Paragraph::new(format!("Email: {}", app.email))
        .style(email_style)
        .block(Block::default().borders(Borders::ALL).title("Email Field"));
    f.render_widget(email, chunks[2]);
    
    // Selection
    let options = ["Yes", "No", "Maybe", "Other"];
    let selection_text = format!("Selection: {}", options[app.selection]);
    let selection = Paragraph::new(selection_text)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Selection (Left/Right arrows)"));
    f.render_widget(selection, chunks[3]);
    
    // Debug
    let debug = Paragraph::new(app.message.as_str())
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Debug"));
    f.render_widget(debug, chunks[4]);
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();
    let mut current_field = 0; // 0=name, 1=email, 2=selection
    
    loop {
        terminal.draw(|f| ui(f, &app))?;
        
        if let Event::Key(key) = event::read()? {
            app.message = format!("Key pressed: {:?}", key.code);
            
            match app.editing_field {
                None => {
                    // Navigation mode
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Tab => {
                            current_field = (current_field + 1) % 3;
                            app.message = format!("Navigated to field {}", current_field);
                        }
                        KeyCode::Enter => {
                            if current_field < 2 {
                                app.editing_field = Some(current_field);
                                app.message = format!("Started editing field {}", current_field);
                            }
                        }
                        KeyCode::Left => {
                            if current_field == 2 && app.selection > 0 {
                                app.selection -= 1;
                                app.message = format!("Selection changed to {}", app.selection);
                            }
                        }
                        KeyCode::Right => {
                            if current_field == 2 && app.selection < 3 {
                                app.selection += 1;
                                app.message = format!("Selection changed to {}", app.selection);
                            }
                        }
                        _ => {}
                    }
                }
                Some(field) => {
                    // Editing mode
                    match key.code {
                        KeyCode::Enter | KeyCode::Esc => {
                            app.editing_field = None;
                            app.message = "Stopped editing".to_string();
                        }
                        KeyCode::Char(c) => {
                            match field {
                                0 => {
                                    app.name.push(c);
                                    app.message = format!("Added '{}' to name: '{}'", c, app.name);
                                }
                                1 => {
                                    app.email.push(c);
                                    app.message = format!("Added '{}' to email: '{}'", c, app.email);
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Backspace => {
                            match field {
                                0 => {
                                    if !app.name.is_empty() {
                                        app.name.pop();
                                        app.message = format!("Backspace in name: '{}'", app.name);
                                    }
                                }
                                1 => {
                                    if !app.email.is_empty() {
                                        app.email.pop();
                                        app.message = format!("Backspace in email: '{}'", app.email);
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    println!("Final values - Name: '{}', Email: '{}', Selection: {}", 
             app.name, app.email, app.selection);
    Ok(())
}
