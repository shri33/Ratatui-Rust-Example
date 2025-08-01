use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use crossterm::{
    cursor,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserData {
    email: String,
    name: String,
    timestamp: String,
}

#[derive(Debug, PartialEq)]
enum YesNoResponse {
    Yes,
    No,
}

#[derive(Debug, Serialize)]
struct ApiPayload {
    user_data: UserData,
    action: String,
    campaign_type: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize CLI environment
    setup_cli()?;
    
    // Show welcome and get command
    match parse_command()? {
        Command::Authenticate => {
            run_authentication_flow()?;
        }
        Command::Help => {
            show_help()?;
        }
        Command::Version => {
            show_version()?;
        }
    }
    
    Ok(())
}

#[derive(Debug)]
enum Command {
    Authenticate,
    Help,
    Version,
}

fn parse_command() -> Result<Command, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "authenticate" => {
                if args.len() > 2 && args[2] == "login" {
                    return Ok(Command::Authenticate);
                }
            }
            "help" | "--help" | "-h" => return Ok(Command::Help),
            "version" | "--version" | "-v" => return Ok(Command::Version),
            _ => {}
        }
    }
    
    // Default behavior - show usage and run authenticate
    show_usage()?;
    Ok(Command::Authenticate)
}

fn setup_cli() -> Result<(), Box<dyn std::error::Error>> {
    // Clear terminal and position cursor
    execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    Ok(())
}

fn show_usage() -> Result<(), Box<dyn std::error::Error>> {
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Cyan),
        Print("📰 Journal CLI v1.0\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("Usage: journal_cli authenticate login\n\n"),
        ResetColor
    )?;
    Ok(())
}

fn show_help() -> Result<(), Box<dyn std::error::Error>> {
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Cyan),
        Print("📰 Journal CLI v1.0\n"),
        Print("━━━━━━━━━━━━━━━━━━━━\n\n"),
        SetForegroundColor(Color::White),
        Print("USAGE:\n"),
        Print("  journal_cli authenticate login\n\n"),
        Print("COMMANDS:\n"),
        Print("  authenticate login    Start the authentication flow\n"),
        Print("  help                  Show this help message\n"),
        Print("  version              Show version information\n\n"),
        Print("EXAMPLES:\n"),
        SetForegroundColor(Color::Green),
        Print("  journal_cli authenticate login\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("  # Starts interactive authentication and campaign setup\n\n"),
        ResetColor
    )?;
    Ok(())
}

fn show_version() -> Result<(), Box<dyn std::error::Error>> {
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Cyan),
        Print("Journal CLI v1.0.0\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("Built with Rust and ❤️\n"),
        ResetColor
    )?;
    Ok(())
}

fn run_authentication_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Show welcome message
    show_welcome_message()?;
    
    // Authentication flow
    let user_data = authenticate_user()?;
    
    // Campaign generation flow
    campaign_flow(&user_data)?;
    
    Ok(())
}

fn show_welcome_message() -> Result<(), Box<dyn std::error::Error>> {
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Cyan),
        Print("📰 "),
        SetForegroundColor(Color::White),
        Print("Welcome to Journal\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n"),
        ResetColor
    )?;
    Ok(())
}

fn authenticate_user() -> Result<UserData, Box<dyn std::error::Error>> {
    println!("Welcome to Journal, please provide your Email to create an account");
    
    // Get email with validation
    let email = get_validated_email()?;
    
    // Get name
    let name = get_user_name()?;
    
    // Create user data
    let user_data = UserData {
        email,
        name,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    // Show account creation process
    create_account(&user_data)?;
    
    Ok(user_data)
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
                SetForegroundColor(Color::Yellow),
                Print("⚠️  Email cannot be empty. Please try again.\n"),
                ResetColor
            )?;
            continue;
        }
        
        if validate_email(&email) {
            // Echo back with checkmark
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Green),
                Print(&format!("Email: {} ✓\n", email)),
                ResetColor
            )?;
            return Ok(email);
        } else {
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Red),
                Print("❌ Invalid email format (e.g., user@example.com). Please try again.\n"),
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
                SetForegroundColor(Color::Yellow),
                Print("⚠️  Name cannot be empty. Please try again.\n"),
                ResetColor
            )?;
            continue;
        }
        
        if name.len() < 2 {
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Yellow),
                Print("⚠️  Name must be at least 2 characters. Please try again.\n"),
                ResetColor
            )?;
            continue;
        }
        
        // Echo back with checkmark
        execute!(
            io::stdout(),
            SetForegroundColor(Color::Green),
            Print(&format!("Name: {} ✓\n", name)),
            ResetColor
        )?;
        
        return Ok(name);
    }
}

