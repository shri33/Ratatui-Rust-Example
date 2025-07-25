pub mod clipboard;
pub mod image;
pub mod input;
pub mod table;

// Re-export for easier access
pub use table::{InteractiveTable, TableData, SelectComponent, SelectOption, SelectType};
