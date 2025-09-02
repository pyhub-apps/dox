use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod replacer;
pub use replacer::Replacer;

#[cfg(test)]
mod tests;

/// A single replacement rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub old: String,
    pub new: String,
}

impl Rule {
    /// Create a new replacement rule
    pub fn new(old: impl Into<String>, new: impl Into<String>) -> Self {
        Rule {
            old: old.into(),
            new: new.into(),
        }
    }
    
    /// Validate the rule
    pub fn validate(&self) -> Result<()> {
        if self.old.is_empty() {
            anyhow::bail!("Replacement rule 'old' value cannot be empty");
        }
        if self.old == self.new {
            anyhow::bail!("Replacement rule 'old' and 'new' values cannot be the same");
        }
        Ok(())
    }
}

/// Options for replacement operations
#[derive(Debug, Clone)]
pub struct ReplaceOptions {
    pub dry_run: bool,
    pub backup: bool,
    pub recursive: bool,
    pub exclude: Option<String>,
    pub concurrent: bool,
    pub max_workers: usize,
    pub show_diff: bool,
}

impl Default for ReplaceOptions {
    fn default() -> Self {
        ReplaceOptions {
            dry_run: false,
            backup: false,
            recursive: true,
            exclude: None,
            concurrent: false,
            max_workers: 4,
            show_diff: false,
        }
    }
}

/// Results from a replacement operation
#[derive(Debug, Default)]
pub struct ReplaceResults {
    pub files_processed: usize,
    pub total_replacements: usize,
    pub errors: usize,
    pub skipped: usize,
}

/// Load replacement rules from a YAML file
pub fn load_rules(path: &Path) -> Result<Vec<Rule>> {
    use std::fs;
    
    if !path.exists() {
        anyhow::bail!("Rules file not found: {}", path.display());
    }
    
    let content = fs::read_to_string(path)?;
    let rules: Vec<Rule> = serde_yaml::from_str(&content)?;
    
    // Validate all rules
    for (i, rule) in rules.iter().enumerate() {
        rule.validate()
            .map_err(|e| anyhow::anyhow!("Invalid rule at index {}: {}", i, e))?;
    }
    
    Ok(rules)
}

/// Find all document files in a directory
pub fn find_document_files(
    path: &Path,
    recursive: bool,
    exclude: Option<&str>,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    if path.is_file() {
        if is_supported_document(path) {
            files.push(path.to_path_buf());
        }
        return Ok(files);
    }
    
    let walker = if recursive {
        WalkDir::new(path)
    } else {
        WalkDir::new(path).max_depth(1)
    };
    
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        // Skip if matches exclude pattern
        if let Some(pattern) = exclude {
            if let Some(file_name) = path.file_name() {
                if glob::Pattern::new(pattern)?.matches(file_name.to_str().unwrap_or("")) {
                    continue;
                }
            }
        }
        
        if path.is_file() && is_supported_document(path) {
            files.push(path.to_path_buf());
        }
    }
    
    Ok(files)
}

/// Check if a file is a supported document type
pub fn is_supported_document(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some("docx") | Some("pptx") => true,
        _ => false,
    }
}