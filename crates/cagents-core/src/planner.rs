// evaluate globs, when/env filters, and ordering to plan outputs

use crate::loader::Rule;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use globset::{Glob, GlobSetBuilder};

/// Build context from CLI args
#[derive(Debug, Clone)]
pub struct BuildContext {
    // Legacy fields for backward compatibility
    pub env: Option<String>,
    pub role: Option<String>,
    pub language: Option<String>,
    pub target: Option<String>,

    // Arbitrary variables
    pub variables: HashMap<String, String>,
}

impl BuildContext {
    pub fn new(env: Option<String>, role: Option<String>, language: Option<String>) -> Self {
        let mut variables = HashMap::new();

        // Store legacy values in variables map for unified access
        if let Some(env_val) = &env {
            variables.insert("env".to_string(), env_val.clone());
        }
        if let Some(role_val) = &role {
            variables.insert("role".to_string(), role_val.clone());
        }
        if let Some(language_val) = &language {
            variables.insert("language".to_string(), language_val.clone());
        }

        Self {
            env,
            role,
            language,
            target: None,
            variables,
        }
    }

    /// Create context with specific target
    pub fn with_target(env: Option<String>, role: Option<String>, language: Option<String>, target: String) -> Self {
        let mut ctx = Self::new(env, role, language);
        ctx.target = Some(target.clone());
        ctx.variables.insert("target".to_string(), target);
        ctx
    }

    /// Create context from arbitrary variables
    pub fn from_variables(vars: HashMap<String, String>) -> Self {
        let env = vars.get("env").cloned();
        let role = vars.get("role").cloned();
        let language = vars.get("language").cloned();
        let target = vars.get("target").cloned();

        Self {
            env,
            role,
            language,
            target,
            variables: vars,
        }
    }

    /// Check if a rule's when clause matches this context
    pub fn matches_when(&self, when: &Option<crate::model::When>) -> bool {
        let Some(when) = when else {
            return true; // No when clause = always matches
        };

        // Get all variable requirements from when clause (includes legacy fields + arbitrary vars)
        let when_vars = when.all_variables();

        // Check each variable requirement
        for (var_name, allowed_values) in when_vars {
            // Get the context value for this variable
            let ctx_value = self.variables.get(&var_name);

            if let Some(ctx_val) = ctx_value {
                // Context has this variable - check if value is in allowed list
                if !allowed_values.contains(ctx_val) {
                    return false;
                }
            } else {
                // Context doesn't have this variable, but rule requires it
                return false;
            }
        }

        true
    }
}

/// Filter rules that apply to the root output
/// M1 Slice 3: Added context filtering via when clauses
/// - A rule applies if globs is missing/empty (no file scoping)
/// - When clause filtering is done later per-target to support when.target
pub fn filter_rules_for_root(rules: &[Rule], _context: &BuildContext) -> Result<Vec<Rule>> {
    let filtered: Vec<Rule> = rules
        .iter()
        .filter(|rule| {
            // Include rules with no globs or empty globs
            // These apply to root regardless of when clause
            // When clause filtering happens per-target in build
            if let Some(globs) = &rule.frontmatter.globs {
                globs.is_empty()
            } else {
                true // No globs = applies to root
            }
        })
        .cloned()
        .collect();

    Ok(filtered)
}

/// Filter rules that apply to a specific file
/// A rule applies if:
/// - Context matches (no when clause = always matches context)
/// - AND either: no globs/empty globs OR file matches glob patterns
pub fn filter_rules_for_file(
    rules: &[Rule],
    file_path: &Path,
    context: &BuildContext,
) -> Result<Vec<Rule>> {
    let filtered: Vec<Rule> = rules
        .iter()
        .filter(|rule| {
            // First check if context matches
            // No when clause = always matches context
            if !context.matches_when(&rule.frontmatter.when) {
                return false;
            }

            // Now check file/glob matching
            // If no globs or empty globs, apply to all files
            let Some(globs) = &rule.frontmatter.globs else {
                return true; // No globs = applies to all files
            };

            if globs.is_empty() {
                return true; // Empty globs = applies to all files
            }

            // Has globs - check if file matches
            let mut builder = GlobSetBuilder::new();
            for pattern in globs {
                if let Ok(glob) = Glob::new(pattern) {
                    builder.add(glob);
                }
            }

            if let Ok(globset) = builder.build() {
                return globset.is_match(file_path);
            }

            false
        })
        .cloned()
        .collect();

    Ok(filtered)
}

