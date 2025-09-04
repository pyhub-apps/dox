use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

/// Create Word or PowerPoint documents from Markdown files
#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Input Markdown file
    #[arg(short, long, value_name = "FILE")]
    pub from: PathBuf,

    /// Output document path
    #[arg(short, long, value_name = "FILE")]
    pub output: PathBuf,

    /// Template document for styling
    #[arg(short, long, value_name = "FILE")]
    pub template: Option<PathBuf>,

    /// Output format (auto-detected from extension if not specified)
    #[arg(long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Overwrite existing files without prompting
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Docx,
    Pptx,
}

pub async fn execute(args: CreateArgs) -> Result<()> {
    use dox_core::create::{
        CreateOptions, CreateRequest, DocumentCreatorFactory, MarkdownParser, OutputFormat,
    };
    use dox_core::utils::ui;

    ui::print_header(&format!(
        "📄 Creating document from '{}'",
        args.from.display()
    ));

    // Validate input file
    if !args.from.exists() {
        return Err(anyhow::anyhow!(
            "Input file not found: {}",
            args.from.display()
        ));
    }

    // Detect output format from extension or use explicit format
    let output_format = if let Some(format) = args.format {
        match format {
            crate::cli::commands::create::OutputFormat::Docx => OutputFormat::Word,
            crate::cli::commands::create::OutputFormat::Pptx => OutputFormat::PowerPoint,
        }
    } else {
        // Auto-detect from output file extension
        if let Some(ext) = args.output.extension().and_then(|s| s.to_str()) {
            OutputFormat::from_extension(ext).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unsupported output format: {}. Supported formats: docx, pptx",
                    ext
                )
            })?
        } else {
            return Err(anyhow::anyhow!(
                "Cannot determine output format. Please specify --format or use .docx/.pptx extension"
            ));
        }
    };

    ui::print_step(
        1,
        4,
        &format!("📋 Parsing Markdown file: {}", args.from.display()),
    );

    // Create parser with options
    let mut create_options = CreateOptions::default();
    create_options.title = args
        .output
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    if let Some(template) = &args.template {
        ui::print_info(&format!("📋 Using template: {}", template.display()));
    }

    let parser = MarkdownParser::new(create_options.clone());
    let markdown_doc = parser
        .parse_file(&args.from)
        .map_err(|e| anyhow::anyhow!("Failed to parse Markdown file: {}", e))?;

    ui::print_success(&format!(
        "✅ Parsed {} sections with title: '{}'",
        markdown_doc.sections.len(),
        markdown_doc.title.as_deref().unwrap_or("Untitled")
    ));

    ui::print_step(
        2,
        4,
        &format!("🔧 Creating {} document", output_format.as_str()),
    );

    // Create document creation request
    let request = CreateRequest {
        content: std::fs::read_to_string(&args.from)?,
        format: output_format,
        template_path: args.template.as_ref().map(|p| p.display().to_string()),
        output_path: args.output.display().to_string(),
        options: create_options,
    };

    // Check if output file exists and handle --force flag
    if args.output.exists() && !args.force {
        ui::print_warning(&format!(
            "Output file '{}' already exists. Use --force to overwrite.",
            args.output.display()
        ));
        return Ok(());
    }

    ui::print_step(3, 4, "🚀 Generating document");

    // Create document using appropriate creator
    let creator = DocumentCreatorFactory::create_creator(output_format)?;
    creator.create_document(&markdown_doc, &request)?;

    ui::print_step(4, 4, "✅ Finalizing");

    // Show summary
    ui::print_header("📊 Summary");
    ui::print_success(&format!(
        "✅ Successfully created {} document: {}",
        output_format.as_str().to_uppercase(),
        args.output.display()
    ));

    if let Some(title) = &markdown_doc.title {
        ui::print_info(&format!("📄 Title: {}", title));
    }

    if let Some(author) = &markdown_doc.metadata.author {
        ui::print_info(&format!("👤 Author: {}", author));
    }

    ui::print_info(&format!("📑 Sections: {}", markdown_doc.sections.len()));

    let content_count = markdown_doc
        .sections
        .iter()
        .map(|s| s.content.len())
        .sum::<usize>();
    ui::print_info(&format!("📝 Content elements: {}", content_count));

    if let Some(template) = &args.template {
        ui::print_info(&format!("📋 Template used: {}", template.display()));
    }

    Ok(())
}
