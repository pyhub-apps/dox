use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod commands;
use commands::*;

/// Document automation and AI-powered content generation CLI
#[derive(Parser, Debug)]
#[command(
    name = "dox",
    version,
    author,
    about,
    long_about = None,
    arg_required_else_help = true
)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,
    
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    /// Suppress non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,
    
    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
    
    /// Set interface language (en, ko)
    #[arg(long, global = true, value_name = "LANG")]
    pub lang: Option<String>,
    
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate content using AI
    Generate(GenerateArgs),
}

impl Cli {
    pub async fn execute(self) -> Result<()> {
        // Apply global settings
        if self.no_color || std::env::var("NO_COLOR").is_ok() {
            colored::control::set_override(false);
        }
        
        // Execute the command
        match self.command {
            Commands::Generate(args) => generate::execute(args).await,
        }
    }
}