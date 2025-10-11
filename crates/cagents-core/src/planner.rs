// evaluate globs, when/env filters, and ordering to plan outputs

use crate::loader::Rule;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use globset::{Glob, GlobSetBuilder};

/// Build context from CLI args
#[derive(Debug, Clone)]
pub struct BuildContext {
    pub env: Option<String>,
    pub role: Option<String>,
    pub language: Option<String>,
    pub target: Option<String>,
}

impl BuildContext {
    pub fn new(env: Option<String>, role: Option<String>, language: Option<String>) -> Self {
        Self { env, role, language, target: None }
    }

    /// Create context with specific target
    pub fn with_target(env: Option<String>, role: Option<String>, language: Option<String>, target: String) -> Self {
        Self { env, role, language, target: Some(target) }
    }

    /// Check if a rule's when clause matches this context
    pub fn matches_when(&self, when: &Option<crate::model::When>) -> bool {
        let Some(when) = when else {
            return true; // No when clause = always matches
        };

        // Check env
        if let Some(env_list) = &when.env {
            if let Some(ctx_env) = &self.env {
                if !env_list.contains(ctx_env) {
                    return false;
                }
            } else {
                // Context has no env, but rule requires one
                return false;
            }
        }

        // Check role
        if let Some(role_list) = &when.role {
            if let Some(ctx_role) = &self.role {
                if !role_list.contains(ctx_role) {
                    return false;
                }
            } else {
                // Context has no role, but rule requires one
                return false;
            }
        }

        // Check language
        if let Some(lang_list) = &when.language {
            if let Some(ctx_lang) = &self.language {
                if !lang_list.contains(ctx_lang) {
                    return false;
                }
            } else {
                // Context has no language, but rule requires one
                return false;
            }
        }

        // Check target
        if let Some(target_list) = &when.target {
            if let Some(ctx_target) = &self.target {
                if !target_list.contains(ctx_target) {
                    return false;
                }
            } else {
                // Context has no target, but rule requires one
                return false;
            }
        }

        true
    }
}

/// Filter rules that apply to the root output
/// M1 Slice 3: Added context filtering via when clauses
/// - A rule applies if alwaysApply is true OR globs is missing/empty
/// - When clause filtering is done later per-target to support when.target
pub fn filter_rules_for_root(rules: &[Rule], _context: &BuildContext) -> Result<Vec<Rule>> {
    let filtered: Vec<Rule> = rules
        .iter()
        .filter(|rule| {
            // Check glob/alwaysApply criteria only
            // When clause filtering happens per-target in build
            if rule.frontmatter.always_apply == Some(true) {
                true
            } else if let Some(globs) = &rule.frontmatter.globs {
                globs.is_empty()
            } else {
                true
            }
        })
        .cloned()
        .collect();

    Ok(filtered)
}

