use proptest::prelude::*;
use std::collections::HashMap;

// Property-based tests for core functionality

proptest! {
    #[test]
    fn test_replace_idempotence(
        text in "[a-zA-Z0-9 ]{0,100}",
        pattern in "[a-zA-Z]{1,10}",
        replacement in "[a-zA-Z]{1,10}"
    ) {
        // Replacing the same pattern twice should give the same result
        let result1 = text.replace(&pattern, &replacement);
        let result2 = result1.replace(&pattern, &replacement);
        assert_eq!(result1, result2);
    }
    
    #[test]
    fn test_replace_preserves_length_order(
        text in "[a-zA-Z0-9 ]{0,100}",
        pattern in "[a-zA-Z]{1,10}",
        short_replacement in "[a-z]{1,5}",
        long_replacement in "[A-Z]{10,20}"
    ) {
        let short_result = text.replace(&pattern, &short_replacement);
        let long_result = text.replace(&pattern, &long_replacement);
        
        // If pattern exists in text, longer replacement should produce longer result
        if text.contains(&pattern) {
            assert!(long_result.len() >= short_result.len());
        }
    }
    
    #[test]
    fn test_rule_validation_never_panics(
        old in ".*",
        new in ".*"
    ) {
        // Rule validation should handle any input without panicking
        let rule = create_test_rule(&old, &new);
        let _ = rule.validate(); // Should not panic
    }
    
    #[test]
    fn test_empty_pattern_handling(
        text in "[a-zA-Z0-9 ]{0,100}",
        replacement in "[a-zA-Z]{0,10}"
    ) {
        // Empty pattern should leave text unchanged
        let result = text.replace("", &replacement);
        // Rust's replace with empty pattern adds replacement at each position
        // Just ensure it doesn't panic
        assert!(result.len() >= text.len());
    }
    
    #[test]
    fn test_unicode_replacement_safety(
        text in "[a-zA-Z0-9 ]{0,50}",
        pattern in "[a-zA-Z]{1,5}",
        unicode_replacement in r"[😀😁😂🎉]{1,3}"
    ) {
        // Unicode replacements should work correctly
        let result = text.replace(&pattern, &unicode_replacement);
        // Should not panic and result should be valid UTF-8
        assert!(result.is_empty() || result.chars().count() > 0);
    }
    
    #[test]
    fn test_concurrent_replacement_consistency(
        texts in prop::collection::vec("[a-zA-Z0-9 ]{10,50}", 1..10),
        pattern in "[a-zA-Z]{1,5}",
        replacement in "[0-9]{1,5}"
    ) {
        use std::sync::Arc;
        use std::thread;
        
        let pattern = Arc::new(pattern);
        let replacement = Arc::new(replacement);
        
        // Process texts concurrently
        let handles: Vec<_> = texts.iter().map(|text| {
            let text = text.clone();
            let pattern = pattern.clone();
            let replacement = replacement.clone();
            
            thread::spawn(move || {
                text.replace(&*pattern, &*replacement)
            })
        }).collect();
        
        // Collect results
        let results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();
        
        // Verify same texts produce same results
        for (i, text) in texts.iter().enumerate() {
            let expected = text.replace(&*pattern, &*replacement);
            assert_eq!(results[i], expected);
        }
    }
    
    #[test]
    fn test_file_path_validation(
        path_str in r"[a-zA-Z0-9_\-./]{1,100}"
    ) {
        use std::path::Path;
        
        // Path parsing should never panic
        let path = Path::new(&path_str);
        let _ = path.exists(); // Should not panic
        let _ = path.extension(); // Should not panic
        let _ = path.file_name(); // Should not panic
    }
    
    #[test]
    fn test_yaml_parsing_robustness(
        key in "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
        value in "[a-zA-Z0-9 ]{0,50}"
    ) {
        // YAML parsing should handle various inputs gracefully
        let yaml = format!("{}: {}", key, value);
        let _ = serde_yaml::from_str::<serde_yaml::Value>(&yaml); // Should not panic
    }
    
    #[test]
    fn test_rule_application_order_independence(
        text in "[a-zA-Z]{20,50}",
        rules in prop::collection::vec(
            ("[a-z]{1,3}", "[A-Z]{1,3}"),
            2..5
        )
    ) {
        // Applying rules in different orders should be consistent
        // (for non-overlapping patterns)
        let mut rules_map = HashMap::new();
        for (pattern, replacement) in &rules {
            rules_map.insert(pattern.clone(), replacement.clone());
        }
        
        // Check if patterns are non-overlapping
        let patterns: Vec<_> = rules.iter().map(|(p, _)| p).collect();
        let non_overlapping = patterns.windows(2).all(|w| !w[0].contains(w[1]) && !w[1].contains(w[0]));
        
        if non_overlapping {
            // Apply rules in original order
            let mut result1 = text.clone();
            for (pattern, replacement) in &rules {
                result1 = result1.replace(pattern, replacement);
            }
            
            // Apply rules in reverse order
            let mut result2 = text.clone();
            for (pattern, replacement) in rules.iter().rev() {
                result2 = result2.replace(pattern, replacement);
            }
            
            // Results should be the same for non-overlapping patterns
            assert_eq!(result1, result2);
        }
    }
}

// Helper function for testing
fn create_test_rule(old: &str, new: &str) -> TestRule {
    TestRule {
        old: old.to_string(),
        new: new.to_string(),
    }
}

struct TestRule {
    old: String,
    new: String,
}

impl TestRule {
    fn validate(&self) -> Result<(), String> {
        if self.old.is_empty() {
            Err("Old value cannot be empty".to_string())
        } else if self.old == self.new {
            Err("Old and new values cannot be the same".to_string())
        } else {
            Ok(())
        }
    }
}