use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

/// Replace text in Word and PowerPoint documents based on rules
#[derive(Args, Debug)]
pub struct ReplaceArgs {
    /// YAML file containing replacement rules
    #[arg(short, long, value_name = "FILE")]
    pub rules: PathBuf,
    
    /// Target file or directory path
    #[arg(short, long, value_name = "PATH")]
    pub path: PathBuf,
    
    /// Preview changes without applying them
    #[arg(long)]
    pub dry_run: bool,
    
    /// Create backups before modifying files
    #[arg(long)]
    pub backup: bool,
    
    /// Process subdirectories recursively
    #[arg(long, default_value = "true")]
    pub recursive: bool,
    
    /// Glob pattern for files to exclude
    #[arg(long, value_name = "PATTERN")]
    pub exclude: Option<String>,
    
    /// Enable concurrent processing
    #[arg(long)]
    pub concurrent: bool,
    
    /// Maximum number of parallel workers
    #[arg(long, value_name = "N", default_value = "4")]
    pub max_workers: usize,
    
    /// Show differences for each change
    #[arg(long)]
    pub show_diff: bool,
}

pub async fn execute(args: ReplaceArgs) -> Result<()> {
    use crate::core::replace::Replacer;
    use crate::utils::ui;
    
    // Load replacement rules
    let rules = crate::core::replace::load_rules(&args.rules)?;
    
    if rules.is_empty() {
        ui::print_warning("No replacement rules found in the file");
        return Ok(());
    }
    
    // Display rules in dry-run mode
    if args.dry_run {
        ui::print_header("Replacement Rules to Apply");
        for (i, rule) in rules.iter().enumerate() {
            ui::print_step(
                i + 1,
                rules.len(),
                &format!("Replace '{}' with '{}'", rule.old, rule.new),
            );
        }
    }
    
    // Create replacer instance
    let replacer = Replacer::new(rules);
    
    // Process documents
    let options = crate::core::replace::ReplaceOptions {
        dry_run: args.dry_run,
        backup: args.backup,
        recursive: args.recursive,
        exclude: args.exclude,
        concurrent: args.concurrent,
        max_workers: args.max_workers,
        show_diff: args.show_diff,
    };
    
    let results = replacer.process_path(&args.path, options).await?;
    
    // Display summary
    ui::print_header("Summary");
    ui::print_success(&format!(
        "Processed {} files with {} replacements",
        results.files_processed, results.total_replacements
    ));
    
    if results.errors > 0 {
        ui::print_error(&format!("{} files had errors", results.errors));
    }
    
    Ok(())
}