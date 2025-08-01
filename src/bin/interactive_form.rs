use std::io;
use std::collections::VecDeque;
use std::fs;
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
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
        BarChart, Table, Row, Cell, Clear, Gauge,
    },
    Frame, Terminal,
};

const HISTORY_SIZE: usize = 10;

#[derive(PartialEq, Copy, Clone)]
enum InputMode {
    Navigation,
    Editing,
    Generating, // For loading states
}

#[derive(PartialEq, Copy, Clone)]
enum InputField {
    Name,
    Email,
    Selection,
    Table,
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
    table_data: Vec<Vec<String>>,
    table_headers: Vec<String>,
    selected_table_row: usize,
    selected_table_col: usize,
    loading_progress: u16,
    show_loading: bool,
    campaign_generated: bool,
    show_image_placeholder: bool,
    last_generation_time: Option<Instant>,
    // Command history for shell-like experience
    command_history: VecDeque<String>,
    history_index: Option<usize>,
    current_input_backup: String,
}

impl Default for App {
    fn default() -> Self {
        let mut history = VecDeque::new();
        history.push_back("Welcome to Vue.js-style TUI".to_string());
        history.push_back("Enter your information below".to_string());
        
        let data = vec![
            ("Campaign A".to_string(), 95),
            ("Campaign B".to_string(), 67),
            ("Campaign C".to_string(), 78),
            ("Campaign D".to_string(), 89),
            ("Campaign E".to_string(), 45),
            ("Campaign F".to_string(), 93),
        ];

        let table_data = vec![
            vec!["John".to_string(), "john@example.com".to_string(), "Yes".to_string(), "Premium".to_string()],
            vec!["Alice".to_string(), "alice@example.com".to_string(), "No".to_string(), "Basic".to_string()],
            vec!["Bob".to_string(), "bob@example.com".to_string(), "Maybe".to_string(), "Premium".to_string()],
            vec!["Sarah".to_string(), "sarah@example.com".to_string(), "Yes".to_string(), "Basic".to_string()],
            vec!["Mike".to_string(), "mike@example.com".to_string(), "Other".to_string(), "Premium".to_string()],
        ];

        let table_headers = vec![
            "Name".to_string(),
            "Email".to_string(),
            "Subscribe".to_string(),
            "Plan".to_string(),
        ];

        Self {
            name: String::new(),
            email: String::new(),
            input_mode: InputMode::Navigation,
            active_field: InputField::Name,
            selected_option: SelectOption::Yes,
            history,
            name_valid: false,
            email_valid: false,
            highlighted_rows: vec![0],
            highlighted_cols: vec![0],
            data,
            table_data,
            table_headers,
            selected_table_row: 0,
            selected_table_col: 0,
            loading_progress: 0,
            show_loading: false,
            campaign_generated: false,
            show_image_placeholder: true,
            last_generation_time: None,
            command_history: VecDeque::new(),
            history_index: None,
            current_input_backup: String::new(),
        }
    }
}

impl App {
    fn validate_email(&mut self) {
        // Enhanced email validation with detailed feedback
        if self.email.is_empty() {
            self.email_valid = false;
            return;
        }
        
        let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        let basic_valid = email_regex.is_match(&self.email);
        
        // Additional checks for common issues
        let has_at = self.email.contains('@');
        let has_dot = self.email.contains('.');
        let starts_with_at = self.email.starts_with('@');
        let ends_with_dot = self.email.ends_with('.');
        let multiple_at = self.email.matches('@').count() > 1;
        
        self.email_valid = basic_valid && has_at && has_dot && !starts_with_at && !ends_with_dot && !multiple_at;
        
        // Add validation feedback to history
        if !self.email.is_empty() && !self.email_valid {
            let reason = if !has_at {
                "Missing @ symbol"
            } else if !has_dot {
                "Missing domain (.com, .org, etc.)"
            } else if starts_with_at {
                "Cannot start with @"
            } else if ends_with_dot {
                "Cannot end with ."
            } else if multiple_at {
                "Too many @ symbols"
            } else {
                "Invalid email format"
            };
            
            // Only add feedback once per validation issue
            let feedback_msg = format!("✗ Email: {}", reason);
            if !self.history.front().map_or(false, |h| h.contains(&feedback_msg)) {
                self.add_to_history(feedback_msg);
            }
        }
    }
    
