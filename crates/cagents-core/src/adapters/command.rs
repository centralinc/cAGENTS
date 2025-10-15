// Execute external compiler via JSON stdin/stdout protocol
// IN: { templateSource, templatePath, data, frontmatter, cwd }
// OUT: { content, diagnostics? }

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Serialize)]
struct CompilerInput<'a> {
    #[serde(rename = "templateSource")]
    template_source: &'a str,
    #[serde(rename = "templatePath")]
    template_path: String,
    data: &'a Value,
    frontmatter: &'a Value,
    cwd: String,
}

#[derive(Deserialize)]
struct CompilerOutput {
    content: String,
    diagnostics: Option<Vec<String>>,
}

/// Render template using external command
pub fn render_external(
    command: &str,
    source: &str,
    data: &Value,
    frontmatter: &Value,
    template_path: &str,
) -> Result<String> {
    // Prepare input
    let input = CompilerInput {
        template_source: source,
        template_path: template_path.to_string(),
        data,
        frontmatter,
        cwd: std::env::current_dir()?
            .to_string_lossy()
            .to_string(),
    };

    let input_json = serde_json::to_string(&input)?;

    // Execute command
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn external compiler: {}", command))?;

    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input_json.as_bytes())?;
        stdin.flush()?;
        // Explicitly drop stdin to close the pipe before waiting
        drop(stdin);
    }

    // Wait for output
    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("External compiler failed: {}", stderr);
    }

    // Parse output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: CompilerOutput = serde_json::from_str(&stdout)
        .with_context(|| format!("Failed to parse compiler output: {}", stdout))?;

    // Show diagnostics if any
    if let Some(diagnostics) = result.diagnostics {
        for diag in diagnostics {
            eprintln!("Compiler diagnostic: {}", diag);
        }
    }

    Ok(result.content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_render_external_echo() {
        // Test using a command that reads from stdin and transforms it
        // This simulates a real external compiler that processes the input
        let command = r#"cat > /dev/null && printf '{"content": "Hello from external"}'"#;
        let data = json!({});
        let frontmatter = json!({});

        let result = render_external(command, "source", &data, &frontmatter, "test.md").unwrap();
        assert_eq!(result, "Hello from external");
    }
}

