use super::{ReplaceOptions, ReplaceResults, Rule};
use crate::compat::Document;
use anyhow::Result;
use colored::*;
use dox_core::utils::ui;
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
        let files =
            super::find_document_files(path, options.recursive, options.exclude.as_deref())?;

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

        let progress = ui::create_progress_bar(files.len() as u64, "문서 처리 중");

        for (i, file) in files.iter().enumerate() {
            progress.set_message(format!("처리 중: {}", file.display()));

            match self.process_file(&file, &options).await {
                Ok(count) => {
                    results.files_processed += 1;
                    results.total_replacements += count;
                    if count > 0 {
                        info!("Processed {}: {} replacements", file.display(), count);
                    }
                }
                Err(e) => {
                    error!("Error processing {}: {}", file.display(), e);
                    results.errors += 1;
                }
            }

            progress.set_position((i + 1) as u64);
        }

        progress.finish_with_message("문서 처리 완료");
        Ok(results)
    }

    /// Process files concurrently
    async fn process_concurrent(
        &self,
        files: Vec<std::path::PathBuf>,
        options: ReplaceOptions,
    ) -> Result<ReplaceResults> {
        use futures::stream::{self, StreamExt};
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };

        let max_workers = options.max_workers.min(files.len());
        info!(
            "Processing {} files with {} workers",
            files.len(),
            max_workers
        );

        let progress = ui::create_progress_bar(
            files.len() as u64,
            &format!("병렬 처리 중 ({}개 작업자)", max_workers),
        );
        let completed = Arc::new(AtomicUsize::new(0));

        let results = stream::iter(files)
            .map(|file| {
                let rules = self.rules.clone();
                let opts = options.clone();
                let progress = progress.clone();
                let completed = Arc::clone(&completed);
                async move {
                    let result = {
                        let replacer = Replacer::new(rules);
                        replacer
                            .process_file(&file, &opts)
                            .await
                            .map(|count| (1, count, 0))
                            .unwrap_or_else(|e| {
                                error!("Error processing {}: {}", file.display(), e);
                                (0, 0, 1)
                            })
                    };

                    let current = completed.fetch_add(1, Ordering::SeqCst) + 1;
                    progress.set_position(current as u64);

                    result
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

        progress.finish_with_message("병렬 처리 완료");
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

        // Get original content for diff if requested
        let original_content = if options.show_diff {
            Some(doc.get_text()?)
        } else {
            None
        };

        // Apply replacements
        let mut total_replacements = 0;
        let mut applied_rules = Vec::new();

        for rule in &self.rules {
            let count = doc.replace_text(&rule.old, &rule.new)?;
            if count > 0 {
                debug!(
                    "Replaced {} occurrences of '{}' with '{}'",
                    count, rule.old, rule.new
                );
                total_replacements += count;
                applied_rules.push((rule.clone(), count));
            }
        }

        // Show diff if requested and changes were made
        if options.show_diff && total_replacements > 0 {
            if let Some(original) = original_content {
                let new_content = doc.get_text()?;
                println!(
                    "\n{}",
                    format!("Changes in {}", path.display()).cyan().bold()
                );
                println!("{}", "=".repeat(60).cyan());
                ui::print_diff(&original, &new_content, 3);
                println!(); // Add newline for better formatting
            }
        }

        // Save the document if not in dry-run mode
        if !options.dry_run && total_replacements > 0 {
            doc.save()?;
            info!("Saved changes to {}", path.display());
        } else if options.dry_run && total_replacements > 0 {
            println!(
                "  {} {}: {} replacements would be made",
                "→".cyan(),
                path.display(),
                total_replacements
            );
            for (rule, count) in applied_rules {
                println!(
                    "    {} '{}' → '{}' ({}회)",
                    "•".yellow(),
                    rule.old,
                    rule.new,
                    count
                );
            }
        }

        Ok(total_replacements)
    }

    /// Create a backup of the file
    fn create_backup(&self, path: &Path) -> Result<()> {
        use std::time::SystemTime;

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let backup_path = if let Some(parent) = path.parent() {
            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
            let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let backup_name = if extension.is_empty() {
                format!("{}.backup.{}", file_stem, timestamp)
            } else {
                format!("{}.backup.{}.{}", file_stem, timestamp, extension)
            };
            parent.join(backup_name)
        } else {
            path.with_extension(format!(
                "{}.backup.{}",
                path.extension().and_then(|s| s.to_str()).unwrap_or(""),
                timestamp
            ))
        };

        std::fs::copy(path, &backup_path)?;
        info!("Created backup: {}", backup_path.display());

        Ok(())
    }
}
