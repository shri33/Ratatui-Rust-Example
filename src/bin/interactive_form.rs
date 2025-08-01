use std::io;
use std::collections::VecDeque;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Tabs,
        BarChart,
    },
    Frame, Terminal,
};

const HISTORY_SIZE: usize = 10;

#[derive(PartialEq, Copy, Clone)]
enum InputMode {
    Navigation,
    Editing,
}

#[derive(PartialEq, Copy, Clone)]
enum InputField {
    Name,
    Email,
    Selection,
}

#[derive(PartialEq, Copy, Clone)]
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
    history: VecDeque<String>,
    name_valid: bool,
    email_valid: bool,
    highlighted_rows: Vec<usize>,
    highlighted_cols: Vec<usize>,
    data: Vec<(String, u64)>,
}

impl Default for App {
    fn default() -> Self {
        let data = vec![
            ("Jan".to_string(), 45),
            ("Feb".to_string(), 27),
            ("Mar".to_string(), 38),
            ("Apr".to_string(), 63),
            ("May".to_string(), 25),
            ("Jun".to_string(), 42),
        ];
        
        Self {
            name: String::new(),
            email: String::new(),
            input_mode: InputMode::Navigation,
            active_field: InputField::Name,
            selected_option: SelectOption::Yes,
            history: VecDeque::with_capacity(HISTORY_SIZE),
            name_valid: false,
            email_valid: false,
            highlighted_rows: vec![0],
            highlighted_cols: vec![0],
            data,
        }
    }
}

impl App {
    fn validate_email(&mut self) {
        let valid = !self.email.is_empty() &&
                   self.email.contains('@') && 
                   self.email.contains('.') && 
                   self.email.find('@') < self.email.rfind('.') &&
                   !self.email.starts_with('@') &&
                   !self.email.ends_with('.');
        self.email_valid = valid;
    }
    
    fn validate_name(&mut self) {
        self.name_valid = !self.name.trim().is_empty() && self.name.trim().len() >= 2;
    }
    
    fn add_to_history(&mut self, entry: String) {
        self.history.push_front(entry);
        while self.history.len() > HISTORY_SIZE {
            self.history.pop_back();
        }
    }
    
    fn move_highlight(&mut self, direction: KeyCode, with_shift: bool) {
        match direction {
            KeyCode::Up => {
                if !self.highlighted_rows.is_empty() && self.highlighted_rows[0] > 0 {
                    if with_shift {
                        if !self.highlighted_rows.contains(&(self.highlighted_rows[0] - 1)) {
                            self.highlighted_rows.push(self.highlighted_rows[0] - 1);
                        }
                    } else {
                        self.highlighted_rows = vec![self.highlighted_rows[0] - 1];
                    }
                }
            },
            KeyCode::Down => {
                if !self.highlighted_rows.is_empty() && self.highlighted_rows[0] < 5 {
                    if with_shift {
                        if !self.highlighted_rows.contains(&(self.highlighted_rows[0] + 1)) {
                            self.highlighted_rows.push(self.highlighted_rows[0] + 1);
                        }
                    } else {
                        self.highlighted_rows = vec![self.highlighted_rows[0] + 1];
                    }
                }
            },
            KeyCode::Left => {
                if !self.highlighted_cols.is_empty() && self.highlighted_cols[0] > 0 {
                    if with_shift {
                        if !self.highlighted_cols.contains(&(self.highlighted_cols[0] - 1)) {
                            self.highlighted_cols.push(self.highlighted_cols[0] - 1);
                        }
                    } else {
                        self.highlighted_cols = vec![self.highlighted_cols[0] - 1];
                    }
                }
            },
            KeyCode::Right => {
                if !self.highlighted_cols.is_empty() && self.highlighted_cols[0] < 2 {
                    if with_shift {
                        if !self.highlighted_cols.contains(&(self.highlighted_cols[0] + 1)) {
                            self.highlighted_cols.push(self.highlighted_cols[0] + 1);
                        }
                    } else {
                        self.highlighted_cols = vec![self.highlighted_cols[0] + 1];
                    }
                }
            },
            _ => {},
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
            Constraint::Length(8),
            Constraint::Min(10),
        ].as_ref())
        .split(f.area());
    