    fn validate_name(&mut self) {
        let trimmed = self.name.trim();
        let was_valid = self.name_valid;
        self.name_valid = !trimmed.is_empty() && trimmed.len() >= 2;
        
        // Add validation feedback to history
        if !self.name.is_empty() && !self.name_valid && was_valid != self.name_valid {
            self.add_to_history("✗ Name must be at least 2 characters".to_string());
        } else if self.name_valid && was_valid != self.name_valid {
            self.add_to_history("✓ Name is valid".to_string());
        }
    }
    
    fn add_to_command_history(&mut self, command: String) {
        if !command.trim().is_empty() && !self.command_history.contains(&command) {
            self.command_history.push_front(command);
            if self.command_history.len() > 20 {
                self.command_history.pop_back();
            }
        }
        self.history_index = None;
        self.current_input_backup.clear();
    }
    
    fn scroll_command_history(&mut self, direction: i32) -> bool {
        if self.command_history.is_empty() {
            return false;
        }
        
        // Backup current input on first scroll
        if self.history_index.is_none() {
            self.current_input_backup = match self.active_field {
                InputField::Name => self.name.clone(),
                InputField::Email => self.email.clone(),
                _ => String::new(),
            };
        }
        
        let new_index = match self.history_index {
            None => {
                if direction > 0 {
                    Some(0)
                } else {
                    None
                }
            }
            Some(current) => {
                if direction > 0 && current + 1 < self.command_history.len() {
                    Some(current + 1)
                } else if direction < 0 && current > 0 {
                    Some(current - 1)
                } else if direction < 0 && current == 0 {
                    None
                } else {
                    Some(current)
                }
            }
        };
        
        if new_index != self.history_index {
            self.history_index = new_index;
            
            let text = if let Some(idx) = new_index {
                self.command_history[idx].clone()
            } else {
                self.current_input_backup.clone()
            };
            
            match self.active_field {
                InputField::Name => {
                    self.name = text;
                    self.validate_name();
                }
                InputField::Email => {
                    self.email = text;
                    self.validate_email();
                }
                _ => {}
            }
            
            return true;
        }
        
        false
    }
    
    fn add_to_history(&mut self, entry: String) {
        self.history.push_front(entry);
        while self.history.len() > HISTORY_SIZE {
            self.history.pop_back();
        }
    }
    
    fn move_table_selection(&mut self, direction: KeyCode, with_shift: bool) {
        let old_row = self.selected_table_row;
        let old_col = self.selected_table_col;
        
        match direction {
            KeyCode::Up => {
                if self.selected_table_row > 0 {
                    self.selected_table_row -= 1;
                }
            },
            KeyCode::Down => {
                if self.selected_table_row < self.table_data.len() - 1 {
                    self.selected_table_row += 1;
                }
            },
            KeyCode::Left => {
                if self.selected_table_col > 0 {
                    self.selected_table_col -= 1;
                }
            },
            KeyCode::Right => {
                if self.selected_table_col < self.table_headers.len() - 1 {
                    self.selected_table_col += 1;
                }
            },
            _ => return,
        }
        
        // Handle selection highlighting
        if with_shift {
            // Multi-selection mode
            match direction {
                KeyCode::Up | KeyCode::Down => {
                    // Add range of rows to selection
                    let start = old_row.min(self.selected_table_row);
                    let end = old_row.max(self.selected_table_row);
                    for row in start..=end {
                        if !self.highlighted_rows.contains(&row) {
                            self.highlighted_rows.push(row);
                        }
                    }
                    self.add_to_history(format!("Selected rows {}-{}", start, end));
                },
                KeyCode::Left | KeyCode::Right => {
                    // Add range of columns to selection
                    let start = old_col.min(self.selected_table_col);
                    let end = old_col.max(self.selected_table_col);
                    for col in start..=end {
                        if !self.highlighted_cols.contains(&col) {
                            self.highlighted_cols.push(col);
                        }
                    }
                    self.add_to_history(format!("Selected columns {}-{}", start, end));
                },
                _ => {}
            }
        } else {
            // Single selection mode
            self.highlighted_rows = vec![self.selected_table_row];
            self.highlighted_cols = vec![self.selected_table_col];
        }
    }

