// Create tests/integration_test.rs
#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_main_compiles() {
        let output = Command::new("cargo")
            .args(["check"])
            .output()
            .expect("Failed to execute cargo check");
        
        assert!(output.status.success(), "Main application should compile without errors");
    }

    #[test]
    fn test_interactive_form_compiles() {
        let output = Command::new("cargo")
            .args(["check", "--bin", "interactive_form"])
            .output()
            .expect("Failed to execute cargo check");
        
        assert!(output.status.success(), "Interactive form should compile without errors");
    }
}