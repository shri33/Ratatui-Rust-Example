use std::io;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut input = String::new();
    
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ].as_ref())
                .split(f.area());

            let title = Paragraph::new("Basic Form Test - Type something and press Esc to quit")
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().title("Test").borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            let input_paragraph = Paragraph::new(input.as_str())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().title("Input").borders(Borders::ALL));
            f.render_widget(input_paragraph, chunks[1]);

            let help = Paragraph::new("If you can type here, the TUI keyboard input is working correctly!")
                .style(Style::default().fg(Color::Green))
                .block(Block::default().title("Status").borders(Borders::ALL));
            f.render_widget(help, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Char(c) => {
                        input.push(c);
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    println!("Input received: {}", input);
    Ok(())
}
