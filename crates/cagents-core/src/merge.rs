// section-aware merge: append/prepend/replace; dedupe headings

use anyhow::Result;

/// Merge multiple rendered rule bodies into a single document
/// For this slice: simple concatenation with double newline separator
pub fn merge_rule_bodies(rendered: &[String]) -> Result<String> {
    if rendered.is_empty() {
        return Ok(String::new());
    }

    let merged = rendered.join("\n\n");
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_empty() {
        let result = merge_rule_bodies(&[]).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_merge_single() {
        let result = merge_rule_bodies(&["## Section 1\nContent".to_string()]).unwrap();
        assert_eq!(result, "## Section 1\nContent");
    }

    #[test]
    fn test_merge_multiple() {
        let sections = vec![
            "## Section 1\nContent 1".to_string(),
            "## Section 2\nContent 2".to_string(),
        ];
        let result = merge_rule_bodies(&sections).unwrap();
        assert!(result.contains("Section 1"));
        assert!(result.contains("Section 2"));
        assert!(result.contains("\n\n"));
    }
}
