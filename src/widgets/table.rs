//! Table widget module
//! 
//! Enhanced table widget with dynamic data and interactive features.

use std::collections::HashSet;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState, Clear},
    Frame,
};
use crossterm::event::KeyCode;

#[derive(Clone, Debug)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub struct InteractiveTable {
    pub data: TableData,
    pub state: TableState,
    pub selected_column: usize,
    pub multi_selection: HashSet<(usize, usize)>, // (row, col) pairs
    pub column_widths: Vec<Constraint>,
}

impl InteractiveTable {
    pub fn new(data: TableData) -> Self {
        let column_count = data.headers.len();
        let column_widths = vec![Constraint::Percentage(100 / column_count as u16); column_count];
        
        Self {
            data,
            state: TableState::default(),
            selected_column: 0,
            multi_selection: HashSet::new(),
            column_widths,
        }
    }

    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.data.rows.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.data.rows.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn next_column(&mut self) {
        if self.selected_column >= self.data.headers.len() - 1 {
            self.selected_column = 0;
        } else {
            self.selected_column += 1;
        }
    }

    pub fn previous_column(&mut self) {
        if self.selected_column == 0 {
            self.selected_column = self.data.headers.len() - 1;
        } else {
            self.selected_column -= 1;
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let selected_style = Style::default().bg(Color::Yellow).fg(Color::Black);
        let normal_style = Style::default().bg(Color::Reset).fg(Color::White);
        let header_style = Style::default().bg(Color::Blue).fg(Color::White);

        // Create header cells with column highlighting
        let header_cells: Vec<Cell> = self.data.headers
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let style = if i == self.selected_column {
                    header_style.bg(Color::Cyan)
                } else {
                    header_style
                };
                Cell::from(h.as_str()).style(style)
            })
            .collect();

        // Create data rows with cell highlighting
        let rows: Vec<Row> = self.data.rows
            .iter()
            .enumerate()
            .map(|(row_idx, item)| {
                let cells: Vec<Cell> = item
                    .iter()
                    .enumerate()
                    .map(|(col_idx, c)| {
                        let style = if Some(row_idx) == self.state.selected() && col_idx == self.selected_column {
                            selected_style
                        } else if Some(row_idx) == self.state.selected() {
                            Style::default().bg(Color::DarkGray)
                        } else if col_idx == self.selected_column {
                            Style::default().bg(Color::Gray)
                        } else if self.multi_selection.contains(&(row_idx, col_idx)) {
                            Style::default().bg(Color::Magenta).fg(Color::White)
                        } else {
                            normal_style
                        };
                        Cell::from(c.as_str()).style(style)
                    })
                    .collect();
                Row::new(cells)
            })
            .collect();

        let table = Table::new(rows, &[Constraint::Percentage(25); 4])
            .header(Row::new(header_cells))
            .block(Block::default().borders(Borders::ALL).title("Interactive Table"))
            .highlight_style(selected_style);

        f.render_stateful_widget(table, area, &mut self.state);
    }

    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Up => {
                self.previous_row();
                true
            }
            KeyCode::Down => {
                self.next_row();
                true
            }
            KeyCode::Left => {
                self.previous_column();
                true
            }
            KeyCode::Right => {
                self.next_column();
                true
            }
            KeyCode::Char('c') => {
                // Clear multi-selection
                self.multi_selection.clear();
                true
            }
            _ => false,
        }
    }

    pub fn handle_key_with_shift(&mut self, key: KeyCode, shift_pressed: bool) -> bool {
        if shift_pressed {
            // Handle multi-selection with Shift
            if let Some(current_row) = self.state.selected() {
                match key {
                    KeyCode::Up => {
                        self.multi_selection.insert((current_row, self.selected_column));
                        self.previous_row();
                        return true;
                    }
                    KeyCode::Down => {
                        self.multi_selection.insert((current_row, self.selected_column));
                        self.next_row();
                        return true;
                    }
                    KeyCode::Left => {
                        self.multi_selection.insert((current_row, self.selected_column));
                        self.previous_column();
                        return true;
                    }
                    KeyCode::Right => {
                        self.multi_selection.insert((current_row, self.selected_column));
                        self.next_column();
                        return true;
                    }
                    _ => {}
                }
            }
        }
        self.handle_key(key)
    }

    pub fn get_selection_count(&self) -> usize {
        self.multi_selection.len()
    }

    pub fn get_selected_cells(&self) -> Vec<(usize, usize)> {
        self.multi_selection.iter().cloned().collect()
    }
}