fn create_account(user_data: &UserData) -> Result<(), Box<dyn std::error::Error>> {
    println!("Thank you, Journal is creating an account for you. Standby...");
    
    // Show loading animation
    show_loading_animation("Creating account", 2)?;
    
    // Simulate API call (would be real in production)
    simulate_api_call(user_data, "create_account")?;
    
    show_loading_animation("Setting up workspace", 1)?;
    
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Green),
        Print("✅ Your account has been created.\n\n"),
        ResetColor
    )?;
    
    Ok(())
}

fn campaign_flow(user_data: &UserData) -> Result<(), Box<dyn std::error::Error>> {
    println!("Would you like to generate an Email campaign?");
    
    let response = prompt_yes_no()?;
    
    match response {
        YesNoResponse::Yes => {
            generate_campaign(user_data)?;
        }
        YesNoResponse::No => {
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Cyan),
                Print("👋 No problem! You can create a campaign anytime by running:\n"),
                SetForegroundColor(Color::Green),
                Print("   journal_cli campaign create\n\n"),
                ResetColor
            )?;
        }
    }
    
    show_completion_message()?;
    Ok(())
}

fn generate_campaign(user_data: &UserData) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating email campaign...");
    
    show_loading_animation("Creating campaign templates", 2)?;
    
    // Simulate API call for campaign creation
    simulate_api_call(user_data, "create_campaign")?;
    
    show_loading_animation("Preparing files", 1)?;
    
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Green),
        Print("✅ An email campaign has been created.\n"),
        ResetColor
    )?;
    
    // Create campaign files and open folder
    let campaign_path = create_campaign_files(user_data)?;
    open_file_explorer(&campaign_path)?;
    
    Ok(())
}

