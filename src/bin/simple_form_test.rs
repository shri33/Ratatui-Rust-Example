use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};

#[derive(PartialEq, Copy, Clone, Debug)]
enum InputMode {
    Navigation,
    Editing,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum InputField {
    Name,
    Email,
    Selection,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum SelectOption {
    Yes,
    No,
    Maybe,
    Other,
}

struct App {
    name: String,
    email: String,
    input_mode: InputMode,
    active_field: InputField,
    selected_option: SelectOption,
}

impl Default for App {
    fn default() -> Self {
        Self {
            name: String::new(),
            email: String::new(),
            input_mode: InputMode::Navigation,
            active_field: InputField::Name,
            selected_option: SelectOption::Yes,
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
        ].as_ref())
        .split(f.area());
    
    // Instructions
    let instructions = format!(
        "Mode: {:?} | Field: {:?} | Controls: Tab=navigate, Enter=edit/stop, Q=quit, Arrow keys=select",
        app.input_mode, app.active_field
    );
    let title = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);
    
    // Name input
    let name_style = if app.active_field == InputField::Name {
        if app.input_mode == InputMode::Editing {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        }
    } else {
        Style::default()
    };
    
    let name_block = Block::default()
        .title(Span::styled("Name", name_style))
        .borders(Borders::ALL)
        .border_style(if app.active_field == InputField::Name {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });
    
    let name_text = Paragraph::new(app.name.as_str())
        .block(name_block);
    f.render_widget(name_text, chunks[1]);
    
    // Email input
    let email_style = if app.active_field == InputField::Email {
        if app.input_mode == InputMode::Editing {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        }
    } else {
        Style::default()
    };
    
    let email_block = Block::default()
        .title(Span::styled("Email", email_style))
        .borders(Borders::ALL)
        .border_style(if app.active_field == InputField::Email {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });
    
    let email_text = Paragraph::new(app.email.as_str())
        .block(email_block);
    f.render_widget(email_text, chunks[2]);
    
    // Selection
    let selection_style = if app.active_field == InputField::Selection {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    
    let options = ["Yes", "No", "Maybe", "Other"];
    let selected_idx = match app.selected_option {
        SelectOption::Yes => 0,
        SelectOption::No => 1,
        SelectOption::Maybe => 2,
        SelectOption::Other => 3,
    };
    
    let tabs = Tabs::new(
        options
            .iter()
            .map(|t| Line::from(Span::raw(*t)))
            .collect::<Vec<_>>(),
    )
    .select(selected_idx)
    .block(Block::default()
        .title(Span::styled("Selection (use Left/Right arrows)", selection_style))
        .borders(Borders::ALL)
        .border_style(if app.active_field == InputField::Selection {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        }))
    .style(Style::default())
    .highlight_style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(tabs, chunks[3]);
    
    // Debug info
    let debug_info = format!(
        "Debug: name='{}' email='{}' selection={:?}",
        app.name, app.email, app.selected_option
    );
    let debug = Paragraph::new(debug_info)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().title("Debug").borders(Borders::ALL));
    f.render_widget(debug, chunks[4]);
    
    // Set cursor position when editing
    if app.input_mode == InputMode::Editing {
        match app.active_field {
            InputField::Name => {
                f.set_cursor_position(ratatui::layout::Position::new(
                    chunks[1].x + app.name.len() as u16 + 1, 
                    chunks[1].y + 1
                ));
            }
            InputField::Email => {
                f.set_cursor_position(ratatui::layout::Position::new(
                    chunks[2].x + app.email.len() as u16 + 1, 
                    chunks[2].y + 1
                ));
            }
            _ => {}
        }
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> std::io::Result<()> {
    let mut app = App::default();
    
    loop {
        terminal.draw(|f| ui(f, &app))?;
        
        if let Event::Key(key) = event::read()? {
            // Only process key press events
            if key.kind != KeyEventKind::Press {
                continue;
            }
            
            match app.input_mode {
                InputMode::Navigation => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Tab => {
                        app.active_field = match app.active_field {
                            InputField::Name => InputField::Email,
                            InputField::Email => InputField::Selection,
                            InputField::Selection => InputField::Name,
                        };
                    }
                    KeyCode::Up => {
                        app.active_field = match app.active_field {
                            InputField::Name => InputField::Selection,
                            InputField::Email => InputField::Name,
                            InputField::Selection => InputField::Email,
                        };
                    }
                    KeyCode::Down => {
                        app.active_field = match app.active_field {
                            InputField::Name => InputField::Email,
                            InputField::Email => InputField::Selection,
                            InputField::Selection => InputField::Name,
                        };
                    }
                    KeyCode::Enter => {
                        if app.active_field != InputField::Selection {
                            app.input_mode = InputMode::Editing;
                        }
                    }
                    KeyCode::Left => {
                        if app.active_field == InputField::Selection {
                            app.selected_option = match app.selected_option {
                                SelectOption::Yes => SelectOption::Other,
                                SelectOption::No => SelectOption::Yes,
                                SelectOption::Maybe => SelectOption::No,
                                SelectOption::Other => SelectOption::Maybe,
                            };
                        }
                    }
                    KeyCode::Right => {
                        if app.active_field == InputField::Selection {
                            app.selected_option = match app.selected_option {
                                SelectOption::Yes => SelectOption::No,
                                SelectOption::No => SelectOption::Maybe,
                                SelectOption::Maybe => SelectOption::Other,
                                SelectOption::Other => SelectOption::Yes,
                            };
                        }
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter | KeyCode::Esc => {
                        app.input_mode = InputMode::Navigation;
                    }
                    KeyCode::Char(c) => {
                        match app.active_field {
                            InputField::Name => {
                                app.name.push(c);
                            }
                            InputField::Email => {
                                app.email.push(c);
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Backspace => {
                        match app.active_field {
                            InputField::Name => {
                                app.name.pop();
                            }
                            InputField::Email => {
                                app.email.pop();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let res = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}")
    }

    Ok(())
}