    // Title
    let title_text = match app.input_mode {
        InputMode::Navigation => "Interactive Form - Tab/Arrow keys to navigate, Enter to edit, Q/Esc to quit",
        InputMode::Editing => &format!("Editing {} - Type to input, Enter/Esc to stop editing", 
            match app.active_field {
                InputField::Name => "Name",
                InputField::Email => "Email", 
                InputField::Selection => "Selection",
            }),
    };
    
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);
    
    // Name input
    let name_style = if app.active_field == InputField::Name && app.input_mode == InputMode::Editing {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else if app.active_field == InputField::Name {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    
    let validation_style = if app.name_valid {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    };
    
    let name_block = Block::default()
        .title(Span::styled("Name", name_style))
        .borders(Borders::ALL)
        .border_style(if app.active_field == InputField::Name {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });
    
    let name_validation = if !app.name.is_empty() {
        if app.name_valid {
            " ✓"
        } else {
            " ✗ Name must be at least 2 characters"
        }
    } else {
        ""
    };
    
    let name_text = Paragraph::new(Text::from(vec![
        Line::from(vec! [
            Span::raw(&app.name),
            Span::styled(name_validation, validation_style),
        ]),
    ]))
    .block(name_block);
    f.render_widget(name_text, chunks[1]);
    
    // Email input
    let email_style = if app.active_field == InputField::Email && app.input_mode == InputMode::Editing {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else if app.active_field == InputField::Email {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    
    let validation_style = if app.email_valid {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    };
    
    let email_block = Block::default()
        .title(Span::styled("Email", email_style))
        .borders(Borders::ALL)
        .border_style(if app.active_field == InputField::Email {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });
    
    let email_validation = if !app.email.is_empty() {
        if app.email_valid {
            " ✓"
        } else {
            " ✗ Invalid email format (e.g. user@example.com)"
        }
    } else {
        ""
    };
    
    let email_text = Paragraph::new(Text::from(vec! [
        Line::from(vec! [
            Span::raw(&app.email),
            Span::styled(email_validation, validation_style),
        ]),
    ]))
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
        .title(Span::styled("Selection", selection_style))
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
    
    // History
    let history_items: Vec<ListItem> = app
        .history
        .iter()
        .map(|h| {
            let style = if h.contains("✗") {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(h, style)))
        })
        .collect();
    
    let history = List::new(history_items)
        .block(Block::default().title("History").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(history, chunks[4]);
    
    // Visualization section
    let viz_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[5]);
    
    // Bar Chart
    let data_values: Vec<(&str, u64)> = app.data
        .iter()
        .enumerate()
        .map(|(i, (name, val))| {
            let is_highlighted = app.highlighted_rows.contains(&i);
            (name.as_str(), if is_highlighted { *val + 10 } else { *val })
        })
        .collect();
    
    let barchart = BarChart::default()
        .block(Block::default().title("Bar Chart").borders(Borders::ALL))
        .data(&data_values)
        .bar_width(7)
        .bar_style(Style::default().fg(Color::Cyan))
        .value_style(Style::default().fg(Color::Black).bg(Color::Cyan));
    f.render_widget(barchart, viz_chunks[0]);
    
    // Histogram
    let barchart = BarChart::default()
        .block(Block::default().title("Histogram").borders(Borders::ALL))
        .data(&data_values)
        .bar_width(3)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::Yellow))
        .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));
    f.render_widget(barchart, viz_chunks[1]);
    
    // Cursor positioning - Fix the nested Position::new calls
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

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
) -> std::io::Result<()> {
    let mut app = App::default();
    
    loop {
        terminal.draw(|f| ui(f, &app))?;
        
        if let Event::Key(key) = event::read()? {
            // Only process key press events, not release events
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
                    KeyCode::BackTab => {
                        app.active_field = match app.active_field {
                            InputField::Name => InputField::Selection,
                            InputField::Email => InputField::Name,
                            InputField::Selection => InputField::Email,
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
                        } else if app.name_valid && app.email_valid {
                            let selection = match app.selected_option {
                                SelectOption::Yes => "Yes",
                                SelectOption::No => "No",
                                SelectOption::Maybe => "Maybe",
                                SelectOption::Other => "Other",
                            };
                            
                            let entry = format!(
                                "✓ Name: {}, Email: {}, Selection: {}",
                                app.name, app.email, selection
                            );
                            app.add_to_history(entry);
                            
                            app.name = String::new();
                            app.email = String::new();
                            app.name_valid = false;
                            app.email_valid = false;
                        } else {
                            app.add_to_history("✗ Please complete all fields correctly".to_string());
                        }
                    }
                    KeyCode::Left => {
                        if app.active_field == InputField::Selection {
                            app.selected_option = match app.selected_option {
                                SelectOption::Yes => SelectOption::Yes,
                                SelectOption::No => SelectOption::Yes,
                                SelectOption::Maybe => SelectOption::No,
                                SelectOption::Other => SelectOption::Maybe,
                            };
                        } else {
                            app.move_highlight(KeyCode::Left, key.modifiers.contains(KeyModifiers::SHIFT));
                        }
                    }
                    KeyCode::Right => {
                        if app.active_field == InputField::Selection {
                            app.selected_option = match app.selected_option {
                                SelectOption::Yes => SelectOption::No,
                                SelectOption::No => SelectOption::Maybe,
                                SelectOption::Maybe => SelectOption::Other,
                                SelectOption::Other => SelectOption::Other,
                            };
                        } else {
                            app.move_highlight(KeyCode::Right, key.modifiers.contains(KeyModifiers::SHIFT));
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
                                app.validate_name();
                            }
                            InputField::Email => {
                                app.email.push(c);
                                app.validate_email();
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Backspace => {
                        match app.active_field {
                            InputField::Name => {
                                if !app.name.is_empty() {
                                    app.name.pop();
                                    app.validate_name();
                                }
                            }
                            InputField::Email => {
                                if !app.email.is_empty() {
                                    app.email.pop();
                                    app.validate_email();
                                }
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