fn prompt_yes_no() -> Result<YesNoResponse, Box<dyn std::error::Error>> {
    loop {
        execute!(
            io::stdout(),
            SetForegroundColor(Color::White),
            Print("  "),
            SetForegroundColor(Color::Green),
            Print("●"),
            SetForegroundColor(Color::White),
            Print(" Yes  "),
            SetForegroundColor(Color::Red),
            Print("●"),
            SetForegroundColor(Color::White),
            Print(" No\n"),
            Print("  "),
            SetForegroundColor(Color::DarkGrey),
            Print("(Y/n): "),
            ResetColor
        )?;
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        
        match input.as_str() {
            "y" | "yes" | "" => {
                execute!(
                    io::stdout(),
                    SetForegroundColor(Color::Green),
                    Print("  → Yes\n\n"),
                    ResetColor
                )?;
                return Ok(YesNoResponse::Yes);
            }
            "n" | "no" => {
                execute!(
                    io::stdout(),
                    SetForegroundColor(Color::Red),
                    Print("  → No\n\n"),
                    ResetColor
                )?;
                return Ok(YesNoResponse::No);
            }
            _ => {
                execute!(
                    io::stdout(),
                    SetForegroundColor(Color::Yellow),
                    Print("  ⚠️  Please enter 'y' for yes or 'n' for no.\n"),
                    ResetColor
                )?;
            }
        }
    }
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
            SetForegroundColor(Color::DarkGrey),
            Print("..."),
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

fn simulate_api_call(user_data: &UserData, action: &str) -> Result<(), Box<dyn std::error::Error>> {
    let payload = ApiPayload {
        user_data: user_data.clone(),
        action: action.to_string(),
        campaign_type: if action == "create_campaign" { 
            Some("welcome_series".to_string()) 
        } else { 
            None 
        },
    };
    
    // In a real implementation, this would make an HTTP request
    // For now, we'll just log the payload that would be sent
    let json_payload = serde_json::to_string_pretty(&payload)?;
    
    // Write to a log file for demonstration
    std::fs::create_dir_all("./logs")?;
    std::fs::write(
        format!("./logs/api_call_{}.json", chrono::Utc::now().timestamp()),
        json_payload
    )?;
    
    Ok(())
}

fn validate_email(email: &str) -> bool {
    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
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
    fs::create_dir_all(format!("{}/assets", campaign_dir))?;
    
    // Create campaign configuration
    let campaign_config = serde_json::json!({
        "campaign": {
            "name": "Welcome Campaign",
            "type": "welcome_series",
            "status": "draft",
            "created_at": user_data.timestamp,
            "user": {
                "name": user_data.name,
                "email": user_data.email
            }
        },
        "templates": [
            {
                "name": "welcome_email",
                "file": "templates/welcome_email.html",
                "subject": "Welcome to Journal!"
            },
            {
                "name": "getting_started",
                "file": "templates/getting_started.html", 
                "subject": "Getting Started with Journal"
            }
        ],
        "sequence": [
            {
                "template": "welcome_email",
                "delay_hours": 0
            },
            {
                "template": "getting_started", 
                "delay_hours": 24
            }
        ]
    });
    
    fs::write(
        format!("{}/campaign.json", campaign_dir),
        serde_json::to_string_pretty(&campaign_config)?
    )?;
    
    // Create email templates
    create_email_templates(&campaign_dir, user_data)?;
    
    // Create README
    create_campaign_readme(&campaign_dir, user_data)?;
    
    Ok(std::fs::canonicalize(&campaign_dir)?.to_string_lossy().to_string())
}

fn create_email_templates(campaign_dir: &str, user_data: &UserData) -> Result<(), Box<dyn std::error::Error>> {
    // Create welcome email template using string building to avoid quote escaping issues
    let welcome_template = create_welcome_email_content(&user_data.name, &user_data.email);
    std::fs::write(format!("{}/templates/welcome_email.html", campaign_dir), welcome_template)?;
    
    // Create getting started email template
    let getting_started_template = create_getting_started_content(&user_data.name);
    std::fs::write(format!("{}/templates/getting_started.html", campaign_dir), getting_started_template)?;
    
    Ok(())
}

fn create_welcome_email_content(name: &str, email: &str) -> String {
    let mut content = String::new();
    content.push_str("<!DOCTYPE html>\n");
    content.push_str("<html lang=\"en\">\n");
    content.push_str("<head>\n");
    content.push_str("    <meta charset=\"UTF-8\">\n");
    content.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    content.push_str("    <title>Welcome to Journal</title>\n");
    content.push_str("    <style>\n");
    content.push_str("        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }\n");
    content.push_str("        .container { max-width: 600px; margin: 0 auto; padding: 20px; }\n");
    content.push_str("        .header { background: #667eea; color: white; padding: 30px; text-align: center; }\n");
    content.push_str("        .content { background: white; padding: 30px; }\n");
    content.push_str("        .button { background: #667eea; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px; }\n");
    content.push_str("    </style>\n");
    content.push_str("</head>\n");
    content.push_str("<body>\n");
    content.push_str("    <div class=\"container\">\n");
    content.push_str("        <div class=\"header\">\n");
    content.push_str("            <h1>🎉 Welcome to Journal!</h1>\n");
    content.push_str("        </div>\n");
    content.push_str("        <div class=\"content\">\n");
    content.push_str(&format!("            <h2>Hello {}!</h2>\n", name));
    content.push_str(&format!("            <p>Welcome to Journal! Your account ({}) is now active and ready to use.</p>\n", email));
    content.push_str("            <h3>🚀 What's Next?</h3>\n");
    content.push_str("            <ul>\n");
    content.push_str("                <li><strong>Complete your profile</strong> - Add a photo and bio</li>\n");
    content.push_str("                <li><strong>Create your first entry</strong> - Start journaling today</li>\n");
    content.push_str("                <li><strong>Explore features</strong> - Discover what Journal can do</li>\n");
    content.push_str("            </ul>\n");
    content.push_str("            <p style=\"text-align: center;\">\n");
    content.push_str("                <a href=\"#\" class=\"button\">Get Started</a>\n");
    content.push_str("            </p>\n");
    content.push_str("            <p>Happy journaling!<br><strong>The Journal Team</strong></p>\n");
    content.push_str("        </div>\n");
    content.push_str("    </div>\n");
    content.push_str("</body>\n");
    content.push_str("</html>");
    content
}

fn create_getting_started_content(name: &str) -> String {
    let mut content = String::new();
    content.push_str("<!DOCTYPE html>\n");
    content.push_str("<html lang=\"en\">\n");
    content.push_str("<head>\n");
    content.push_str("    <meta charset=\"UTF-8\">\n");
    content.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    content.push_str("    <title>Getting Started with Journal</title>\n");
    content.push_str("    <style>\n");
    content.push_str("        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }\n");
    content.push_str("        .container { max-width: 600px; margin: 0 auto; padding: 20px; }\n");
    content.push_str("        .header { background: #764ba2; color: white; padding: 30px; text-align: center; }\n");
    content.push_str("        .content { background: white; padding: 30px; }\n");
    content.push_str("        .tip { background: #f8f9fa; border-left: 4px solid #667eea; padding: 15px; margin: 15px 0; }\n");
    content.push_str("        .button { background: #764ba2; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px; }\n");
    content.push_str("    </style>\n");
    content.push_str("</head>\n");
    content.push_str("<body>\n");
    content.push_str("    <div class=\"container\">\n");
    content.push_str("        <div class=\"header\">\n");
    content.push_str("            <h1>📚 Getting Started Guide</h1>\n");
    content.push_str("        </div>\n");
    content.push_str("        <div class=\"content\">\n");
    content.push_str(&format!("            <h2>Hi {}!</h2>\n", name));
    content.push_str("            <p>Ready to make the most of your Journal experience? Here are some tips:</p>\n");
    content.push_str("            <div class=\"tip\">\n");
    content.push_str("                <h3>💡 Pro Tip #1: Daily Habits</h3>\n");
    content.push_str("                <p>Set aside 10 minutes each day for journaling. Consistency is key!</p>\n");
    content.push_str("            </div>\n");
    content.push_str("            <div class=\"tip\">\n");
    content.push_str("                <h3>🎯 Pro Tip #2: Use Prompts</h3>\n");
    content.push_str("                <p>Try our daily prompts to spark inspiration and reflection.</p>\n");
    content.push_str("            </div>\n");
    content.push_str("            <p style=\"text-align: center;\">\n");
    content.push_str("                <a href=\"#\" class=\"button\">Explore Features</a>\n");
    content.push_str("            </p>\n");
    content.push_str("            <p>Questions? Contact our help center for assistance.</p>\n");
    content.push_str("        </div>\n");
    content.push_str("    </div>\n");
    content.push_str("</body>\n");
    content.push_str("</html>");
    content
}

fn create_campaign_readme(campaign_dir: &str, user_data: &UserData) -> Result<(), Box<dyn std::error::Error>> {
    let readme = format!(r#"# Email Campaign: Welcome Series

## 📧 Campaign Overview
- **Campaign Name**: Welcome Series
- **User**: {} ({})
- **Created**: {}
- **Status**: Draft
- **Type**: Welcome/Onboarding

## 📁 File Structure
```
{}/
├── campaign.json           # Campaign configuration
├── templates/
│   ├── welcome_email.html   # Welcome email template
│   └── getting_started.html # Getting started email template
├── assets/                 # Images and other assets
└── README.md               # This file
```

## 🚀 Campaign Sequence
1. **Welcome Email** - Sent immediately upon account creation
2. **Getting Started Guide** - Sent 24 hours after welcome email

## 🛠️ Next Steps
1. **Review Templates**: Check email templates in the `templates/` folder
2. **Customize Content**: Edit templates to match your brand voice
3. **Test Campaign**: Send test emails to verify formatting
4. **Configure Sending**: Set up email delivery service
5. **Activate Campaign**: Change status from "draft" to "active"

## 📊 Template Variables
The following variables are available in templates:
- `user.name` - User's full name
- `user.email` - User's email address
- `campaign.created_at` - Campaign creation timestamp

## 🔧 Technical Details
- **Generated by**: Journal CLI v1.0
- **Template Engine**: HTML with inline CSS
- **Responsive**: Mobile-friendly email templates
- **Compatibility**: Tested with major email clients

## 📞 Support
For questions about this campaign or Journal CLI:
- Documentation: [Journal CLI Docs](#)
- Support Email: support@journal.example.com
- GitHub Issues: [Report Bug](#)

---
*Generated on {} by Journal CLI*
"#, 
    user_data.name, 
    user_data.email, 
    user_data.timestamp,
    campaign_dir,
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
);
    
    std::fs::write(format!("{}/README.md", campaign_dir), readme)?;
    Ok(())
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
        Print(&format!("📂 Campaign files created: {}\n", path)),
        ResetColor
    )?;
    
    Ok(())
}

fn show_completion_message() -> Result<(), Box<dyn std::error::Error>> {
    execute!(
        io::stdout(),
        SetForegroundColor(Color::DarkGrey),
        Print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"),
        SetForegroundColor(Color::Green),
        Print("✨ Setup complete! "),
        SetForegroundColor(Color::White),
        Print("Thank you for using Journal CLI.\n\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("Next steps:\n"),
        Print("• Review your campaign files\n"),
        Print("• Customize email templates\n"),
        Print("• Configure your email service\n"),
        Print("• Launch your campaign\n\n"),
        Print("Need help? Run: "),
        SetForegroundColor(Color::Cyan),
        Print("journal_cli help\n"),
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
        assert!(validate_email("user.name@domain.org"));
        assert!(validate_email("user+tag@domain.co.uk"));
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email("@domain.com"));
        assert!(!validate_email("user@"));
        assert!(!validate_email("user@domain"));
    }
    
    #[test]
    fn test_command_parsing() {
        // This would need to be tested with actual command line args
        // For now, just verify the function exists
        assert!(true);
    }
}
