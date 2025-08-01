use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use crossterm::{
    cursor,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

#[derive(Debug, Clone)]
struct UserData {
    email: String,
    name: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() >= 3 && args[1] == "authenticate" && args[2] == "login" {
        run_journal_flow()?;
    } else {
        show_usage();
    }
    
    Ok(())
}

fn show_usage() {
    println!("Usage: cargo run --bin journal_cli authenticate login");
}

fn run_journal_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Clear screen for clean start
    clear_screen()?;
    
    // Show welcome header with image placeholder
    show_welcome_header()?;
    
    // Collect user information
    let user_data = collect_user_info()?;
    
    // Create account with loading animation
    create_account(&user_data)?;
    
    // Ask about email campaign
    if prompt_campaign_generation()? {
        generate_campaign(&user_data)?;
    }
    
    Ok(())
}

fn clear_screen() -> Result<(), Box<dyn std::error::Error>> {
    execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    Ok(())
}

fn show_welcome_header() -> Result<(), Box<dyn std::error::Error>> {
    println!();
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Cyan),
        Print("╔══════════════════════════════════════════════════╗\n"),
        Print("║                                                  ║\n"),
        Print("║     📰 Welcome to Journal                        ║\n"),
        Print("║     Your personal journaling companion           ║\n"),
        Print("║                                                  ║\n"),
        Print("╚══════════════════════════════════════════════════╝\n"),
        ResetColor
    )?;
    println!();
    Ok(())
}

fn collect_user_info() -> Result<UserData, Box<dyn std::error::Error>> {
    println!("Welcome to Journal, please provide your Email to create an account");
    
    // Get email with validation
    let email = get_validated_email()?;
    
    // Get name
    let name = get_user_name()?;
    
    Ok(UserData { email, name })
}

fn get_validated_email() -> Result<String, Box<dyn std::error::Error>> {
    loop {
        print!("Email: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let email = input.trim().to_string();
        
        if email.is_empty() {
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Red),
                Print("❌ Email cannot be empty. Please try again.\n"),
                ResetColor
            )?;
            continue;
        }
        
        if validate_email(&email) {
            // Echo back the entered email
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Green),
                Print(&format!("Email: {}\n", email)),
                ResetColor
            )?;
            return Ok(email);
        } else {
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Red),
                Print("❌ Invalid email format. Please enter a valid email (e.g., user@example.com)\n"),
                ResetColor
            )?;
        }
    }
}

fn get_user_name() -> Result<String, Box<dyn std::error::Error>> {
    loop {
        print!("Name: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let name = input.trim().to_string();
        
        if name.is_empty() {
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Red),
                Print("❌ Name cannot be empty. Please try again.\n"),
                ResetColor
            )?;
            continue;
        }
        
        if name.len() < 2 {
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Red),
                Print("❌ Name must be at least 2 characters long.\n"),
                ResetColor
            )?;
            continue;
        }
        
        // Echo back the entered name
        execute!(
            io::stdout(),
            SetForegroundColor(Color::Green),
            Print(&format!("Name: {}\n", name)),
            ResetColor
        )?;
        
        return Ok(name);
    }
}

fn create_account(user_data: &UserData) -> Result<(), Box<dyn std::error::Error>> {
    println!("Thank you, Journal is creating an account for you. Standby...");
    
    // Show loading animation
    show_loading_animation("Creating your account", 3)?;
    
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Green),
        Print("✅ Your account has been created.\n"),
        ResetColor
    )?;
    
    // Simulate sending data to web service
    simulate_web_service_call(user_data)?;
    
    Ok(())
}

fn prompt_campaign_generation() -> Result<bool, Box<dyn std::error::Error>> {
    println!("Would you like to generate an Email campaign?");
    
    loop {
        print!("Yes / No: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim().to_lowercase();
        
        match choice.as_str() {
            "yes" | "y" => {
                execute!(
                    io::stdout(),
                    SetForegroundColor(Color::Green),
                    Print("→ Yes\n"),
                    ResetColor
                )?;
                return Ok(true);
            }
            "no" | "n" => {
                execute!(
                    io::stdout(),
                    SetForegroundColor(Color::Yellow),
                    Print("→ No\n"),
                    ResetColor
                )?;
                return Ok(false);
            }
            _ => {
                execute!(
                    io::stdout(),
                    SetForegroundColor(Color::Red),
                    Print("❌ Please enter 'Yes' or 'No'\n"),
                    ResetColor
                )?;
            }
        }
    }
}

fn generate_campaign(user_data: &UserData) -> Result<(), Box<dyn std::error::Error>> {
    show_loading_animation("Generating email campaign", 2)?;
    
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Green),
        Print("✅ An email campaign has been created.\n"),
        ResetColor
    )?;
    
    // Create campaign files
    let campaign_path = create_campaign_files(user_data)?;
    
    // Open file explorer
    open_file_explorer(&campaign_path)?;
    
    Ok(())
}

