use super::{Rule, ReplaceOptions, ReplaceResults};
use crate::compat::Document;
use anyhow::Result;
use std::path::Path;
use tracing::{debug, error, info, warn};

/// Handles text replacement in documents
pub struct Replacer {
    rules: Vec<Rule>,
}

impl Replacer {
    /// Create a new replacer with the given rules
    pub fn new(rules: Vec<Rule>) -> Self {
        Replacer { rules }
    }
    
    /// Process a file or directory with the replacement rules
    pub async fn process_path(
        &self,
        path: &Path,
        options: ReplaceOptions,
    ) -> Result<ReplaceResults> {
        let files = super::find_document_files(path, options.recursive, options.exclude.as_deref())?;
        
        if files.is_empty() {
            warn!("No supported documents found in {}", path.display());
            return Ok(ReplaceResults::default());
        }
        
        info!("Found {} document(s) to process", files.len());
        
        if options.concurrent {
            self.process_concurrent(files, options).await
        } else {
            self.process_sequential(files, options).await
        }
    }
    
    /// Process files sequentially
    async fn process_sequential(
        &self,
        files: Vec<std::path::PathBuf>,
        options: ReplaceOptions,
    ) -> Result<ReplaceResults> {
        let mut results = ReplaceResults::default();
        
        for file in files {
            match self.process_file(&file, &options).await {
                Ok(count) => {
                    results.files_processed += 1;
                    results.total_replacements += count;
                    info!("Processed {}: {} replacements", file.display(), count);
                }
                Err(e) => {
                    error!("Error processing {}: {}", file.display(), e);
                    results.errors += 1;
                }
            }
        }
        
        Ok(results)
    }
    
    /// Process files concurrently
    async fn process_concurrent(
        &self,
        files: Vec<std::path::PathBuf>,
        options: ReplaceOptions,
    ) -> Result<ReplaceResults> {
        use futures::stream::{self, StreamExt};
        
        let max_workers = options.max_workers.min(files.len());
        info!("Processing {} files with {} workers", files.len(), max_workers);
        
        let results = stream::iter(files)
            .map(|file| {
                let rules = self.rules.clone();
                let opts = options.clone();
                async move {
                    let replacer = Replacer::new(rules);
                    replacer.process_file(&file, &opts).await
                        .map(|count| (1, count, 0))
                        .unwrap_or_else(|e| {
                            error!("Error processing {}: {}", file.display(), e);
                            (0, 0, 1)
                        })
                }
            })
            .buffer_unordered(max_workers)
            .fold(
                ReplaceResults::default(),
                |mut acc, (processed, replacements, errors)| async move {
                    acc.files_processed += processed;
                    acc.total_replacements += replacements;
                    acc.errors += errors;
                    acc
                },
            )
            .await;
        
        Ok(results)
    }
    
    /// Process a single file
    async fn process_file(&self, path: &Path, options: &ReplaceOptions) -> Result<usize> {
        debug!("Processing file: {}", path.display());
        
        // Create backup if requested
        if options.backup && !options.dry_run {
            self.create_backup(path)?;
        }
        
        // Open the document
        let mut doc = Document::open(path)?;
        
        // Apply replacements
        let mut total_replacements = 0;
        for rule in &self.rules {
            let count = doc.replace_text(&rule.old, &rule.new)?;
            if count > 0 {
                debug!("Replaced {} occurrences of '{}' with '{}'", count, rule.old, rule.new);
                total_replacements += count;
            }
        }
        
        // Save the document if not in dry-run mode
        if !options.dry_run && total_replacements > 0 {
            doc.save()?;
            info!("Saved changes to {}", path.display());
        }
        
        Ok(total_replacements)
    }
    
    /// Create a backup of the file
    fn create_backup(&self, path: &Path) -> Result<()> {
        let backup_path = path.with_extension(format!(
            "{}.backup",
            path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
        ));
        
        std::fs::copy(path, &backup_path)?;
        debug!("Created backup: {}", backup_path.display());
        
        Ok(())
    }
}