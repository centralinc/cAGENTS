// Built-in simple template engine for basic {{variable}} replacement
// Strict: fails if variable is undefined

use anyhow::Result;
use regex::Regex;
use serde_json::Value;

/// Render template using built-in simple string interpolation
/// Replaces {{variable}} with values from data
/// STRICT: Returns error if variable is not found in data
pub fn render_simple(source: &str, data: &Value) -> Result<String> {
    let var_pattern = Regex::new(r"\{\{(\w+)\}\}").unwrap();
    let mut result = source.to_string();
    let mut undefined_vars = Vec::new();

    // Find all {{variable}} patterns
    for cap in var_pattern.captures_iter(source) {
        let var_name = &cap[1];
        let placeholder = &cap[0]; // Full {{var}} including braces

        // Look up variable in data
        if let Some(value) = data.get(var_name) {
            let replacement = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                _ => {
                    anyhow::bail!(
                        "Variable '{}' has unsupported type (expected string, number, or boolean)",
                        var_name
                    );
                }
            };
            result = result.replace(placeholder, &replacement);
        } else {
            // Variable not found - this is an error in strict mode
            undefined_vars.push(var_name.to_string());
        }
    }

    // If any variables were undefined, fail
    if !undefined_vars.is_empty() {
        anyhow::bail!(
            "Undefined variables in template: {}",
            undefined_vars.join(", ")
        );
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_string_replacement() {
        let template = "Hello {{name}}!";
        let data = json!({"name": "World"});
        let result = render_simple(template, &data).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_multiple_variables() {
        let template = "Project: {{project}}\nOwner: {{owner}}";
        let data = json!({"project": "cagents", "owner": "Jordan"});
        let result = render_simple(template, &data).unwrap();
        assert_eq!(result, "Project: cagents\nOwner: Jordan");
    }

    #[test]
    fn test_number_replacement() {
        let template = "Version: {{version}}";
        let data = json!({"version": 42});
        let result = render_simple(template, &data).unwrap();
        assert_eq!(result, "Version: 42");
    }

    #[test]
    fn test_boolean_replacement() {
        let template = "Active: {{active}}";
        let data = json!({"active": true});
        let result = render_simple(template, &data).unwrap();
        assert_eq!(result, "Active: true");
    }

    #[test]
    fn test_undefined_variable_fails() {
        let template = "Hello {{undefined}}!";
        let data = json!({"name": "World"});
        let result = render_simple(template, &data);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Undefined variables"));
        assert!(err_msg.contains("undefined"));
    }

    #[test]
    fn test_multiple_undefined_variables() {
        let template = "{{var1}} and {{var2}}";
        let data = json!({"other": "value"});
        let result = render_simple(template, &data);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("var1"));
        assert!(err_msg.contains("var2"));
    }

    #[test]
    fn test_same_variable_multiple_times() {
        let template = "{{name}} loves {{name}}!";
        let data = json!({"name": "Rust"});
        let result = render_simple(template, &data).unwrap();
        assert_eq!(result, "Rust loves Rust!");
    }

    #[test]
    fn test_no_variables() {
        let template = "Plain text with no variables";
        let data = json!({});
        let result = render_simple(template, &data).unwrap();
        assert_eq!(result, "Plain text with no variables");
    }
}
