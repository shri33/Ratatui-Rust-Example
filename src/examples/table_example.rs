//! Component showcase example
//!
//! Demonstrates various interactive components like dropdowns, radio buttons, and checkboxes.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Terminal, Frame,
};
use std::io;

// Component definitions
#[derive(Clone, Debug)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
    pub enabled: bool,
}

impl SelectOption {
    pub fn new(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            enabled: true,
        }
    }

    pub fn disabled(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            enabled: false,
        }
    }
}

pub enum SelectType {
    Dropdown,
    RadioGroup,
    CheckboxGroup,
}

pub struct SelectComponent {
    pub options: Vec<SelectOption>,
    pub selected_index: usize,
    pub selected_values: Vec<String>,
    pub is_open: bool,
    pub select_type: SelectType,
    pub title: String,
}

impl SelectComponent {
    pub fn new(title: &str, options: Vec<SelectOption>, select_type: SelectType) -> Self {
        Self {
            options,
            selected_index: 0,
            selected_values: Vec::new(),
            is_open: false,
            select_type,
            title: title.to_string(),
        }
    }

    pub fn next(&mut self) {
        if self.selected_index < self.options.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    pub fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.options.len() - 1;
        }
    }

    pub fn toggle_open(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn select_current(&mut self) {
        if self.selected_index < self.options.len() {
            let option = &self.options[self.selected_index];
            if !option.enabled {
                return;
            }

            match self.select_type {
                SelectType::Dropdown | SelectType::RadioGroup => {
                    self.selected_values.clear();
                    self.selected_values.push(option.value.clone());
                    self.is_open = false;
                }
                SelectType::CheckboxGroup => {
                    if let Some(pos) = self.selected_values.iter().position(|x| x == &option.value) {
                        self.selected_values.remove(pos);
                    } else {
                        self.selected_values.push(option.value.clone());
                    }
                }
            }
        }
    }

    pub fn get_selected_labels(&self) -> Vec<String> {
        self.selected_values
            .iter()
            .filter_map(|value| {
                self.options
                    .iter()
                    .find(|opt| &opt.value == value)
                    .map(|opt| opt.label.clone())
            })
            .collect()
    }

    pub fn render(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let selected_text = if self.selected_values.is_empty() {
            "Select an option...".to_string()
        } else {
            self.get_selected_labels().join(", ")
        };

        let display_text = format!("{}: {}", self.title, selected_text);

        let paragraph = Paragraph::new(display_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default());

        frame.render_widget(paragraph, area);
    }
}

// Main App struct - ONLY ONE!
struct App {
    dropdown: SelectComponent,
    radio_group: SelectComponent,
    checkbox_group: SelectComponent,
    active_component: usize,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let dropdown_options = vec![
            SelectOption::new("Option 1", "opt1"),
            SelectOption::new("Option 2", "opt2"),
            SelectOption::new("Option 3", "opt3"),
            SelectOption::disabled("Disabled Option", "disabled"),
        ];

        let radio_options = vec![
            SelectOption::new("Yes", "yes"),
            SelectOption::new("No", "no"),
            SelectOption::new("Maybe", "maybe"),
            SelectOption::new("Other", "other"),
        ];

        let checkbox_options = vec![
            SelectOption::new("Feature A", "feature_a"),
            SelectOption::new("Feature B", "feature_b"),
            SelectOption::new("Feature C", "feature_c"),
            SelectOption::new("Feature D", "feature_d"),
        ];

        Self {
            dropdown: SelectComponent::new("Dropdown Menu", dropdown_options, SelectType::Dropdown),
            radio_group: SelectComponent::new("Radio Selection", radio_options, SelectType::RadioGroup),
            checkbox_group: SelectComponent::new("Checkbox Group", checkbox_options, SelectType::CheckboxGroup),
            active_component: 0,
            should_quit: false,
        }
    }

    fn next_component(&mut self) {
        self.active_component = (self.active_component + 1) % 3;
    }

    fn previous_component(&mut self) {
        self.active_component = if self.active_component == 0 { 2 } else { self.active_component - 1 };
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => self.next_component(),
            KeyCode::BackTab => self.previous_component(),
            KeyCode::Up => {
                match self.active_component {
                    0 => self.dropdown.previous(),
                    1 => self.radio_group.previous(),
                    2 => self.checkbox_group.previous(),
                    _ => {}
                }
            }
            KeyCode::Down => {
                match self.active_component {
                    0 => self.dropdown.next(),
                    1 => self.radio_group.next(),
                    2 => self.checkbox_group.next(),
                    _ => {}
                }
            }
            KeyCode::Enter => {
                match self.active_component {
                    0 => {
                        if self.dropdown.is_open {
                            self.dropdown.select_current();
                        } else {
                            self.dropdown.toggle_open();
                        }
                    }
                    1 => self.radio_group.select_current(),
                    2 => self.checkbox_group.select_current(),
                    _ => {}
                }
            }
            KeyCode::Esc => {
                if self.dropdown.is_open {
                    self.dropdown.is_open = false;
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let size = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(size);

        // Instructions
        let instructions = Paragraph::new(vec![
            Line::from("Tab: Next Component | ↑↓: Navigate | Enter: Select | Esc: Close | q: Quit"),
        ])
        .block(Block::default().borders(Borders::ALL).title("Select Component Demo"));
        frame.render_widget(instructions, chunks[0]);

        // Dropdown
        self.dropdown.render(frame, chunks[1]);

        // Radio Group  
        self.radio_group.render(frame, chunks[2]);

        // Checkbox Group
        self.checkbox_group.render(frame, chunks[3]);

        // Results
        let results_text = vec![
            Line::from(format!("Dropdown: {:?}", self.dropdown.get_selected_labels())),
            Line::from(format!("Radio: {:?}", self.radio_group.get_selected_labels())),
            Line::from(format!("Checkbox: {:?}", self.checkbox_group.get_selected_labels())),
        ];
        
        let results = Paragraph::new(results_text)
            .block(Block::default().borders(Borders::ALL).title("Selected Values"));
        frame.render_widget(results, chunks[4]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| app.render(f))?;

        if let Event::Key(key) = event::read()? {
            app.on_key(key.code);
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    Ok(())
}
