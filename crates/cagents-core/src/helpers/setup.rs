// Package manager setup helpers

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::fs;
use std::path::PathBuf;

/// Add cagents to package.json scripts
pub fn setup_pnpm() -> Result<()> {
    let package_json_path = PathBuf::from("package.json");

    if !package_json_path.exists() {
        anyhow::bail!("package.json not found. Is this a Node.js project?");
    }

    // Read package.json
    let content = fs::read_to_string(&package_json_path)?;
    let mut package: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse package.json")?;

    // Get scripts object or create it
    let scripts = package
        .get_mut("scripts")
        .and_then(|s| s.as_object_mut())
        .ok_or_else(|| anyhow::anyhow!("package.json missing scripts field"))?;

    // Check if postinstall already exists
    if scripts.contains_key("postinstall") {
        println!("{} {}", "▸ ".yellow(), "package.json already has postinstall script".yellow());
        println!("   Current: {}", scripts["postinstall"]);
        println!();

        if crate::interactive::is_interactive() {
            use inquire::Confirm;
            let overwrite = Confirm::new("Overwrite postinstall script?")
                .with_default(false)
                .prompt()?;

            if !overwrite {
                println!("Cancelled.");
                return Ok(());
            }
        } else {
            return Ok(());
        }
    }

    // Add scripts
    scripts.insert("postinstall".to_string(), serde_json::json!("cagents build || true"));
    scripts.insert("rules:build".to_string(), serde_json::json!("cagents build"));
    scripts.insert("rules:lint".to_string(), serde_json::json!("cagents lint"));

    // Write back
    let pretty = serde_json::to_string_pretty(&package)?;
    fs::write(&package_json_path, pretty)?;

    println!("{} {}", "✅".bright_green(), "Updated package.json".green().bold());
    println!();
    println!("  {} {}", "+".green(), "postinstall: cagents build || true".bright_white());
    println!("  {} {}", "+".green(), "rules:build: cagents build".bright_white());
    println!("  {} {}", "+".green(), "rules:lint: cagents lint".bright_white());
    println!();
    println!("{} {}", "→".bright_blue(), "AGENTS.md will regenerate on every pnpm install!".bright_blue());

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
    fn test_setup_pnpm_adds_scripts() {
        let tmp = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(tmp.path());

        // Create package.json
        let package = r#"{"name": "test", "scripts": {}}"#;
        fs::write("package.json", package).unwrap();

        setup_pnpm().unwrap();

        let content = fs::read_to_string("package.json").unwrap();
        assert!(content.contains("postinstall"));
        assert!(content.contains("rules:build"));
        assert!(content.contains("rules:lint"));
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