/// Filter rules that apply to a specific file
/// A rule applies if:
/// - alwaysApply is true, OR
/// - The file matches one of the rule's glob patterns
/// Additionally, rules are filtered by context (when clause)
pub fn filter_rules_for_file(
    rules: &[Rule],
    file_path: &Path,
    context: &BuildContext,
) -> Result<Vec<Rule>> {
    let filtered: Vec<Rule> = rules
        .iter()
        .filter(|rule| {
            // First check if context matches
            if !context.matches_when(&rule.frontmatter.when) {
                return false;
            }

            // Always include alwaysApply rules
            if rule.frontmatter.always_apply == Some(true) {
                return true;
            }

            // Check if file matches any glob pattern
            if let Some(globs) = &rule.frontmatter.globs {
                if globs.is_empty() {
                    return false;
                }

                // Build globset for this rule
                let mut builder = GlobSetBuilder::new();
                for pattern in globs {
                    if let Ok(glob) = Glob::new(pattern) {
                        builder.add(glob);
                    }
                }

                if let Ok(globset) = builder.build() {
                    return globset.is_match(file_path);
                }
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
    fn test_filter_always_apply() {
        let ctx = BuildContext::new(None, None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                always_apply: Some(true),
                globs: Some(vec!["**/*.ts".to_string()]),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let filtered = filter_rules_for_root(&[rule], &ctx).unwrap();
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_no_globs() {
        let ctx = BuildContext::new(None, None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                always_apply: None,
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
    fn test_filter_with_globs() {
        let ctx = BuildContext::new(None, None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                always_apply: None,
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
    fn test_filter_env_match() {
        let ctx = BuildContext::new(Some("cloud".to_string()), None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                always_apply: Some(true),
                when: Some(crate::model::When {
                    env: Some(vec!["cloud".to_string(), "prod".to_string()]),
                    role: None,
                    language: None,
                    target: None,
                }),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let filtered = filter_rules_for_root(&[rule], &ctx).unwrap();
        // filter_rules_for_root now only checks globs/alwaysApply
        assert_eq!(filtered.len(), 1, "Should include rule (when filtering deferred to build)");
    }

    #[test]
    fn test_filter_env_no_match() {
        // This test now verifies matches_when directly since filter_rules_for_root
        // no longer does when clause filtering
        let ctx = BuildContext::new(Some("local".to_string()), None, None);
        let when_clause = Some(crate::model::When {
            env: Some(vec!["cloud".to_string(), "prod".to_string()]),
            role: None,
            language: None,
            target: None,
        });

        assert!(!ctx.matches_when(&when_clause), "Should not match when env not in list");
    }

    #[test]
    fn test_filter_role_match() {
        let ctx = BuildContext::new(None, Some("backend".to_string()), None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                always_apply: Some(true),
                when: Some(crate::model::When {
                    env: None,
                    role: Some(vec!["backend".to_string(), "fullstack".to_string()]),
                    language: None,
                    target: None,
                }),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let filtered = filter_rules_for_root(&[rule], &ctx).unwrap();
        assert_eq!(filtered.len(), 1, "Should match when role is in list");
    }

    #[test]
    fn test_filter_language_match() {
        let ctx = BuildContext::new(None, None, Some("rust".to_string()));
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                always_apply: Some(true),
                when: Some(crate::model::When {
                    env: None,
                    role: None,
                    target: None,
                    language: Some(vec!["rust".to_string(), "typescript".to_string()]),
                }),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let filtered = filter_rules_for_root(&[rule], &ctx).unwrap();
        assert_eq!(filtered.len(), 1, "Should match when language is in list");
    }

    #[test]
    fn test_filter_multiple_when_all_match() {
        let ctx = BuildContext::new(
            Some("cloud".to_string()),
            Some("backend".to_string()),
            Some("rust".to_string()),
        );
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                always_apply: Some(true),
                when: Some(crate::model::When {
                    env: Some(vec!["cloud".to_string()]),
                    role: Some(vec!["backend".to_string()]),
                    target: None,
                    language: Some(vec!["rust".to_string()]),
                }),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let filtered = filter_rules_for_root(&[rule], &ctx).unwrap();
        // filter_rules_for_root now only checks globs/alwaysApply
        assert_eq!(filtered.len(), 1, "Should include rule (when filtering deferred to build)");
    }

    #[test]
    fn test_filter_multiple_when_partial_match() {
        // This test now verifies matches_when directly
        let ctx = BuildContext::new(
            Some("cloud".to_string()),
            Some("frontend".to_string()),
            Some("rust".to_string()),
        );
        let when_clause = Some(crate::model::When {
            env: Some(vec!["cloud".to_string()]),
            role: Some(vec!["backend".to_string()]),
            language: Some(vec!["rust".to_string()]),
            target: None,
        });

        assert!(!ctx.matches_when(&when_clause), "Should not match when any when condition fails");
    }

    #[test]
    fn test_filter_rules_for_file_with_matching_glob() {
        let ctx = BuildContext::new(None, None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                always_apply: None,
                globs: Some(vec!["**/*.rs".to_string()]),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let file_path = PathBuf::from("src/main.rs");
        let filtered = filter_rules_for_file(&[rule], &file_path, &ctx).unwrap();
        assert_eq!(filtered.len(), 1, "Should match file with glob pattern");
    }

    #[test]
    fn test_filter_rules_for_file_with_non_matching_glob() {
        let ctx = BuildContext::new(None, None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                always_apply: None,
                globs: Some(vec!["**/*.ts".to_string()]),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let file_path = PathBuf::from("src/main.rs");
        let filtered = filter_rules_for_file(&[rule], &file_path, &ctx).unwrap();
        assert_eq!(filtered.len(), 0, "Should not match file with non-matching glob");
    }

    #[test]
    fn test_filter_rules_for_file_with_always_apply() {
        let ctx = BuildContext::new(None, None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                always_apply: Some(true),
                globs: Some(vec!["**/*.ts".to_string()]),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let file_path = PathBuf::from("src/main.rs");
        let filtered = filter_rules_for_file(&[rule], &file_path, &ctx).unwrap();
        assert_eq!(filtered.len(), 1, "Should match any file when alwaysApply is true");
    }

    #[test]
    fn test_filter_rules_for_file_respects_when_clause() {
        let ctx = BuildContext::new(Some("prod".to_string()), None, None);
        let rule = Rule {
            frontmatter: RuleFrontmatter {
                always_apply: None,
                globs: Some(vec!["**/*.rs".to_string()]),
                when: Some(crate::model::When {
                    env: Some(vec!["dev".to_string()]),
                    role: None,
                    language: None,
                    target: None,
                }),
                ..Default::default()
            },
            body: "test".to_string(),
            path: PathBuf::from("test.md"),
        };

        let file_path = PathBuf::from("src/main.rs");
        let filtered = filter_rules_for_file(&[rule], &file_path, &ctx).unwrap();
        assert_eq!(filtered.len(), 0, "Should not match when when clause doesn't match context");
    }
}