/// Plan outputs by grouping rules into target directories
/// M1 Slice 5: Nested directory outputs
///
/// Returns a HashMap where:
/// - Key: target directory path (e.g., ".", "src", "tests")
/// - Value: Vec of rules that apply to files in that directory
pub fn plan_outputs(
    rules: &[Rule],
    context: &BuildContext,
    project_root: &Path,
) -> Result<HashMap<PathBuf, Vec<Rule>>> {
    let mut outputs: HashMap<PathBuf, Vec<Rule>> = HashMap::new();

    // First, collect all rules that apply to root (no globs or alwaysApply)
    let root_rules = filter_rules_for_root(rules, context)?;
    if !root_rules.is_empty() {
        outputs.insert(PathBuf::from("."), root_rules);
    }

    // Then, for each rule with globs, find which directories it applies to
    for rule in rules {
        // Skip if no globs (already handled in root)
        let Some(globs) = &rule.frontmatter.globs else {
            continue;
        };
        if globs.is_empty() {
            continue;
        }

        // Note: when clause filtering (including target) is done later per-target output

        // Check if we should simplify to common parent
        let simplify = rule.frontmatter.simplify_globs_to_parent.unwrap_or(true);

        let dirs = if simplify {
            // Find common parent directory
            find_common_parent_directory(globs)
        } else {
            // Find all directories that match the globs
            find_matching_directories(project_root, globs)?
        };

        for dir in dirs {
            outputs
                .entry(dir)
                .or_default()
                .push(rule.clone());
        }
    }

    Ok(outputs)
}

/// Find common parent directory from glob patterns
/// Used when simplifyGlobsToParent is true
fn find_common_parent_directory(globs: &[String]) -> HashSet<PathBuf> {
    if globs.is_empty() {
        return HashSet::new();
    }

    // Extract directory parts from each glob (before wildcards)
    let dir_parts: Vec<Vec<String>> = globs
        .iter()
        .map(|g| {
            // Remove wildcard parts and extract concrete directory path
            let cleaned = g
                .split('/')
                .take_while(|part| !part.contains('*'))
                .filter(|part| !part.is_empty())
                .map(|s| s.to_string())
                .collect();
            cleaned
        })
        .collect();

    // If any pattern starts with wildcards, use root
    if dir_parts.iter().any(|parts| parts.is_empty()) {
        return vec![PathBuf::from(".")].into_iter().collect();
    }

    // Find common prefix across all directory parts
    let mut common_prefix = Vec::new();
    let first = &dir_parts[0];

    for (i, part) in first.iter().enumerate() {
        if dir_parts.iter().all(|p| p.get(i).map(|s| s.as_str()) == Some(part.as_str())) {
            common_prefix.push(part.clone());
        } else {
            break;
        }
    }

    if common_prefix.is_empty() {
        vec![PathBuf::from(".")].into_iter().collect()
    } else {
        vec![PathBuf::from(common_prefix.join("/"))].into_iter().collect()
    }
}

