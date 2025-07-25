use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,  // Remove Span
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    fs, 
    io,
    path::{Path, PathBuf},
};  // Remove Duration, Instant

#[derive(Debug)]
#[allow(dead_code)]  // Add this to suppress warnings
enum PlayerError {
    Io(io::Error),
    Other(String),  // Keep this even if unused - might be useful later
}

impl From<io::Error> for PlayerError {
    fn from(err: io::Error) -> Self {
        PlayerError::Io(err)
    }
}

#[derive(Clone, Copy)]
enum AppMode {
    FileBrowser,
    VideoPlayer,
    Help,
}

struct FileBrowser {
    current_dir: PathBuf,
    items: Vec<String>,
    selected: usize,  // Fix: Use 'selected' instead of 'selected_index'
    list_state: ListState,  // Fix: Add ListState
}

impl FileBrowser {
    fn new() -> Result<Self, PlayerError> {
        let mut browser = Self {
            current_dir: std::env::current_dir()?,
            items: Vec::new(),
            selected: 0,
            list_state: ListState::default(),
        };
        browser.refresh_items()?;
        Ok(browser)
    }

    fn refresh_items(&mut self) -> Result<(), PlayerError> {
        self.items.clear();
        
        if let Some(_parent) = self.current_dir.parent() {  // Fix: Add underscore
            self.items.push("..".to_string());
        }

        let entries = fs::read_dir(&self.current_dir)?;
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            if entry.path().is_dir() || self.is_video_file(&entry.path()) {
                self.items.push(file_name);
            }
        }
        
        if self.selected >= self.items.len() && !self.items.is_empty() {
            self.selected = self.items.len() - 1;
        }
        
        self.list_state.select(Some(self.selected));
        Ok(())
    }

    fn is_video_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(
                ext.to_string_lossy().to_lowercase().as_str(),
                "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm"
            )
        } else {
            false
        }
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
        }
    }

    fn move_down(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
            self.list_state.select(Some(self.selected));
        }
    }

    fn select_current(&mut self) -> Result<Option<PathBuf>, PlayerError> {
        if self.items.is_empty() {
            return Ok(None);
        }

        let selected_item = &self.items[self.selected];
        
        if selected_item == ".." {
            if let Some(parent) = self.current_dir.parent() {
                self.current_dir = parent.to_path_buf();
                self.selected = 0;
                self.refresh_items()?;
            }
            return Ok(None);
        }

        let selected_path = self.current_dir.join(selected_item);
        
        if selected_path.is_dir() {
            self.current_dir = selected_path;
            self.selected = 0;
            self.refresh_items()?;
            Ok(None)
        } else if self.is_video_file(&selected_path) {
            Ok(Some(selected_path))
        } else {
            Ok(None)
        }
    }
}

struct VideoPlayerApp {
    mode: AppMode,
    should_quit: bool,
    status_message: String,
    error_message: Option<String>,
    file_browser: FileBrowser,
    current_video_path: Option<PathBuf>,  // Fix: Use PathBuf instead of String
    is_playing: bool,
}

impl VideoPlayerApp {
    fn new() -> Self {
        Self {
            mode: AppMode::FileBrowser,
            should_quit: false,
            status_message: "Navigate with arrows, Enter to select, 'q' to quit".to_string(),
            error_message: None,
            file_browser: FileBrowser::new().unwrap_or_else(|_| FileBrowser {
                current_dir: std::env::current_dir().unwrap_or_default(),
                items: Vec::new(),
                selected: 0,
                list_state: ListState::default(),
            }),
            current_video_path: None,
            is_playing: false,
        }
    }

