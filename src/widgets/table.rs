//! Table widget module
//! 
//! Enhanced table widget with dynamic data and interactive features.

use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};

/// Enhanced table widget with state management
pub struct EnhancedTable {
    /// Table state for selection and scrolling
    pub state: TableState,
    /// Table data
    pub data: Vec<Vec<String>>,
    /// Column headers
    pub headers: Vec<String>,
    /// Column widths
    pub widths: Vec<Constraint>,
}

impl Default for EnhancedTable {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedTable {
    /// Create a new enhanced table
    pub fn new() -> Self {
        Self {
            state: TableState::default(),
            data: Vec::new(),
            headers: Vec::new(),
            widths: Vec::new(),
        }
    }

    /// Set table data
    pub fn set_data(&mut self, data: Vec<Vec<String>>) {
        self.data = data;
        // Reset selection when data changes
        self.state.select(None);
    }

    /// Set table headers
    pub fn set_headers(&mut self, headers: Vec<String>) {
        self.headers = headers;
    }

    /// Set column widths
    pub fn set_widths(&mut self, widths: Vec<Constraint>) {
        self.widths = widths;
    }

    /// Move selection up
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.data.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Move selection down
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.data.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Get the currently selected row data
    pub fn get_selected(&self) -> Option<&Vec<String>> {
        self.state.selected().and_then(|i| self.data.get(i))
    }

    /// Render the table
    pub fn render(&mut self, frame: &mut Frame, area: Rect, title: &str) {
        let rows: Vec<Row> = self.data
            .iter()
            .map(|row| Row::new(row.iter().map(|cell| cell.as_str()).collect::<Vec<_>>()))
            .collect();

        let table = Table::new(rows, &self.widths)
            .header(
                Row::new(self.headers.iter().map(|h| h.as_str()).collect::<Vec<_>>())
                    .style(Style::default().fg(Color::Yellow))
            )
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().bg(Color::Blue))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut self.state);
    }
}

/// Create a sample table with demo data
pub fn create_demo_table() -> EnhancedTable {
    let mut table = EnhancedTable::new();
    
    table.set_headers(vec![
        "Name".to_string(),
        "Age".to_string(),
        "City".to_string(),
        "Status".to_string(),
    ]);

    table.set_data(vec![
        vec!["Alice".to_string(), "25".to_string(), "New York".to_string(), "Active".to_string()],
        vec!["Bob".to_string(), "30".to_string(), "London".to_string(), "Active".to_string()],
        vec!["Charlie".to_string(), "35".to_string(), "Tokyo".to_string(), "Inactive".to_string()],
        vec!["Diana".to_string(), "28".to_string(), "Paris".to_string(), "Active".to_string()],
    ]);

    table.set_widths(vec![
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Percentage(35),
        Constraint::Percentage(25),
    ]);

    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_creation() {
        let table = EnhancedTable::new();
        assert!(table.data.is_empty());
        assert!(table.headers.is_empty());
        assert_eq!(table.state.selected(), None);
    }

    #[test]
    fn test_navigation() {
        let mut table = create_demo_table();
        
        // Initially no selection
        assert_eq!(table.state.selected(), None);
        
        // Move to first item
        table.next();
        assert_eq!(table.state.selected(), Some(0));
        
        // Move to second item
        table.next();
        assert_eq!(table.state.selected(), Some(1));
        
        // Move back to first
        table.previous();
        assert_eq!(table.state.selected(), Some(0));
    }

    #[test]
    fn test_demo_table() {
        let table = create_demo_table();
        assert_eq!(table.data.len(), 4);
        assert_eq!(table.headers.len(), 4);
        assert_eq!(table.widths.len(), 4);
    }
}