/// Create a demo table with sample data
pub fn create_demo_table() -> InteractiveTable {
    let data = TableData {
        headers: vec![
            "Name".to_string(),
            "Age".to_string(),
            "City".to_string(),
            "Status".to_string(),
        ],
        rows: vec![
            vec!["Alice".to_string(), "25".to_string(), "New York".to_string(), "Active".to_string()],
            vec!["Bob".to_string(), "30".to_string(), "London".to_string(), "Active".to_string()],
            vec!["Charlie".to_string(), "35".to_string(), "Tokyo".to_string(), "Inactive".to_string()],
            vec!["Diana".to_string(), "28".to_string(), "Paris".to_string(), "Active".to_string()],
            vec!["Eve".to_string(), "32".to_string(), "Berlin".to_string(), "Active".to_string()],
            vec!["Frank".to_string(), "29".to_string(), "Sydney".to_string(), "Inactive".to_string()],
        ],
    };
    
    InteractiveTable::new(data)
}

// Select component code follows...
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

#[derive(Debug, Clone)]
pub enum SelectType {
    Dropdown,
    RadioGroup,
    CheckboxGroup,
}

pub struct SelectComponent {
    pub options: Vec<SelectOption>,
    pub selected_index: usize,
    pub selected_values: Vec<String>, // For multi-select
    pub is_open: bool,
    pub select_type: SelectType,
    pub title: String,
    state: ListState,
}

impl SelectComponent {
    pub fn new(title: &str, options: Vec<SelectOption>, select_type: SelectType) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        
        Self {
            options,
            selected_index: 0,
            selected_values: Vec::new(),
            is_open: false,
            select_type,
            title: title.to_string(),
            state,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.options.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_index = i;
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.options.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_index = i;
        self.state.select(Some(i));
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

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL);

        match self.select_type {
            SelectType::Dropdown => self.render_dropdown(frame, area, block),
            SelectType::RadioGroup => self.render_radio_group(frame, area, block),
            SelectType::CheckboxGroup => self.render_checkbox_group(frame, area, block),
        }
    }

    fn render_dropdown(&mut self, frame: &mut Frame, area: Rect, block: Block) {
        let selected_text = if self.selected_values.is_empty() {
            "Select an option...".to_string()
        } else {
            self.get_selected_labels().join(", ")
        };

        let indicator = if self.is_open { "▲" } else { "▼" };
        let display_text = format!("{} {}", selected_text, indicator);

        let paragraph = Paragraph::new(display_text)
            .block(block)
            .style(Style::default());

        frame.render_widget(paragraph, area);

        if self.is_open {
            self.render_options_popup(frame, area);
        }
    }

    fn render_radio_group(&mut self, frame: &mut Frame, area: Rect, block: Block) {
        let items: Vec<ListItem> = self.options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let symbol = if self.selected_values.contains(&option.value) {
                    "●"
                } else {
                    "○"
                };
                
                let style = if option.enabled {
                    if i == self.selected_index {
                        Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    }
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                ListItem::new(format!("{} {}", symbol, option.label)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD));

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn render_checkbox_group(&mut self, frame: &mut Frame, area: Rect, block: Block) {
        let items: Vec<ListItem> = self.options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let symbol = if self.selected_values.contains(&option.value) {
                    "☑"
                } else {
                    "☐"
                };
                
                let style = if option.enabled {
                    if i == self.selected_index {
                        Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    }
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                ListItem::new(format!("{} {}", symbol, option.label)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD));

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn render_options_popup(&mut self, frame: &mut Frame, area: Rect) {
        let popup_area = self.centered_rect(80, 60, area);
        
        frame.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = self.options
            .iter()
            .map(|option| {
                let style = if option.enabled {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                ListItem::new(option.label.clone()).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Select Option"))
            .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD));

        frame.render_stateful_widget(list, popup_area, &mut self.state);
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_creation() {
        let table = create_demo_table();
        assert_eq!(table.data.rows.len(), 6);
        assert_eq!(table.data.headers.len(), 4);
    }

    #[test]
    fn test_navigation() {
        let mut table = create_demo_table();
        
        // Initially no selection
        assert_eq!(table.state.selected(), None);
        
        // Move to first item
        table.next_row();
        assert_eq!(table.state.selected(), Some(0));
        
        // Move to second item
        table.next_row();
        assert_eq!(table.state.selected(), Some(1));
        
        // Move back to first
        table.previous_row();
        assert_eq!(table.state.selected(), Some(0));
    }

    #[test]
    fn test_multi_selection() {
        let mut table = create_demo_table();
        table.state.select(Some(0));
        
        // Add some selections
        table.multi_selection.insert((0, 0));
        table.multi_selection.insert((1, 1));
        
        assert_eq!(table.get_selection_count(), 2);
        
        // Clear selections
        table.multi_selection.clear();
        assert_eq!(table.get_selection_count(), 0);
    }
}