/// Find all directories that contain files matching the given glob patterns
fn find_matching_directories(project_root: &Path, globs: &[String]) -> Result<HashSet<PathBuf>> {
    use walkdir::WalkDir;

    let mut builder = GlobSetBuilder::new();
    for pattern in globs {
        let glob = Glob::new(pattern)?;
        builder.add(glob);
    }
    let globset = builder.build()?;

    let mut directories = HashSet::new();

    // Walk the project directory
    for entry in WalkDir::new(project_root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Don't filter the root directory itself
            if e.path() == project_root {
                return true;
            }

            // Skip hidden directories and common ignore patterns
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.')
                && name != "node_modules"
                && name != "target"
                && name != "dist"
        })
    {
        let entry = entry?;

        if !entry.file_type().is_file() {
            continue;
        }

        // Get path relative to project root
        let rel_path = entry.path().strip_prefix(project_root)
            .unwrap_or(entry.path());

        // Check if this file matches any glob
        if globset.is_match(rel_path) {
            // Add the parent directory
            if let Some(parent) = rel_path.parent() {
                directories.insert(parent.to_path_buf());
            } else {
                // File is in the root directory
                directories.insert(PathBuf::from("."));
            }
        }
    }

    Ok(directories)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::RuleFrontmatter;
    use std::path::PathBuf;

    #[test]
    fn test_filter_root_excludes_rules_with_globs() {
        let ctx = BuildContext::new(None, None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                when: None,
                globs: Some(vec!["**/*.ts".to_string()]),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let filtered = filter_rules_for_root(&[rule], &ctx).unwrap();
        assert_eq!(filtered.len(), 0, "Rules with globs should not apply to root");
    }

    #[test]
    fn test_filter_no_globs() {
        let ctx = BuildContext::new(None, None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                when: None,
                globs: None,
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let filtered = filter_rules_for_root(&[rule], &ctx).unwrap();
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_with_globs_and_when_clause() {
        let ctx = BuildContext::new(None, None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                when: Some(crate::model::When::legacy(
                    Some(vec!["prod".to_string()]),
                    None,
                    None,
                    None,
                )),
                globs: Some(vec!["**/*.ts".to_string()]),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let filtered = filter_rules_for_root(&[rule], &ctx).unwrap();
        assert_eq!(filtered.len(), 0, "Rules with globs and when clause should not apply to root");
    }

    #[test]
    fn test_filter_root_includes_rules_with_when_but_no_globs() {
        let ctx = BuildContext::new(Some("cloud".to_string()), None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                when: Some(crate::model::When::legacy(
                    Some(vec!["cloud".to_string(), "prod".to_string()]),
                    None,
                    None,
                    None,
                )),
                globs: None,
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let filtered = filter_rules_for_root(&[rule], &ctx).unwrap();
        // filter_rules_for_root includes rules without globs (when filtering deferred to build)
        assert_eq!(filtered.len(), 1, "Should include rule without globs");
    }

    #[test]
    fn test_filter_env_no_match() {
        // This test now verifies matches_when directly since filter_rules_for_root
        // no longer does when clause filtering
        let ctx = BuildContext::new(Some("local".to_string()), None, None);
        let when_clause = Some(crate::model::When::legacy(
            Some(vec!["cloud".to_string(), "prod".to_string()]),
            None,
            None,
            None,
        ));

        assert!(!ctx.matches_when(&when_clause), "Should not match when env not in list");
    }

    #[test]
    fn test_filter_root_with_role_when_clause_no_globs() {
        let ctx = BuildContext::new(None, Some("backend".to_string()), None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                when: Some(crate::model::When::legacy(
                    None,
                    Some(vec!["backend".to_string(), "fullstack".to_string()]),
                    None,
                    None,
                )),
                globs: None,
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let filtered = filter_rules_for_root(&[rule], &ctx).unwrap();
        assert_eq!(filtered.len(), 1, "Should include rule with when clause but no globs");
    }

    #[test]
    fn test_filter_root_with_language_when_clause_no_globs() {
        let ctx = BuildContext::new(None, None, Some("rust".to_string()));
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                when: Some(crate::model::When::legacy(
                    None,
                    None,
                    Some(vec!["rust".to_string(), "typescript".to_string()]),
                    None,
                )),
                globs: None,
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let filtered = filter_rules_for_root(&[rule], &ctx).unwrap();
        assert_eq!(filtered.len(), 1, "Should include rule with when clause but no globs");
    }

    #[test]
    fn test_filter_root_with_multiple_when_no_globs() {
        let ctx = BuildContext::new(
            Some("cloud".to_string()),
            Some("backend".to_string()),
            Some("rust".to_string()),
        );
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                when: Some(crate::model::When::legacy(
                    Some(vec!["cloud".to_string()]),
                    Some(vec!["backend".to_string()]),
                    Some(vec!["rust".to_string()]),
                    None,
                )),
                globs: None,
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let filtered = filter_rules_for_root(&[rule], &ctx).unwrap();
        // filter_rules_for_root includes rules without globs (when filtering deferred to build)
        assert_eq!(filtered.len(), 1, "Should include rule without globs");
    }

    #[test]
    fn test_filter_multiple_when_partial_match() {
        // This test now verifies matches_when directly
        let ctx = BuildContext::new(
            Some("cloud".to_string()),
            Some("frontend".to_string()),
            Some("rust".to_string()),
        );
        let when_clause = Some(crate::model::When::legacy(
            Some(vec!["cloud".to_string()]),
            Some(vec!["backend".to_string()]),
            Some(vec!["rust".to_string()]),
            None,
        ));

        assert!(!ctx.matches_when(&when_clause), "Should not match when any when condition fails");
    }

    #[test]
    fn test_filter_rules_for_file_with_matching_glob() {
        let ctx = BuildContext::new(None, None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                when: None,
                globs: Some(vec!["**/*.rs".to_string()]),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let file_path = PathBuf::from("src/main.rs");
        let filtered = filter_rules_for_file(&[rule], &file_path, &ctx).unwrap();
        assert_eq!(filtered.len(), 1, "Should match file when glob pattern matches");
    }

    #[test]
    fn test_filter_rules_for_file_with_non_matching_glob() {
        let ctx = BuildContext::new(None, None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                when: None,
                globs: Some(vec!["**/*.ts".to_string()]),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let file_path = PathBuf::from("src/main.rs");
        let filtered = filter_rules_for_file(&[rule], &file_path, &ctx).unwrap();
        assert_eq!(filtered.len(), 0, "Should not match when glob pattern doesn't match");
    }

    #[test]
    fn test_filter_rules_for_file_without_when_or_globs() {
        let ctx = BuildContext::new(None, None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                when: None,
                globs: None,
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let file_path = PathBuf::from("src/main.rs");
        let filtered = filter_rules_for_file(&[rule], &file_path, &ctx).unwrap();
        assert_eq!(filtered.len(), 1, "Should match any file when no when clause and no globs");
    }

    #[test]
    fn test_filter_rules_for_file_respects_when_clause() {
        let ctx = BuildContext::new(Some("prod".to_string()), None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                globs: Some(vec!["**/*.rs".to_string()]),
                when: Some(crate::model::When::legacy(
                    Some(vec!["dev".to_string()]),
                    None,
                    None,
                    None,
                )),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let file_path = PathBuf::from("src/main.rs");
        let filtered = filter_rules_for_file(&[rule], &file_path, &ctx).unwrap();
        assert_eq!(filtered.len(), 0, "Should not match when when clause doesn't match context");
    }

    #[test]
    fn test_filter_with_arbitrary_variable_in_when_clause() {
        use std::collections::HashMap;

        // Create context with arbitrary variables (using the new variable map approach)
        let mut variables = HashMap::new();
        variables.insert("app_env".to_string(), "production".to_string());
        variables.insert("region".to_string(), "us-west".to_string());

        let ctx = BuildContext::from_variables(variables);

        // Rule that requires app_env=production
        let mut when_vars = HashMap::new();
        when_vars.insert("app_env".to_string(), vec!["production".to_string(), "staging".to_string()]);

        let rule = Rule {
            frontmatter: RuleFrontmatter {
                globs: Some(vec!["**/*.rs".to_string()]),
                when: Some(crate::model::When::from_variables(when_vars)),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let file_path = PathBuf::from("src/main.rs");
        let filtered = filter_rules_for_file(&[rule], &file_path, &ctx).unwrap();
        assert_eq!(filtered.len(), 1, "Should match when app_env variable matches and file matches glob");
    }

    #[test]
    fn test_filter_with_arbitrary_variable_no_match() {
        use std::collections::HashMap;

        // Create context with arbitrary variables
        let mut variables = HashMap::new();
        variables.insert("app_env".to_string(), "development".to_string());

        let ctx = BuildContext::from_variables(variables);

        // Rule that requires app_env=production or staging
        let mut when_vars = HashMap::new();
        when_vars.insert("app_env".to_string(), vec!["production".to_string(), "staging".to_string()]);

        let rule = Rule {
            frontmatter: RuleFrontmatter {
                globs: Some(vec!["**/*.rs".to_string()]),
                when: Some(crate::model::When::from_variables(when_vars)),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let file_path = PathBuf::from("src/main.rs");
        let filtered = filter_rules_for_file(&[rule], &file_path, &ctx).unwrap();
        assert_eq!(filtered.len(), 0, "Should not match when app_env variable doesn't match");
    }
}
