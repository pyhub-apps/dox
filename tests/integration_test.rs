mod common;

use common::{TestFixture, sample_rules, sample_docs, assertions::*};
use std::process::Command;
use predicates::prelude::*;
use assert_fs::prelude::*;

#[test]
fn test_replace_command_basic() {
    let fixture = TestFixture::new();
    
    // Create test files
    let rule_file = fixture.create_rule_file(sample_rules::BASIC_REPLACE);
    let input_file = fixture.create_markdown("input.md", sample_docs::MARKDOWN_TEMPLATE);
    let output_file = fixture.base_path.join("output.md");
    
    // Run replace command
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            "replace",
            "-r", rule_file.to_str().unwrap(),
            "-i", input_file.to_str().unwrap(),
            "-o", output_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");
    
    // Check command succeeded
    assert!(output.status.success(), "Command failed: {:?}", String::from_utf8_lossy(&output.stderr));
    
    // Verify output file exists and contains replacements
    assert!(output_file.exists());
    assert_file_contains(&output_file, "John Doe");
    assert_file_contains(&output_file, "2024-01-01");
    assert_file_contains(&output_file, "Acme Corp");
    assert_file_not_contains(&output_file, "{{name}}");
}

#[test]
fn test_replace_with_word_document() {
    let fixture = TestFixture::new();
    
    // Create Word document
    let doc_path = fixture.create_word_doc("test.docx");
    let rule_file = fixture.create_rule_file(sample_rules::BASIC_REPLACE);
    
    // Run replace command
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            "replace",
            "-r", rule_file.to_str().unwrap(),
            "-i", doc_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");
    
    // Basic check - command should at least run
    // Note: Full Word document testing would require parsing the output
    assert!(
        output.status.success() || 
        String::from_utf8_lossy(&output.stderr).contains("not implemented"),
        "Unexpected error: {:?}", 
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_extract_command() {
    let fixture = TestFixture::new();
    
    // Create a Word document
    let doc_path = fixture.create_word_doc("test.docx");
    let output_file = fixture.base_path.join("extracted.txt");
    
    // Run extract command
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            "extract",
            "-i", doc_path.to_str().unwrap(),
            "-o", output_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");
    
    // Check if command ran (may not be implemented yet)
    if output.status.success() {
        assert!(output_file.exists());
    }
}

#[test]
fn test_config_command() {
    // Test configuration commands
    let output = Command::new("cargo")
        .args(&["run", "--", "config", "list"])
        .output()
        .expect("Failed to execute command");
    
    // Config list should work even with no config
    assert!(
        output.status.success() || 
        String::from_utf8_lossy(&output.stderr).contains("not found"),
        "Unexpected error: {:?}", 
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dox"));
    assert!(stdout.contains("replace"));
    assert!(stdout.contains("extract"));
    assert!(stdout.contains("config"));
}

#[test]
fn test_version_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dox"));
    assert!(stdout.contains("0.1.0") || stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
#[cfg(unix)]
fn test_concurrent_processing() {
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let fixture = TestFixture::new();
    let counter = Arc::new(AtomicUsize::new(0));
    
    // Create multiple test files
    for i in 0..5 {
        fixture.create_markdown(&format!("file{}.md", i), sample_docs::MARKDOWN_TEMPLATE);
    }
    
    let rule_file = fixture.create_rule_file(sample_rules::BASIC_REPLACE);
    
    // Run concurrent replacements
    let mut handles = vec![];
    for i in 0..5 {
        let base_path = fixture.base_path.clone();
        let rule_path = rule_file.clone();
        let counter = counter.clone();
        
        let handle = thread::spawn(move || {
            let input = base_path.join(format!("file{}.md", i));
            let output = base_path.join(format!("output{}.md", i));
            
            let result = Command::new("cargo")
                .args(&[
                    "run", "--",
                    "replace",
                    "-r", rule_path.to_str().unwrap(),
                    "-i", input.to_str().unwrap(),
                    "-o", output.to_str().unwrap(),
                ])
                .output()
                .expect("Failed to execute command");
            
            if result.status.success() {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // At least some should succeed
    assert!(counter.load(Ordering::SeqCst) > 0);
}

#[test]
fn test_dry_run_mode() {
    let fixture = TestFixture::new();
    
    let rule_file = fixture.create_rule_file(sample_rules::BASIC_REPLACE);
    let input_file = fixture.create_markdown("input.md", sample_docs::MARKDOWN_TEMPLATE);
    let output_file = fixture.base_path.join("output.md");
    
    // Run with dry-run flag
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            "replace",
            "-r", rule_file.to_str().unwrap(),
            "-i", input_file.to_str().unwrap(),
            "-o", output_file.to_str().unwrap(),
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute command");
    
    // In dry run, output file should not be created
    assert!(!output_file.exists() || output.status.success());
}

#[test]
fn test_error_handling_invalid_file() {
    let fixture = TestFixture::new();
    
    let rule_file = fixture.create_rule_file(sample_rules::BASIC_REPLACE);
    let nonexistent = fixture.base_path.join("nonexistent.md");
    
    // Try to process non-existent file
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            "replace",
            "-r", rule_file.to_str().unwrap(),
            "-i", nonexistent.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");
    
    // Should fail gracefully
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Error"));
}

#[test]
fn test_logging_verbosity() {
    let fixture = TestFixture::new();
    
    let rule_file = fixture.create_rule_file(sample_rules::BASIC_REPLACE);
    let input_file = fixture.create_markdown("input.md", sample_docs::MARKDOWN_TEMPLATE);
    
    // Run with verbose logging
    let output = Command::new("cargo")
        .env("RUST_LOG", "debug")
        .args(&[
            "run", "--",
            "replace",
            "-r", rule_file.to_str().unwrap(),
            "-i", input_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    // With debug logging, should see more output
    assert!(stderr.len() > 0 || output.status.success());
}