// Git integration helpers

use anyhow::Result;
use owo_colors::OwoColorize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Add generated output files to .gitignore based on config
pub fn ignore_outputs() -> Result<()> {
    let gitignore_path = PathBuf::from(".gitignore");

    // Load config to check output targets
    let config = crate::config::load_config_with_precedence().ok();
    let output_targets = config
        .as_ref()
        .and_then(|c| c.output.as_ref())
        .and_then(|o| o.targets.as_ref())
        .cloned()
        .unwrap_or_else(|| vec!["agents-md".to_string()]);

    // Build patterns based on configured outputs
    let mut patterns = vec!["# cAGENTS generated files".to_string()];

    for target in &output_targets {
        match target.as_str() {
            "agents-md" => {
                patterns.push("AGENTS.md".to_string());
                patterns.push("**/AGENTS.md".to_string());
            }
            "claude-md" => {
                patterns.push("CLAUDE.md".to_string());
                patterns.push("**/CLAUDE.md".to_string());
            }
            "cursorrules" => {
                patterns.push(".cursorrules".to_string());
                patterns.push("**/.cursorrules".to_string());
            }
            _ => {}
        }
    }

    // Check if .gitignore exists
    let existing_content = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    // Check if any patterns already exist
    let has_header = existing_content.contains("cAGENTS generated");
    if has_header {
        println!("{} {}", "ℹ️ ".bright_blue(), "Output patterns already in .gitignore".bright_blue());
        return Ok(());
    }

    // Prompt for confirmation if interactive
    if crate::interactive::is_interactive() {
        use inquire::Confirm;

        let should_add = Confirm::new("Add output files to .gitignore?")
            .with_default(true)
            .prompt()?;

        if !should_add {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Append patterns
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&gitignore_path)?;

    // Add newline if file doesn't end with one
    if !existing_content.is_empty() && !existing_content.ends_with('\n') {
        writeln!(file)?;
    }

    // Add patterns
    writeln!(file)?;
    for pattern in &patterns {
        writeln!(file, "{}", pattern)?;
    }

    println!("{} {}", "✅".bright_green(), "Updated .gitignore".green().bold());
    println!();
    for pattern in patterns.iter().skip(1) { // Skip header comment
        println!("  {} {}", "+".green(), pattern.bright_white());
    }

    Ok(())
}

/// Remove all cAGENTS output patterns from .gitignore
pub fn unignore_outputs() -> Result<()> {
    let gitignore_path = PathBuf::from(".gitignore");

    if !gitignore_path.exists() {
        println!("{} {}", "ℹ️ ".bright_blue(), ".gitignore doesn't exist".bright_blue());
        return Ok(());
    }

    let content = fs::read_to_string(&gitignore_path)?;
    let lines: Vec<&str> = content.lines().collect();

    // Filter out all cAGENTS generated file patterns
    let filtered: Vec<&str> = lines
        .into_iter()
        .filter(|line| {
            !line.contains("AGENTS.md")
            && !line.contains("CLAUDE.md")
            && !line.contains(".cursorrules")
            && !line.contains("cAGENTS generated")
        })
        .collect();

    if filtered.len() == content.lines().count() {
        println!("{} {}", "ℹ️ ".bright_blue(), "No output patterns in .gitignore".bright_blue());
        return Ok(());
    }

    // Write back
    fs::write(&gitignore_path, filtered.join("\n") + "\n")?;

    println!("{} {}", "✅".bright_green(), "Removed output patterns from .gitignore".green().bold());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn test_ignore_outputs_creates_gitignore() {
        let tmp = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(tmp.path());

        ignore_outputs().unwrap();

        let content = fs::read_to_string(".gitignore").unwrap();
        assert!(content.contains("AGENTS.md"));
        assert!(content.contains("**/AGENTS.md"));
    }

    #[test]
    #[serial]
    fn test_ignore_outputs_appends() {
        let tmp = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(tmp.path());

        // Create existing .gitignore
        fs::write(".gitignore", "node_modules\n").unwrap();

        ignore_outputs().unwrap();

        let content = fs::read_to_string(".gitignore").unwrap();
        assert!(content.contains("node_modules"));
        assert!(content.contains("AGENTS.md"));
    }

    /// RAII guard to safely change directory for tests
    struct TestDirGuard {
        original: std::path::PathBuf,
    }

    impl TestDirGuard {
        fn new(path: &std::path::Path) -> Self {
            let original = env::current_dir().unwrap();
            env::set_current_dir(path).unwrap();
            Self { original }
        }
    }

    impl Drop for TestDirGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.original);
        }
    }
}
