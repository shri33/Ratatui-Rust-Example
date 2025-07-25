use std::collections::VecDeque;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Tabs,
        BarChart, Dataset, GraphType, Chart, Axis,
    },
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

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
    // Input fields
    name: String,
    email: String,
    
    // Current state
    input_mode: InputMode,
    active_field: InputField,
    selected_option: SelectOption,
    
    // History - now with fixed size
    history: VecDeque<String>,
    
    // Validation
    name_valid: bool,
    email_valid: bool,
    
    // For row/column highlighting
    highlighted_rows: Vec<usize>,
    highlighted_cols: Vec<usize>,
    
    // Data for visualization
    data: Vec<(String, u64)>,
}

impl Default for App {
    fn default() -> Self {
        // Sample data for the charts
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
        // Enhanced email validation - looks for @ and . with proper positioning
        let valid = !self.email.is_empty() &&
                   self.email.contains('@') && 
                   self.email.contains('.') && 
                   self.email.find('@') < self.email.rfind('.') &&
                   !self.email.starts_with('@') &&
                   !self.email.ends_with('.');
        self.email_valid = valid;
    }
    
    fn validate_name(&mut self) {
        // Simple name validation - non-empty and at least 2 chars
        self.name_valid = !self.name.trim().is_empty() && self.name.trim().len() >= 2;
    }
    
    fn add_to_history(&mut self, entry: String) {
        self.history.push_front(entry);
        // Keep history at fixed size to prevent memory growth
        while self.history.len() > HISTORY_SIZE {
            self.history.pop_back();
        }
    }
    
    fn move_highlight(&mut self, direction: KeyCode, with_shift: bool) {
        match direction {
            KeyCode::Up => {
                if !self.highlighted_rows.is_empty() && self.highlighted_rows[0] > 0 {
                    if with_shift {
                        // Add to selection
                        if !self.highlighted_rows.contains(&(self.highlighted_rows[0] - 1)) {
                            self.highlighted_rows.push(self.highlighted_rows[0] - 1);
                        }
                    } else {
                        // Single selection
                        self.highlighted_rows = vec![self.highlighted_rows[0] - 1];
                    }
                }
            },
            KeyCode::Down => {
                if !self.highlighted_rows.is_empty() && self.highlighted_rows[0] < 5 { // Assuming 6 data points (0-5)
                    if with_shift {
                        // Add to selection
                        if !self.highlighted_rows.contains(&(self.highlighted_rows[0] + 1)) {
                            self.highlighted_rows.push(self.highlighted_rows[0] + 1);
                        }
                    } else {
                        // Single selection
                        self.highlighted_rows = vec![self.highlighted_rows[0] + 1];
                    }
                }
            },
            KeyCode::Left => {
                if !self.highlighted_cols.is_empty() && self.highlighted_cols[0] > 0 {
                    if with_shift {
                        // Add to selection
                        if !self.highlighted_cols.contains(&(self.highlighted_cols[0] - 1)) {
                            self.highlighted_cols.push(self.highlighted_cols[0] - 1);
                        }
                    } else {
                        // Single selection
                        self.highlighted_cols = vec![self.highlighted_cols[0] - 1];
                    }
                }
            },
            KeyCode::Right => {
                if !self.highlighted_cols.is_empty() && self.highlighted_cols[0] < 2 { // Assuming 3 columns (0-2)
                    if with_shift {
                        // Add to selection
                        if !self.highlighted_cols.contains(&(self.highlighted_cols[0] + 1)) {
                            self.highlighted_cols.push(self.highlighted_cols[0] + 1);
                        }
                    } else {
                        // Single selection
                        self.highlighted_cols = vec![self.highlighted_cols[0] + 1];
                    }
                }
            },
            _ => {},
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Name input
            Constraint::Length(3),  // Email input
            Constraint::Length(3),  // Selection
            Constraint::Length(8),  // History
            Constraint::Min(10),    // Visualization
        ].as_ref())
        .split(f.area());
    
    // Title
    let title = Paragraph::new("Interactive Form Example")
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
    
    // Improved validation feedback with more distinct colors
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
        Line::from(vec![
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
    
    let email_text = Paragraph::new(Text::from(vec![
        Line::from(vec![
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
            .collect(),
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
    
    // Visualization section - split for bar chart and histogram
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
    
    // Histogram (simulated with bar chart for simplicity)
    let barchart = BarChart::default()
        .block(Block::default().title("Histogram").borders(Borders::ALL))
        .data(&data_values)
        .bar_width(3)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::Yellow))
        .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));
    f.render_widget(barchart, viz_chunks[1]);
    
    // Fixed cursor positioning - ensure cursor is at the right position in text fields
    if app.input_mode == InputMode::Editing {
        match app.active_field {
            InputField::Name => {
                // Position cursor at end of text
                f.set_cursor_position(ratatui::layout::Position::new(
                    chunks[1].x + app.name.width() as u16 + 1,
                    chunks[1].y + 1,
                ));
            }
            InputField::Email => {
                // Position cursor at end of text
                f.set_cursor_position(ratatui::layout::Position::new(
                    chunks[2].x + app.email.width() as u16 + 1,
                    chunks[2].y + 1,
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
        
        // Fixed event handling to ensure all key combinations are properly captured
        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Navigation => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Tab => {
                        // Enhanced form navigation - cycle through all elements
                        app.active_field = match app.active_field {
                            InputField::Name => InputField::Email,
                            InputField::Email => InputField::Selection,
                            InputField::Selection => InputField::Name,
                        };
                    }
                    KeyCode::BackTab => {
                        // Support Shift+Tab for reverse navigation
                        app.active_field = match app.active_field {
                            InputField::Name => InputField::Selection,
                            InputField::Email => InputField::Name,
                            InputField::Selection => InputField::Email,
                        };
                    }
                    KeyCode::Enter => {
                        if app.active_field != InputField::Selection {
                            app.input_mode = InputMode::Editing;
                        } else {
                            // When Selection is active and Enter is pressed
                            if app.name_valid && app.email_valid {
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
                            } else if app.name_valid && app.email_valid {
                                // When Selection is active and Enter is pressed
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
                            // For highlighting in data visualization
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
                            // For highlighting in data visualization
                            app.move_highlight(KeyCode::Right, key.modifiers.contains(KeyModifiers::SHIFT));
                        }
                    }
                    KeyCode::Up => {
                        app.move_highlight(KeyCode::Up, key.modifiers.contains(KeyModifiers::SHIFT));
                    }
                    KeyCode::Down => {
                        app.move_highlight(KeyCode::Down, key.modifiers.contains(KeyModifiers::SHIFT));
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
                                app.name.pop();
                                app.validate_name();
                            }
                            InputField::Email => {
                                app.email.pop();
                                app.validate_email();
                            }
                            _ => {}
                        }
                    }
                    // Support for Ctrl+Left/Right for word navigation
                    KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Word navigation left implementation would go here
                    }
                    KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Word navigation right implementation would go here
                    }
                    _ => {}
                },
            }
        }
    }
}
