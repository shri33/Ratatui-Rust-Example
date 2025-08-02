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

#[derive(PartialEq, Debug)]
enum InputMode {
    Navigation,
    Editing,
}

struct App {
    name: String,
    email: String,
    input_mode: InputMode,
    active_field: usize, // 0 = name, 1 = email
}

impl Default for App {
    fn default() -> Self {
        Self {
            name: String::new(),
            email: String::new(),
            input_mode: InputMode::Navigation,
            active_field: 0,
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(f.area());

    let title = Paragraph::new("Simple Test - Tab to navigate, Enter to edit, Type to input")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let name_style = if app.active_field == 0 && app.input_mode == InputMode::Editing {
        Style::default().fg(Color::Yellow)
    } else if app.active_field == 0 {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let name_widget = Paragraph::new(app.name.as_str())
        .style(name_style)
        .block(Block::default().title("Name").borders(Borders::ALL));
    f.render_widget(name_widget, chunks[1]);

    let email_style = if app.active_field == 1 && app.input_mode == InputMode::Editing {
        Style::default().fg(Color::Yellow)
    } else if app.active_field == 1 {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let email_widget = Paragraph::new(app.email.as_str())
        .style(email_style)
        .block(Block::default().title("Email").borders(Borders::ALL));
    f.render_widget(email_widget, chunks[2]);

    let info = Paragraph::new(format!("Mode: {:?}, Field: {}", app.input_mode, app.active_field))
        .block(Block::default().title("Debug").borders(Borders::ALL));
    f.render_widget(info, chunks[3]);

    // Set cursor position when editing
    if app.input_mode == InputMode::Editing {
        let (x, y) = if app.active_field == 0 {
            (chunks[1].x + app.name.len() as u16 + 1, chunks[1].y + 1)
        } else {
            (chunks[2].x + app.email.len() as u16 + 1, chunks[2].y + 1)
        };
        f.set_cursor_position(ratatui::layout::Position::new(x, y));
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Navigation => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Tab => {
                        app.active_field = if app.active_field == 0 { 1 } else { 0 };
                    }
                    KeyCode::Enter => {
                        app.input_mode = InputMode::Editing;
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter | KeyCode::Esc => {
                        app.input_mode = InputMode::Navigation;
                    }
                    KeyCode::Char(c) => {
                        if app.active_field == 0 {
                            app.name.push(c);
                        } else {
                            app.email.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if app.active_field == 0 && !app.name.is_empty() {
                            app.name.pop();
                        } else if app.active_field == 1 && !app.email.is_empty() {
                            app.email.pop();
                        }
                    }
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