    fn on_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match self.mode {
            AppMode::FileBrowser => self.handle_browser_key(key, modifiers),
            AppMode::VideoPlayer => self.handle_player_key(key, modifiers),
            AppMode::Help => self.handle_help_key(key, modifiers),
        }
    }

    fn handle_browser_key(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('h') => self.mode = AppMode::Help,
            KeyCode::Up => self.file_browser.move_up(),
            KeyCode::Down => self.file_browser.move_down(),
            KeyCode::Enter => {
                match self.file_browser.select_current() {
                    Ok(Some(path)) => {
                        self.current_video_path = Some(path.clone());
                        self.status_message = format!("Selected: {}", path.display());
                        self.mode = AppMode::VideoPlayer;
                    }
                    Ok(None) => {
                        // Directory navigation handled in select_current
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Error: {:?}", e));
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_player_key(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('b') => self.mode = AppMode::FileBrowser,
            KeyCode::Char('h') => self.mode = AppMode::Help,
            KeyCode::Char(' ') => {
                self.is_playing = !self.is_playing;
                self.status_message = if self.is_playing { "Playing..." } else { "Paused" }.to_string();
            }
            _ => {}
        }
    }

    fn handle_help_key(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc | KeyCode::Char('h') => self.mode = AppMode::FileBrowser,
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        match self.mode {
            AppMode::FileBrowser => self.render_file_browser(frame),
            AppMode::VideoPlayer => self.render_video_player(frame),
            AppMode::Help => self.render_help(frame),
        }

        if let Some(ref error) = self.error_message {
            self.render_error_popup(frame, error);
        }
    }

    fn render_file_browser(&mut self, frame: &mut Frame) {
        let size = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(size);

        // Title
        let title = Paragraph::new("Video Player - File Browser")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // File list - Fix: Store items in a local variable first
        let display_items = self.file_browser.items.iter().map(|item| {
            let path = self.file_browser.current_dir.join(item);
            let style = if path.is_dir() {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::Green)
            };
            ListItem::new(item.clone()).style(style)
        }).collect::<Vec<_>>();

        let list = List::new(display_items)
            .block(Block::default().borders(Borders::ALL).title("Files"))
            .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD));

        frame.render_stateful_widget(list, chunks[1], &mut self.file_browser.list_state);

        // Status
        let current_dir = format!("Current: {}", self.file_browser.current_dir.display());
        let status = Paragraph::new(vec![
            Line::from(current_dir),
            Line::from(self.status_message.clone()),
        ])
        .block(Block::default().borders(Borders::ALL).title("Status"));
        frame.render_widget(status, chunks[2]);
    }

    fn render_video_player(&self, frame: &mut Frame) {
        let size = frame.area();  // Fix: Use frame.area()

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(5),
            ])
            .split(size);

        // Title
        let title_text = if let Some(ref path) = self.current_video_path {
            format!("Video Player - {}", path.file_name().unwrap_or_default().to_string_lossy())  // Fix: Handle file_name properly
        } else {
            "Video Player".to_string()
        };

        let title = Paragraph::new(title_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Video area (placeholder)
        let video_placeholder = Paragraph::new("Video would be displayed here\n(Feature not implemented)")
            .block(Block::default().borders(Borders::ALL).title("Video"))
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(video_placeholder, chunks[1]);

        // Controls
        let controls_text = vec![
            Line::from(format!("Status: {}", if self.is_playing { "Playing" } else { "Paused" })),
            Line::from("Controls: [Space] Play/Pause, [b] Back to browser, [q] Quit, [h] Help"),
        ];

        let controls = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .wrap(Wrap { trim: true });
        frame.render_widget(controls, chunks[2]);
    }

    fn render_help(&self, frame: &mut Frame) {
        let size = frame.area();  // Fix: Use frame.area()

        let help_text = vec![
            Line::from("Video Player Help"),
            Line::from(""),
            Line::from("File Browser:"),
            Line::from("  ↑/↓ - Navigate files"),
            Line::from("  Enter - Select file/directory"),
            Line::from("  h - Show this help"),
            Line::from("  q - Quit"),
            Line::from(""),
            Line::from("Video Player:"),
            Line::from("  Space - Play/Pause"),
            Line::from("  b - Back to file browser"),
            Line::from("  h - Show this help"),
            Line::from("  q - Quit"),
            Line::from(""),
            Line::from("Press 'h' or Esc to close this help"),
        ];

        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        let area = centered_rect(80, 80, size);
        frame.render_widget(Clear, area);
        frame.render_widget(help, area);
    }

    fn render_error_popup(&self, frame: &mut Frame, error: &str) {
        let size = frame.area();  // Fix: Use frame.area()
        let area = centered_rect(60, 20, size);
        
        let error_text = Paragraph::new(error)
            .block(Block::default().borders(Borders::ALL).title("Error"))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(Clear, area);
        frame.render_widget(error_text, area);
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = VideoPlayerApp::new();

    loop {
        terminal.draw(|f| app.render(f))?;

        if let Event::Key(key) = event::read()? {
            app.on_key(key.code, key.modifiers);
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