    fn start_campaign_generation(&mut self) {
        self.input_mode = InputMode::Generating;
        self.show_loading = true;
        self.loading_progress = 0;
        self.campaign_generated = false;
        self.last_generation_time = Some(Instant::now());
    }

    fn update_loading(&mut self) {
        if let Some(start_time) = self.last_generation_time {
            let elapsed = start_time.elapsed();
            let progress = (elapsed.as_millis() as f32 / 3000.0 * 100.0) as u16;
            
            if progress >= 100 {
                self.loading_progress = 100;
                self.show_loading = false;
                self.campaign_generated = true;
                self.input_mode = InputMode::Navigation;
                self.create_campaign_files();
                self.add_to_history("✓ Campaign files generated successfully!".to_string());
            } else {
                self.loading_progress = progress;
            }
        }
    }

    fn create_campaign_files(&self) {
        // Create campaign directory
        let campaign_dir = format!("campaign_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        let _ = fs::create_dir_all(&campaign_dir);

        // Create campaign.json
        let campaign_data = serde_json::json!({
            "name": self.name,
            "email": self.email,
            "selection": match self.selected_option {
                SelectOption::Yes => "Yes",
                SelectOption::No => "No", 
                SelectOption::Maybe => "Maybe",
                SelectOption::Other => "Other",
            },
            "created_at": chrono::Utc::now().to_rfc3339(),
            "data": self.data
        });

        let campaign_file = format!("{}/campaign.json", campaign_dir);
        let _ = fs::write(campaign_file, serde_json::to_string_pretty(&campaign_data).unwrap());

        // Create README.md
        let readme_content = format!(
            "# Campaign: {}\n\n- Email: {}\n- Selection: {}\n- Generated: {}\n",
            self.name,
            self.email,
            match self.selected_option {
                SelectOption::Yes => "Yes",
                SelectOption::No => "No",
                SelectOption::Maybe => "Maybe", 
                SelectOption::Other => "Other",
            },
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        let readme_file = format!("{}/README.md", campaign_dir);
        let _ = fs::write(readme_file, readme_content);
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Name
            Constraint::Length(3),  // Email
            Constraint::Length(3),  // Selection
            Constraint::Length(3),  // Actions
            Constraint::Length(6),  // History
            Constraint::Length(8),  // Table
            Constraint::Min(6),     // Charts & Image
        ].as_ref())
        .split(f.area());
    
    // Title with enhanced instructions
    let title_text = match app.input_mode {
        InputMode::Navigation => {
            let mut instructions = vec!["Interactive Form - "];
            instructions.push("Tab/↑↓ navigate, Enter edit, Space generate");
            if app.active_field == InputField::Table {
                instructions.push(", Shift+Arrow multi-select, Ctrl+A select all, Ctrl+C clear");
            }
            instructions.push(", G toggle image, Q quit");
            instructions.join("")
        },
        InputMode::Editing => {
            let field_name = match app.active_field {
                InputField::Name => "Name",
                InputField::Email => "Email", 
                InputField::Selection => "Selection",
                InputField::Table => "Table",
            };
            let mut instructions = format!("Editing {} - Type to input, Enter/Esc to stop", field_name);
            if app.command_history.len() > 0 {
                instructions.push_str(", ↑↓ for history");
            }
            instructions
        },
        InputMode::Generating => "Generating campaign files... Press Q/Esc to quit".to_string(),
    };
    
    let title = Paragraph::new(title_text.as_str())
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
            " ✓ Valid email"
        } else {
            // Detailed validation feedback
            if !app.email.contains('@') {
                " ✗ Missing @ symbol"
            } else if !app.email.contains('.') {
                " ✗ Missing domain (.com, .org, etc.)"
            } else if app.email.starts_with('@') {
                " ✗ Cannot start with @"
            } else if app.email.ends_with('.') {
                " ✗ Cannot end with ."
            } else if app.email.matches('@').count() > 1 {
                " ✗ Too many @ symbols"
            } else {
                " ✗ Invalid format (e.g. user@example.com)"
            }
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

    // Actions section with enhanced feedback
    let actions_text = if app.name_valid && app.email_valid {
        format!("✓ Ready! Space=generate files | G=toggle image | Ctrl+A=select all rows | Ctrl+C=clear selection")
    } else {
        let mut issues = Vec::new();
        if !app.name_valid {
            issues.push("name");
        }
        if !app.email_valid {
            issues.push("email");
        }
        format!("⚠ Complete {} to enable generation | G=toggle image", issues.join(" & "))
    };
    
    let actions = Paragraph::new(actions_text)
        .style(if app.name_valid && app.email_valid {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        })
        .block(Block::default().title("Actions & Status").borders(Borders::ALL));
    f.render_widget(actions, chunks[4]);

    // Loading indicator
    if app.show_loading {
        let loading_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.area())[1];
        
        let gauge = Gauge::default()
            .block(Block::default().title("Generating Campaign").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(app.loading_progress)
            .label(format!("{}%", app.loading_progress));
        
        f.render_widget(Clear, loading_area);
        f.render_widget(gauge, loading_area);
    }
    
    // History
    let history_items: Vec<ListItem> = app
        .history
        .iter()
        .map(|h| {
            let style = if h.contains("✗") {
                Style::default().fg(Color::Red)
            } else if h.contains("✓") {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(h, style)))
        })
        .collect();
    
