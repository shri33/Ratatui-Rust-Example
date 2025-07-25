//! Main Menu - Showcase all Ratatui features
//! Navigate through all implemented examples

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Clear},
    Terminal, Frame,
};
use std::{
    error::Error,
    io,
    process::Command,
    time::{Duration, Instant},
};

#[derive(Clone)]
struct MenuItem {
    name: String,
    description: String,
    binary_name: String,
    status: String,
}

struct MainMenuApp {
    items: Vec<MenuItem>,
    list_state: ListState,
    should_quit: bool,
    show_help: bool,
    selected_index: usize,
}

impl MainMenuApp {
    fn new() -> Self {
        let items = vec![
            MenuItem {
                name: "📥 Interactive Form".to_string(),
                description: "User input with history scrollback (Vue.js style)".to_string(),
                binary_name: "interactive_form".to_string(),
                status: "✅ Complete".to_string(),
            },
            MenuItem {
                name: "✅ Select Components".to_string(),
                description: "Dropdown, radio, checkbox with navigation".to_string(),
                binary_name: "table_example".to_string(),
                status: "✅ Complete".to_string(),
            },
            MenuItem {
                name: "📊 Charts Demo".to_string(),
                description: "Bar charts and histograms".to_string(),
                binary_name: "charts_demo".to_string(),
                status: "✅ Complete".to_string(),
            },
            MenuItem {
                name: "🔼 Interactive Table".to_string(),
                description: "Row/column highlighting with Shift+Arrow".to_string(),
                binary_name: "interactive_table".to_string(),
                status: "✅ Complete".to_string(),
            },
            MenuItem {
                name: "🎛️ Dashboard".to_string(),
                description: "Multi-tab dashboard with widgets".to_string(),
                binary_name: "dashboard".to_string(),
                status: "✅ Complete".to_string(),
            },
            MenuItem {
                name: "🖼️ Image Viewer".to_string(),
                description: "High-resolution image display".to_string(),
                binary_name: "image_viewer".to_string(),
                status: "✅ Complete".to_string(),
            },
            MenuItem {
                name: "🎬 Video Player".to_string(),
                description: "Video playbook with file browser".to_string(),
                binary_name: "video_player".to_string(),
                status: "✅ Complete".to_string(),
            },
            MenuItem {
                name: "😀 Emoji Picker".to_string(),
                description: "Grid-based emoji selection".to_string(),
                binary_name: "emoji_picker".to_string(),
                status: "✅ Complete".to_string(),
            },
            MenuItem {
                name: "🎨 ASCII Art".to_string(),
                description: "ASCII art display and animation".to_string(),
                binary_name: "ascii_art".to_string(),
                status: "✅ Complete".to_string(),
            },
        ];

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            items,
            list_state,
            should_quit: false,
            show_help: false,
            selected_index: 0,
        }
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_index = i;
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_index = i;
    }

    fn launch_selected(&self) -> Result<(), Box<dyn Error>> {
        if let Some(item) = self.items.get(self.selected_index) {
            // Restore terminal before launching
            disable_raw_mode()?;
            execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
            
            // Launch the selected example
            let mut cmd = Command::new("cargo");
            cmd.args(&["run", "--bin", &item.binary_name]);
            
            let status = cmd.status()?;
            
            // Re-setup terminal
            enable_raw_mode()?;
            execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
            
            if !status.success() {
                eprintln!("Failed to run example: {}", item.binary_name);
            }
        }
        Ok(())
    }
}

fn ui(f: &mut Frame, app: &MainMenuApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("🦀 Ratatui Advanced UI Demo - Main Menu")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Main content area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    // Menu list
    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|item| {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(&item.name, Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::styled(&item.status, Style::default().fg(Color::Green)),
                ]),
                Line::from(Span::styled(&item.description, Style::default().fg(Color::Gray))),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Examples"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, main_chunks[0], &mut app.list_state.clone());

    // Description panel
    if let Some(selected_item) = app.items.get(app.selected_index) {
        let description_text = format!(
            "Selected: {}\n\nDescription:\n{}\n\nBinary: {}\n\nStatus: {}",
            selected_item.name,
            selected_item.description,
            selected_item.binary_name,
            selected_item.status
        );

        let description = Paragraph::new(description_text)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(description, main_chunks[1]);
    }

    // Instructions
    let instructions = if app.show_help {
        "↑↓: Navigate | Enter: Launch | h: Hide help | q: Quit\n\
         All examples demonstrate advanced Ratatui features:\n\
         • Interactive forms with validation\n\
         • Complex table navigation\n\
         • Charts and data visualization"
    } else {
        "↑↓: Navigate | Enter: Launch example | h: Help | q: Quit"
    };

    let help = Paragraph::new(instructions)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .style(Style::default().fg(Color::Yellow))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(help, chunks[2]);

    // Show help overlay if requested
    if app.show_help {
        let area = centered_rect(80, 60, f.area());
        f.render_widget(Clear, area);
        
        let help_text = "🦀 Ratatui Advanced UI Demo\n\n\
            This project demonstrates advanced terminal UI concepts:\n\n\
            📥 Interactive Forms: Vue.js-style input with validation\n\
            ✅ Select Components: Dropdown, radio, checkbox widgets\n\
            📊 Charts: Bar graphs and histograms in terminal\n\
            🔼 Table Navigation: Row/column highlighting\n\
            ➡️ Multi-selection: Shift+Arrow key support\n\n\
            All components are modular and reusable!\n\n\
            Press 'h' to close this help.";
        
        let help_popup = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title(" Help "))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(help_popup, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = MainMenuApp::new();
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('h') => {
                            app.show_help = !app.show_help;
                        }
                        KeyCode::Down => {
                            app.next();
                        }
                        KeyCode::Up => {
                            app.previous();
                        }
                        KeyCode::Enter => {
                            if let Err(e) = app.launch_selected() {
                                eprintln!("Error launching example: {}", e);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.should_quit {
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