fn show_loading_animation(message: &str, duration_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let total_iterations = duration_secs * 10; // 100ms per frame
    
    for i in 0..total_iterations {
        let frame_idx = (i as usize) % frames.len();
        let frame = frames[frame_idx];
        
        execute!(
            io::stdout(),
            cursor::SavePosition,
            SetForegroundColor(Color::Cyan),
            Print(frame),
            SetForegroundColor(Color::White),
            Print(&format!(" {}", message)),
            ResetColor
        )?;
        
        io::stdout().flush()?;
        thread::sleep(Duration::from_millis(100));
        
        execute!(
            io::stdout(),
            cursor::RestorePosition,
            Clear(ClearType::FromCursorDown)
        )?;
    }
    
    Ok(())
}

fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

fn simulate_web_service_call(user_data: &UserData) -> Result<(), Box<dyn std::error::Error>> {
    // Create logs directory
    std::fs::create_dir_all("./logs")?;
    
    // Create API payload
    let payload = format!(
        r#"{{
  "timestamp": "{}",
  "action": "create_account",
  "user_data": {{
    "email": "{}",
    "name": "{}"
  }},
  "status": "success"
}}"#,
        chrono::Utc::now().to_rfc3339(),
        user_data.email,
        user_data.name
    );
    
    // Write to log file
    let log_file = format!("./logs/api_call_{}.json", chrono::Utc::now().timestamp());
    std::fs::write(log_file, payload)?;
    
    Ok(())
}

fn create_campaign_files(user_data: &UserData) -> Result<String, Box<dyn std::error::Error>> {
    use std::fs;
    
    // Create campaign directory
    let safe_email = user_data.email.replace('@', "_at_").replace('.', "_");
    let campaign_name = format!("journal_campaign_{}", safe_email);
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let campaign_dir = format!("./campaigns/{}_{}", campaign_name, timestamp);
    
    // Create directory structure
    fs::create_dir_all(&campaign_dir)?;
    fs::create_dir_all(format!("{}/templates", campaign_dir))?;
    
    // Create campaign.json
    let campaign_config = format!(
        r#"{{
  "campaign_name": "Welcome Series",
  "user": {{
    "name": "{}",
    "email": "{}"
  }},
  "created_at": "{}",
  "status": "draft",
  "templates": [
    "welcome_email.html",
    "follow_up.html"
  ]
}}"#,
        user_data.name,
        user_data.email,
        chrono::Utc::now().to_rfc3339()
    );
    
    fs::write(format!("{}/campaign.json", campaign_dir), campaign_config)?;
    
    // Create welcome email template
    let welcome_html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Welcome to Journal</title>
    <style>
        body {{ font-family: Arial, sans-serif; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #007acc; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 20px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Welcome to Journal!</h1>
        </div>
        <div class="content">
            <h2>Hello {}!</h2>
            <p>Thank you for joining Journal. Your account ({}) is now active.</p>
            <p>Start your journaling journey today!</p>
            <p>Best regards,<br>The Journal Team</p>
        </div>
    </div>
</body>
</html>"#,
        user_data.name, user_data.email
    );
    
    fs::write(format!("{}/templates/welcome_email.html", campaign_dir), welcome_html)?;
    
    // Create README
    let readme = format!(
        r#"# Email Campaign: Welcome Series

## Campaign Details
- **User**: {} ({})
- **Created**: {}
- **Status**: Draft

## Files
- `campaign.json` - Campaign configuration
- `templates/welcome_email.html` - Welcome email template

## Next Steps
1. Review and customize the email template
2. Configure your email service
3. Test the campaign
4. Activate when ready

Generated by Journal CLI
"#,
        user_data.name,
        user_data.email,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    
    fs::write(format!("{}/README.md", campaign_dir), readme)?;
    
    Ok(std::fs::canonicalize(&campaign_dir)?.to_string_lossy().to_string())
}

fn open_file_explorer(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Cyan),
        Print("📁 Opening campaign folder...\n"),
        ResetColor
    )?;
    
    // Platform-specific file explorer opening
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()?;
    }
    
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Green),
        Print(&format!("📂 Campaign files created in: {}\n", path)),
        ResetColor
    )?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com"));
        assert!(validate_email("user@domain.org"));
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email("@domain.com"));
        assert!(!validate_email("user@"));
    }
}
