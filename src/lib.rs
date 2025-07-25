pub mod app;
pub mod ui;
pub mod widgets;

// Re-export commonly used components
pub use widgets::table::{InteractiveTable, TableData};
pub use widgets::table::{SelectComponent, SelectOption, SelectType};