    let history = List::new(history_items)
        .block(Block::default().title("History").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(history, chunks[5]);

    // Table rendering with row/column highlighting
    let table_style = if app.active_field == InputField::Table {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let header_cells = app.table_headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let style = if app.highlighted_cols.contains(&i) {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Cell::from(h.as_str()).style(style)
        })
        .collect::<Vec<_>>();
    
    let header = Row::new(header_cells)
        .style(Style::default().fg(Color::White))
        .height(1)
        .bottom_margin(1);

    let rows = app.table_data
        .iter()
        .enumerate()
        .map(|(row_idx, item)| {
            let cells = item
                .iter()
                .enumerate()
                .map(|(col_idx, cell)| {
                    let mut style = Style::default();
                    if app.highlighted_rows.contains(&row_idx) {
                        style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
                    }
                    if app.highlighted_cols.contains(&col_idx) {
                        style = style.bg(Color::DarkGray);
                    }
                    if row_idx == app.selected_table_row && col_idx == app.selected_table_col {
                        style = style.bg(Color::Blue);
                    }
                    Cell::from(cell.as_str()).style(style)
                })
                .collect::<Vec<_>>();
            Row::new(cells).height(1)
        });

    let table = Table::new(rows, [
        Constraint::Percentage(25),
        Constraint::Percentage(35), 
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ])
    .header(header)
    .block(Block::default()
        .title(Span::styled("Table (Arrow keys + Shift for multi-select)", table_style))
        .borders(Borders::ALL)
        .border_style(if app.active_field == InputField::Table {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        }))
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol(">> ");
    
    f.render_widget(table, chunks[6]);
    
    // Charts and Image Display section
    let viz_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33), 
            Constraint::Percentage(33), 
            Constraint::Percentage(34)
        ].as_ref())
        .split(chunks[7]);
    
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
    let histogram = BarChart::default()
        .block(Block::default().title("Histogram").borders(Borders::ALL))
        .data(&data_values)
        .bar_width(3)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::Yellow))
        .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));
    f.render_widget(histogram, viz_chunks[1]);

    // High-res Image Display Simulation
    let image_content = if app.show_image_placeholder {
        vec![
            Line::from("┌─────────────────┐"),
            Line::from("│    📸 IMAGE     │"),
            Line::from("│  [1920x1080]    │"),
            Line::from("│ sample.jpg      │"),
            Line::from("│                 │"),
            Line::from("└─────────────────┘"),
        ]
    } else {
        vec![
            Line::from("Image display"),
            Line::from("disabled"),
            Line::from(""),
            Line::from("Press G to"),
            Line::from("toggle"),
        ]
    };

    let image_display = Paragraph::new(image_content)
        .style(Style::default().fg(Color::Magenta))
        .block(Block::default()
            .title("High-res Image Display")
            .borders(Borders::ALL));
    f.render_widget(image_display, viz_chunks[2]);
    
    // Cursor positioning for editing
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
        // Update loading progress if generating
        if app.show_loading {
            app.update_loading();
        }
        
        terminal.draw(|f| ui(f, &app))?;
        
        // Handle events with timeout for loading updates
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // FIXED: Removed KeyEventKind filtering that was blocking input
                match app.input_mode {
                    InputMode::Navigation => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Tab => {
                            app.active_field = match app.active_field {
                                InputField::Name => InputField::Email,
                                InputField::Email => InputField::Selection,
                                InputField::Selection => InputField::Table,
                                InputField::Table => InputField::Name,
                            };
                        }
                        KeyCode::BackTab => {
                            app.active_field = match app.active_field {
                                InputField::Name => InputField::Table,
                                InputField::Email => InputField::Name,
                                InputField::Selection => InputField::Email,
                                InputField::Table => InputField::Selection,
                            };
                        }
                        KeyCode::Up => {
                            if app.active_field == InputField::Table {
                                app.move_table_selection(KeyCode::Up, key.modifiers.contains(KeyModifiers::SHIFT));
                            } else {
                                app.active_field = match app.active_field {
                                    InputField::Name => InputField::Table,
                                    InputField::Email => InputField::Name,
                                    InputField::Selection => InputField::Email,
                                    InputField::Table => InputField::Selection,
                                };
                            }
                        }
                        KeyCode::Down => {
                            if app.active_field == InputField::Table {
                                app.move_table_selection(KeyCode::Down, key.modifiers.contains(KeyModifiers::SHIFT));
                            } else {
                                app.active_field = match app.active_field {
                                    InputField::Name => InputField::Email,
                                    InputField::Email => InputField::Selection,
                                    InputField::Selection => InputField::Table,
                                    InputField::Table => InputField::Name,
                                };
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
                            } else if app.active_field == InputField::Table {
                                app.move_table_selection(KeyCode::Left, key.modifiers.contains(KeyModifiers::SHIFT));
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
                            } else if app.active_field == InputField::Table {
                                app.move_table_selection(KeyCode::Right, key.modifiers.contains(KeyModifiers::SHIFT));
                            }
                        }
                        KeyCode::Enter => {
                            if app.active_field == InputField::Name || app.active_field == InputField::Email {
                                app.input_mode = InputMode::Editing;
                            } else if app.active_field == InputField::Selection && app.name_valid && app.email_valid {
                                let selection = match app.selected_option {
                                    SelectOption::Yes => "Yes",
                                    SelectOption::No => "No",
                                    SelectOption::Maybe => "Maybe",
                                    SelectOption::Other => "Other",
                                };
                                
                                let entry = format!(
                                    "✓ Form submitted - Name: {}, Email: {}, Selection: {}",
                                    app.name, app.email, selection
                                );
                                app.add_to_history(entry);
                            } else if app.active_field == InputField::Selection {
                                app.add_to_history("✗ Please complete all fields correctly".to_string());
                            }
                        }
                        KeyCode::Char(' ') => {
                            if app.name_valid && app.email_valid && !app.show_loading {
                                app.start_campaign_generation();
                            }
                        }
                        KeyCode::Char('c') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                // Clear multi-selections
                                app.highlighted_rows = vec![app.selected_table_row];
                                app.highlighted_cols = vec![app.selected_table_col];
                                app.add_to_history("Cleared multi-selection".to_string());
                            }
                        }
                        KeyCode::Char('a') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) && app.active_field == InputField::Table {
                                // Select all rows
                                app.highlighted_rows = (0..app.table_data.len()).collect();
                                app.add_to_history("Selected all rows".to_string());
                            }
                        }
                        KeyCode::Char('g') | KeyCode::Char('G') => {
                            app.show_image_placeholder = !app.show_image_placeholder;
                            app.add_to_history(format!("Image display {}", 
                                if app.show_image_placeholder { "enabled" } else { "disabled" }));
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter | KeyCode::Esc => {
                            // Save current input to command history
                            let current_text = match app.active_field {
                                InputField::Name => app.name.clone(),
                                InputField::Email => app.email.clone(),
                                _ => String::new(),
                            };
                            if !current_text.is_empty() {
                                app.add_to_command_history(current_text);
                            }
                            app.input_mode = InputMode::Navigation;
                        }
                        KeyCode::Up => {
                            if app.scroll_command_history(1) {
                                app.add_to_history("↑ Previous command".to_string());
                            }
                        }
                        KeyCode::Down => {
                            if app.scroll_command_history(-1) {
                                app.add_to_history("↓ Next command".to_string());
                            }
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
                    InputMode::Generating => {
                        // Block input during generation except for quit
                        if let KeyCode::Char('q') | KeyCode::Esc = key.code {
                            return Ok(());
                        }
                    }
                }
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
