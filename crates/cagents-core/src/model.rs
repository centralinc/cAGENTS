use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub project: Option<ProjectMeta>,
    pub paths: Paths,
    pub defaults: Option<Defaults>,
    pub variables: Option<Variables>,
    pub execution: Option<Execution>,
    pub output: Option<Output>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectMeta { pub name: Option<String> }

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Paths {
    #[serde(rename = "templatesDir")]
    pub templates_dir: String,
    #[serde(rename = "outputRoot")]
    pub output_root: String,
    #[serde(rename = "cursorRulesDir")]
    pub cursor_rules_dir: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
    pub engine: Option<String>,
    pub targets: Option<Vec<String>>,
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Variables {
    #[serde(rename = "static")]
    pub static_: Option<serde_json::Value>,
    pub env: Option<serde_json::Value>,
    pub command: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Execution {
    pub shell: Option<String>,
    #[serde(rename = "timeoutMs")]
    pub timeout_ms: Option<u64>,
    #[serde(rename = "allowCommands")]
    pub allow_commands: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub targets: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RuleFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub engine: Option<String>,
    pub globs: Option<Vec<String>>,
    #[serde(rename = "alwaysApply")]
    pub always_apply: Option<bool>,
    pub order: Option<i32>,
    pub when: Option<When>,
    pub vars: Option<serde_json::Value>,
    pub merge: Option<Merge>,
    pub links: Option<Vec<Link>>,
    pub targets: Option<Vec<String>>,
    pub extends: Option<Vec<String>>,
    #[serde(rename = "simplifyGlobsToParent")]
    pub simplify_globs_to_parent: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct When {
    // Legacy fields for backward compatibility
    pub env: Option<Vec<String>>,
    pub role: Option<Vec<String>>,
    pub language: Option<Vec<String>>,
    pub target: Option<Vec<String>>,

    // Arbitrary variables (all other fields)
    #[serde(flatten)]
    pub variables: std::collections::HashMap<String, serde_json::Value>,
}

impl When {
    /// Create a When clause from legacy fields only (for backward compatibility and tests)
    pub fn legacy(
        env: Option<Vec<String>>,
        role: Option<Vec<String>>,
        language: Option<Vec<String>>,
        target: Option<Vec<String>>,
    ) -> Self {
        Self {
            env,
            role,
            language,
            target,
            variables: std::collections::HashMap::new(),
        }
    }

    /// Create a When clause from arbitrary variables
    pub fn from_variables(vars: std::collections::HashMap<String, Vec<String>>) -> Self {
        let mut variables = std::collections::HashMap::new();
        for (key, values) in vars {
            variables.insert(key, serde_json::Value::Array(
                values.into_iter().map(serde_json::Value::String).collect()
            ));
        }

        Self {
            env: None,
            role: None,
            language: None,
            target: None,
            variables,
        }
    }

    /// Get all variable requirements as a unified view
    /// This merges legacy fields (env, role, language, target) with arbitrary variables
    pub fn all_variables(&self) -> std::collections::HashMap<String, Vec<String>> {
        let mut result = std::collections::HashMap::new();

        // Add legacy fields if present
        if let Some(env) = &self.env {
            result.insert("env".to_string(), env.clone());
        }
        if let Some(role) = &self.role {
            result.insert("role".to_string(), role.clone());
        }
        if let Some(language) = &self.language {
            result.insert("language".to_string(), language.clone());
        }
        if let Some(target) = &self.target {
            result.insert("target".to_string(), target.clone());
        }

        // Add arbitrary variables
        for (key, value) in &self.variables {
            // Skip legacy fields to avoid duplicates
            if key == "env" || key == "role" || key == "language" || key == "target" {
                continue;
            }

            // Convert JSON value to Vec<String>
            if let Some(arr) = value.as_array() {
                let strings: Vec<String> = arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                if !strings.is_empty() {
                    result.insert(key.clone(), strings);
                }
            } else if let Some(s) = value.as_str() {
                // Single string value
                result.insert(key.clone(), vec![s.to_string()]);
            }
        }

        result
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Merge {
    pub strategy: Option<String>,
    pub sections: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub path: String,
    pub title: Option<String>,
}
