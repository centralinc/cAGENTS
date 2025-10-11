// Interactive prompts and beautiful CLI output

use anyhow::Result;
use inquire::{Confirm, Select, Text};
use owo_colors::OwoColorize;
use std::io::IsTerminal;

/// Check if we should use interactive mode
pub fn is_interactive() -> bool {
    // Never be interactive in test environment or CI
    if is_test_env() || is_ci() {
        return false;
    }

    // Only interactive if stdin is a TTY
    std::io::stdin().is_terminal()
}

/// Check if running in CI environment
fn is_ci() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("CIRCLECI").is_ok()
}

/// Check if running in test environment
/// This checks multiple indicators:
/// 1. Compile-time test flag (cfg!(test))
/// 2. Runtime environment variable (CAGENTS_TEST)
/// 3. Rust test harness (thread name contains "test_")
fn is_test_env() -> bool {
    // Check compile-time flag
    if cfg!(test) {
        return true;
    }

    // Check environment variable
    if std::env::var("CAGENTS_TEST").is_ok() {
        return true;
    }

    // Check if running under cargo test by examining thread name
    // Rust test harness runs tests in threads named after the test function
    if let Some(name) = std::thread::current().name() {
        // Match test patterns: "test_foo", "module::test_foo", etc.
        if name.contains("test_") {
            return true;
        }
    }

    false
}

/// Prompt for text input
pub fn prompt_text(message: &str, default: Option<&str>) -> Result<String> {
    let mut prompt = Text::new(message);
    if let Some(def) = default {
        prompt = prompt.with_default(def);
    }
    Ok(prompt.prompt()?)
}

/// Prompt for selection
pub fn prompt_select(message: &str, options: &[&str]) -> Result<String> {
    let selection = Select::new(message, options.to_vec()).prompt()?;
    Ok(selection.to_string())
}

/// Prompt for confirmation
pub fn prompt_confirm(message: &str, default: bool) -> Result<bool> {
    Ok(Confirm::new(message).with_default(default).prompt()?)
}

/// Print a header with decoration
pub fn print_header(text: &str) {
    println!();
    println!("{}", "━".repeat(60).bright_black());
    println!("{}", text.bright_cyan().bold());
    println!("{}", "━".repeat(60).bright_black());
    println!();
}

/// Print a section header
pub fn print_section(emoji: &str, text: &str) {
    println!("{} {}", emoji, text.bright_white().bold());
}

/// Print success message
pub fn print_success(text: &str) {
    println!("{} {}", "✓".bright_green(), text.green());
}

/// Print info message
pub fn print_info(text: &str) {
    println!("{} {}", "→".bright_blue(), text.bright_blue());
}

/// Print warning message
pub fn print_warning(text: &str) {
    println!("{} {}", "▸ ".bright_yellow(), text.yellow());
}

/// Print error message
pub fn print_error(text: &str) {
    eprintln!("{} {}", "✗".bright_red(), text.red());
}

/// Print a list item
pub fn print_item(text: &str) {
    println!("   {} {}", "•".bright_black(), text);
}

/// Print a file path
pub fn print_file(prefix: &str, path: &str) {
    println!("   {} {}", prefix.green(), path.bright_white());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_not_interactive_in_tests() {
        // This test verifies that is_interactive() returns false when running under cargo test
        println!("Thread name: {:?}", std::thread::current().name());
        println!("RUST_TEST_THREADS: {:?}", std::env::var("RUST_TEST_THREADS"));
        println!("CAGENTS_TEST: {:?}", std::env::var("CAGENTS_TEST"));
        println!("stdin is terminal: {}", std::io::stdin().is_terminal());

        assert!(!is_interactive(), "is_interactive() should return false during tests");
    }
}
