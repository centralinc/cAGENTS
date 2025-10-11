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
#[serde(deny_unknown_fields)]
pub struct When {
    pub env: Option<Vec<String>>,
    pub role: Option<Vec<String>>,
    pub language: Option<Vec<String>>,
    pub target: Option<Vec<String>>,
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
