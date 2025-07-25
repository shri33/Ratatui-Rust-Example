use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell},
    Terminal,
};

// For clipboard functionality
struct Clipboard;

impl Clipboard {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }
    
    fn set_text(&self, _text: String) {
        // Implementation placeholder
    }
    
    fn get_text(&self) -> Option<String> {
        Some("Clipboard content".to_string())
    }
}

#[derive(PartialEq)]
enum Mode {
    Table,
    Image,
    Input,
    Form,
}

struct App {
    mode: Mode,
    status: String,
    input: String,
    clipboard: Clipboard,
    emoji_picker: Vec<&'static str>,
    emoji_index: usize,
    link: String,
}

impl App {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            mode: Mode::Table,
            status: "Press 1:Table 2:Image 3:Input 4:Form | q:Quit".to_string(),
            input: String::new(),
            clipboard: Clipboard::new()?,
            emoji_picker: vec!["😀", "😂", "🥳", "🚀", "❤️", "👍", "🔥", "🎉"],
            emoji_index: 0,
            link: "https://ratatui.rs".to_string(),
        })
    }

    fn on_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Table => match key.code {
                KeyCode::Char('2') => self.mode = Mode::Image,
                KeyCode::Char('3') => self.mode = Mode::Input,
                KeyCode::Char('4') => self.mode = Mode::Form,
                KeyCode::Char('q') => self.status = "quit".to_string(),
                _ => {}
            },
            Mode::Image => match key.code {
                KeyCode::Char('1') => self.mode = Mode::Table,
                KeyCode::Char('3') => self.mode = Mode::Input,
                KeyCode::Char('4') => self.mode = Mode::Form,
                KeyCode::Char('q') => self.status = "quit".to_string(),
                _ => {}
            },
            Mode::Input => match key.code {
                KeyCode::Char('1') => self.mode = Mode::Table,
                KeyCode::Char('2') => self.mode = Mode::Image,
                KeyCode::Char('4') => self.mode = Mode::Form,
                KeyCode::Char('q') => self.status = "quit".to_string(),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.clipboard.set_text(self.input.clone());
                    self.status = "Copied to clipboard!".to_string();
                },
                KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(text) = self.clipboard.get_text() {
                        self.input.push_str(&text);
                        self.status = "Pasted from clipboard!".to_string();
                    }
                },
                KeyCode::Char('e') => {
                    self.input.push_str(self.emoji_picker[self.emoji_index]);
                    self.emoji_index = (self.emoji_index + 1) % self.emoji_picker.len();
                },
                KeyCode::Char('l') => {
                    self.status = format!("Attempted to open link: {}", self.link);
                },
                KeyCode::Backspace => {
                    self.input.pop();
                },
                KeyCode::Char(c) => {
                    self.input.push(c);
                },
                _ => {}
            },
            Mode::Form => match key.code {
                KeyCode::Char('1') => self.mode = Mode::Table,
                KeyCode::Char('2') => self.mode = Mode::Image,
                KeyCode::Char('3') => self.mode = Mode::Input,
                KeyCode::Char('q') => self.status = "quit".to_string(),
                KeyCode::Enter => {
                    // Launch interactive form
                    self.status = "Launching interactive form...".to_string();
                },
                _ => {}
            },
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let size = f.area();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ].as_ref())
        .split(size);
    
    // Title
    let title = Paragraph::new("🦀 Ratatui All-in-One Demo")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(title, chunks[0]);
    
    // Content based on mode
    match app.mode {
        Mode::Table => {
            let rows = vec![
                Row::new(vec![
                    Cell::from(Span::styled("Feature", Style::default().add_modifier(Modifier::BOLD))),
                    Cell::from(Span::styled("Type", Style::default().add_modifier(Modifier::BOLD))),
                    Cell::from(Span::styled("Supports", Style::default().add_modifier(Modifier::BOLD))),
                ]),
                Row::new(vec![
                    Cell::from("Table"),
                    Cell::from("Widget"),
                    Cell::from("Sorting, Selection"),
                ]),
                Row::new(vec![
                    Cell::from("Image"),
                    Cell::from("High-Res"),
                    Cell::from("Kitty/Sixel"),
                ]),
                Row::new(vec![
                    Cell::from("Input"),
                    Cell::from("Text/Emoji"),
                    Cell::from("Clipboard, Links"),
                ]),
            ];
            
            let column_widths = [
                Constraint::Length(10),
                Constraint::Length(15),
                Constraint::Length(25),
            ];
            
            let table = Table::new(rows, column_widths)
                .block(Block::default().borders(Borders::ALL).title("Feature Table"));
            f.render_widget(table, chunks[1]);
        },
        Mode::Image => {
            let content = Paragraph::new("Press 'h' to show the image in high resolution using viuer.\n\nMake sure your terminal supports Kitty or Sixel graphics protocols for best results.\n\nPress 1:Table 3:Input 4:Form q:Quit")
                .block(Block::default().borders(Borders::ALL).title("Image Mode"));
            f.render_widget(content, chunks[1]);
        },
        Mode::Input => {
            let input_box = Paragraph::new(Text::from(vec![
                Line::from(vec![
                    Span::styled("Input: ", Style::default().fg(Color::Green)),
                    Span::raw(&app.input),
                ]),
                Line::from(vec![
                    Span::styled("Next Emoji: ", Style::default().fg(Color::Magenta)),
                    Span::raw(app.emoji_picker[app.emoji_index]),
                ]),
                Line::from(vec![
                    Span::styled("Link: ", Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED)),
                    Span::raw(&app.link),
                    Span::raw(" (press 'l' to open)"),
                ]),
                Line::from(vec![
                    Span::styled("Navigation: ", Style::default().fg(Color::Cyan)),
                    Span::raw("1:Table 2:Image 3:Input 4:Form q:Quit"),
                ]),
                Line::from("Ctrl+C:Copy  Ctrl+V:Paste  e:Insert Emoji  l:Open Link"),
            ]))
            .block(Block::default().borders(Borders::ALL).title("Input Mode"));
            f.render_widget(input_box, chunks[1]);
        },
        Mode::Form => {
            let content = Paragraph::new("Press Enter to open the interactive form\n\nThis will open a separate screen with a complete form interface.\n\nPress 1:Table 2:Image 3:Input q:Quit")
                .block(Block::default().borders(Borders::ALL).title("Interactive Form Mode"));
            f.render_widget(content, chunks[1]);
        }
    }
    
    // Status bar
    let status = Paragraph::new(app.status.as_str())
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(status, chunks[2]);
}

fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new()?;

    loop {
        // Draw UI
        terminal.draw(|f| ui(f, &app))?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key);
                
                // Special handling for Form mode and Enter key
                if app.mode == Mode::Form && key.code == KeyCode::Enter {
                    // Save terminal state
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    
                    // Launch the interactive form as a separate process
                    let status = std::process::Command::new("cargo")
                        .args(["run", "--bin", "interactive_form"])
                        .status()?;
                        
                    if !status.success() {
                        eprintln!("Interactive form exited with error");
                    }
                    
                    // Restore terminal state
                    enable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        EnterAlternateScreen, 
                        EnableMouseCapture
                    )?;
                }
            }
        }

        // Check if we should exit
        if app.status == "quit" {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_app()
}
