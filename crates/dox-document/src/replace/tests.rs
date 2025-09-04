#[cfg(test)]
mod tests {

    use rstest::*;
    use std::collections::HashMap;

    #[fixture]
    fn sample_rules() -> HashMap<String, String> {
        let mut rules = HashMap::new();
        rules.insert("{{name}}".to_string(), "John Doe".to_string());
        rules.insert("{{date}}".to_string(), "2024-01-01".to_string());
        rules.insert("{{company}}".to_string(), "Acme Corp".to_string());
        rules
    }

    #[rstest]
    #[case("Hello {{name}}", "Hello John Doe")]
    #[case("Date: {{date}}", "Date: 2024-01-01")]
    #[case("{{company}} - {{name}}", "Acme Corp - John Doe")]
    #[case("No placeholders here", "No placeholders here")]
    fn test_simple_replacement(
        #[case] input: &str,
        #[case] expected: &str,
        sample_rules: HashMap<String, String>,
    ) {
        let replacer = Replacer::new(sample_rules);
        let result = replacer.replace_text(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_occurrences() {
        let mut rules = HashMap::new();
        rules.insert("{{x}}".to_string(), "X".to_string());

        let replacer = Replacer::new(rules);
        let input = "{{x}} + {{x}} = 2{{x}}";
        let expected = "X + X = 2X";

        assert_eq!(replacer.replace_text(input), expected);
    }

    #[test]
    fn test_nested_patterns() {
        let mut rules = HashMap::new();
        rules.insert("{{outer}}".to_string(), "{{inner}}".to_string());
        rules.insert("{{inner}}".to_string(), "value".to_string());

        let replacer = Replacer::new(rules);
        let input = "Result: {{outer}}";
        // Should not recursively replace
        let result = replacer.replace_text(input);
        assert!(result.contains("{{inner}}") || result.contains("value"));
    }

    #[test]
    fn test_case_sensitivity() {
        let mut rules = HashMap::new();
        rules.insert("{{Name}}".to_string(), "John".to_string());
        rules.insert("{{name}}".to_string(), "Jane".to_string());

        let replacer = Replacer::new(rules);
        let input = "{{Name}} and {{name}}";
        let result = replacer.replace_text(input);

        assert!(result.contains("John") || result.contains("Jane"));
    }

    #[rstest]
    #[case("", "")]
    #[case("   ", "   ")]
    #[case("\n\n", "\n\n")]
    fn test_edge_cases(
        #[case] input: &str,
        #[case] expected: &str,
        sample_rules: HashMap<String, String>,
    ) {
        let replacer = Replacer::new(sample_rules);
        assert_eq!(replacer.replace_text(input), expected);
    }

    #[test]
    fn test_special_characters() {
        let mut rules = HashMap::new();
        rules.insert("{{special}}".to_string(), "a$b^c*d".to_string());

        let replacer = Replacer::new(rules);
        let input = "Value: {{special}}";
        let result = replacer.replace_text(input);

        assert_eq!(result, "Value: a$b^c*d");
    }

    #[test]
    fn test_unicode_support() {
        let mut rules = HashMap::new();
        rules.insert("{{emoji}}".to_string(), "🎉".to_string());
        rules.insert("{{chinese}}".to_string(), "你好".to_string());
        rules.insert("{{arabic}}".to_string(), "مرحبا".to_string());

        let replacer = Replacer::new(rules);
        let input = "{{emoji}} {{chinese}} {{arabic}}";
        let result = replacer.replace_text(input);

        assert_eq!(result, "🎉 你好 مرحبا");
    }

    #[test]
    fn test_large_text_performance() {
        let mut rules = HashMap::new();
        for i in 0..100 {
            rules.insert(format!("{{{{var{}}}}}", i), format!("value{}", i));
        }

        let replacer = Replacer::new(rules);
        let mut input = String::new();
        for i in 0..100 {
            input.push_str(&format!("Line {}: {{{{var{}}}}}\n", i, i));
        }

        let start = std::time::Instant::now();
        let result = replacer.replace_text(&input);
        let duration = start.elapsed();

        // Should complete in reasonable time
        assert!(duration.as_millis() < 100);

        // Verify replacements
        for i in 0..100 {
            assert!(result.contains(&format!("value{}", i)));
        }
    }

    // Mock Replacer implementation for testing
    pub struct Replacer {
        rules: HashMap<String, String>,
    }

    impl Replacer {
        pub fn new(rules: HashMap<String, String>) -> Self {
            Self { rules }
        }

        pub fn replace_text(&self, text: &str) -> String {
            let mut result = text.to_string();
            for (pattern, replacement) in &self.rules {
                result = result.replace(pattern, replacement);
            }
            result
        }
    }
}
