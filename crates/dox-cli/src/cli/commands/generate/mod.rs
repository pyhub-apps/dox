use anyhow::{Context, Result, anyhow};
use clap::Args;
use std::path::PathBuf;
use std::fs;
use tracing::{info, debug, warn};
use colored::*;
use serde::{Deserialize, Serialize};


/// Generate content using AI (OpenAI, Claude, or other providers)
#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// Generation prompt
    #[arg(short, long)]
    pub prompt: String,
    
    /// Content type to generate
    #[arg(short = 't', long, value_enum, default_value = "custom")]
    pub content_type: ContentType,
    
    /// Output file path (stdout if not specified)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
    
    /// AI model to use
    #[arg(long, default_value = "gpt-3.5-turbo")]
    pub model: String,
    
    /// Maximum tokens in response
    #[arg(long, default_value = "2000")]
    pub max_tokens: usize,
    
    /// Creativity level (0.0-1.0)
    #[arg(long, default_value = "0.7")]
    pub temperature: f32,
    
    /// AI provider (auto-detected from model if not specified)
    #[arg(long, value_enum)]
    pub provider: Option<AIProvider>,
    
    /// API key (uses environment variable if not specified)
    #[arg(long)]
    pub api_key: Option<String>,
    
    /// Template file for customizing prompts
    #[arg(long, value_name = "FILE")]
    pub template: Option<PathBuf>,
    
    /// Additional context file to include in prompt
    #[arg(long, value_name = "FILE")]
    pub context_file: Option<PathBuf>,
    
    /// Variables for template substitution (key=value format)
    #[arg(long, value_name = "KEY=VALUE")]
    pub variables: Vec<String>,
    
    /// Enable streaming output
    #[arg(long)]
    pub stream: bool,
    
    /// Language for localized content
    #[arg(long, default_value = "en")]
    pub language: String,
    
    /// Target audience (general, technical, business)
    #[arg(long, default_value = "general")]
    pub audience: String,
    
    /// Tone of voice (formal, casual, professional, friendly)
    #[arg(long, default_value = "professional")]
    pub tone: String,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    /// Blog post or article
    Blog,
    /// Technical documentation
    Documentation,
    /// Technical report or analysis
    Report,
    /// Summary of existing content
    Summary,
    /// Email or message
    Email,
    /// Business proposal
    Proposal,
    /// Custom content with user-defined template
    Custom,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AIProvider {
    OpenAI,
    Claude,
    Local,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub prompt: String,
    pub content_type: ContentType,
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub provider: AIProvider,
    pub language: String,
    pub audience: String,
    pub tone: String,
    pub context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationResponse {
    pub content: String,
    pub model: String,
    pub provider: AIProvider,
    pub usage: Option<Usage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

pub async fn execute(args: GenerateArgs) -> Result<()> {
    info!("Starting content generation with type: {}", args.content_type.as_str());
    
    // Validate arguments
    validate_args(&args)?;
    
    // Parse template variables
    let variables = parse_variables(&args.variables)?;
    debug!("Template variables: {:?}", variables);
    
    // Load context from file if provided
    let context = if let Some(context_file) = &args.context_file {
        Some(load_context_file(context_file)?)
    } else {
        None
    };
    
    // Build the generation request
    let request = build_request(&args, context, &variables).await?;
    debug!("Generation request: {:?}", request);
    
    // Generate content
    let response = generate_content(&request).await?;
    
    // Output the result
    output_content(&args, &response).await?;
    
    // Print summary
    print_generation_summary(&response);
    
    info!("Content generation completed successfully");
    Ok(())
}

fn validate_args(args: &GenerateArgs) -> Result<()> {
    // Validate temperature
    if !(0.0..=1.0).contains(&args.temperature) {
        return Err(anyhow!("Temperature must be between 0.0 and 1.0, got: {}", args.temperature));
    }
    
    // Validate max_tokens
    if args.max_tokens == 0 || args.max_tokens > 100000 {
        return Err(anyhow!("Max tokens must be between 1 and 100000, got: {}", args.max_tokens));
    }
    
    // Check if context file exists
    if let Some(context_file) = &args.context_file {
        if !context_file.exists() {
            return Err(anyhow!("Context file not found: {}", context_file.display()));
        }
    }
    
    // Check if template file exists
    if let Some(template_file) = &args.template {
        if !template_file.exists() {
            return Err(anyhow!("Template file not found: {}", template_file.display()));
        }
    }
    
    Ok(())
}

fn parse_variables(variables: &[String]) -> Result<std::collections::HashMap<String, String>> {
    let mut result = std::collections::HashMap::new();
    
    for var in variables {
        let parts: Vec<&str> = var.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid variable format: '{}'. Expected 'key=value'", var));
        }
        result.insert(parts[0].to_string(), parts[1].to_string());
    }
    
    Ok(result)
}

fn load_context_file(path: &PathBuf) -> Result<String> {
    info!("Loading context from file: {}", path.display());
    fs::read_to_string(path)
        .with_context(|| format!("Failed to read context file: {}", path.display()))
}

async fn build_request(
    args: &GenerateArgs,
    context: Option<String>,
    variables: &std::collections::HashMap<String, String>,
) -> Result<GenerationRequest> {
    // Build the base prompt
    let mut prompt = args.prompt.clone();
    
    // Load and apply template if provided
    if let Some(template_path) = &args.template {
        prompt = apply_template(template_path, &args.prompt, variables).await?;
    }
    
    // Add content type specific instructions
    prompt = enhance_prompt_for_content_type(&prompt, args.content_type, &args.language, &args.audience, &args.tone)?;
    
    // Add context if provided
    if let Some(ref ctx) = context {
        prompt = format!("{}\n\nAdditional Context:\n{}", prompt, ctx);
    }
    
    // Determine provider if not specified
    let provider = args.provider.unwrap_or_else(|| detect_provider(&args.model));
    
    Ok(GenerationRequest {
        prompt,
        content_type: args.content_type,
        model: args.model.clone(),
        max_tokens: args.max_tokens,
        temperature: args.temperature,
        provider,
        language: args.language.clone(),
        audience: args.audience.clone(),
        tone: args.tone.clone(),
        context,
    })
}

async fn apply_template(
    template_path: &PathBuf,
    user_prompt: &str,
    variables: &std::collections::HashMap<String, String>,
) -> Result<String> {
    let template_content = fs::read_to_string(template_path)
        .with_context(|| format!("Failed to read template file: {}", template_path.display()))?;
    
    let mut result = template_content;
    
    // Replace {{prompt}} with user's prompt
    result = result.replace("{{prompt}}", user_prompt);
    
    // Replace variables
    for (key, value) in variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    
    // Check for unresolved placeholders
    if result.contains("{{") && result.contains("}}") {
        warn!("Template contains unresolved placeholders");
    }
    
    Ok(result)
}

fn enhance_prompt_for_content_type(
    prompt: &str,
    content_type: ContentType,
    language: &str,
    audience: &str,
    tone: &str,
) -> Result<String> {
    let type_instructions = match content_type {
        ContentType::Blog => {
            "Write a blog post that is engaging, informative, and well-structured with a compelling title, introduction, body sections, and conclusion."
        }
        ContentType::Documentation => {
            "Create comprehensive technical documentation that is clear, accurate, and well-organized with proper headings, code examples where applicable, and step-by-step instructions."
        }
        ContentType::Report => {
            "Generate a professional technical report with executive summary, detailed analysis, findings, and recommendations. Include data, charts, and evidence-based conclusions."
        }
        ContentType::Summary => {
            "Create a concise summary that captures the key points, main ideas, and important details while maintaining the essential information."
        }
        ContentType::Email => {
            "Write a professional email with appropriate subject line, greeting, clear message body, and proper closing."
        }
        ContentType::Proposal => {
            "Create a business proposal with problem statement, proposed solution, benefits, timeline, and next steps."
        }
        ContentType::Custom => {
            "Generate content according to the specific requirements provided in the prompt."
        }
    };
    
    let enhanced_prompt = format!(
        "{}\n\nContent Type: {}\nInstructions: {}\nLanguage: {}\nAudience: {}\nTone: {}\n\nUser Request:\n{}",
        type_instructions,
        content_type.as_str(),
        type_instructions,
        language,
        audience,
        tone,
        prompt
    );
    
    Ok(enhanced_prompt)
}

fn detect_provider(model: &str) -> AIProvider {
    if model.starts_with("gpt-") || model.starts_with("davinci") || model.starts_with("curie") || model.starts_with("babbage") || model.starts_with("ada") {
        AIProvider::OpenAI
    } else if model.starts_with("claude-") || model.contains("anthropic") {
        AIProvider::Claude
    } else {
        AIProvider::Local
    }
}

async fn generate_content(request: &GenerationRequest) -> Result<GenerationResponse> {
    info!("Generating content with {} provider using model {}", 
          request.provider.as_str(), request.model);
    
    // This is where we would integrate with the actual AI providers
    // For now, we'll create a placeholder implementation
    match request.provider {
        AIProvider::OpenAI => generate_with_openai(request).await,
        AIProvider::Claude => generate_with_claude(request).await,
        AIProvider::Local => generate_with_local(request).await,
    }
}

async fn generate_with_openai(_request: &GenerationRequest) -> Result<GenerationResponse> {
    // TODO: Implement OpenAI API integration using dox-ai crate
    warn!("OpenAI integration not yet implemented");
    Ok(GenerationResponse {
        content: "[PLACEHOLDER] This is where OpenAI-generated content would appear. OpenAI integration will be implemented in the dox-ai crate.".to_string(),
        model: _request.model.clone(),
        provider: AIProvider::OpenAI,
        usage: Some(Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }),
    })
}

async fn generate_with_claude(_request: &GenerationRequest) -> Result<GenerationResponse> {
    // TODO: Implement Claude API integration using dox-ai crate
    warn!("Claude integration not yet implemented");
    Ok(GenerationResponse {
        content: "[PLACEHOLDER] This is where Claude-generated content would appear. Claude integration will be implemented in the dox-ai crate.".to_string(),
        model: _request.model.clone(),
        provider: AIProvider::Claude,
        usage: Some(Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }),
    })
}

async fn generate_with_local(_request: &GenerationRequest) -> Result<GenerationResponse> {
    // TODO: Implement local model integration using dox-ai crate
    warn!("Local model integration not yet implemented");
    Ok(GenerationResponse {
        content: "[PLACEHOLDER] This is where locally-generated content would appear. Local model integration will be implemented in the dox-ai crate.".to_string(),
        model: _request.model.clone(),
        provider: AIProvider::Local,
        usage: None,
    })
}

async fn output_content(args: &GenerateArgs, response: &GenerationResponse) -> Result<()> {
    let content = &response.content;
    
    match &args.output {
        Some(output_path) => {
            info!("Writing output to file: {}", output_path.display());
            fs::write(output_path, content)
                .with_context(|| format!("Failed to write output to: {}", output_path.display()))?;
            println!("{} Content written to: {}", "✓".green(), output_path.display().to_string().green());
        }
        None => {
            println!("\n{}", "Generated Content:".bold().blue());
            println!("{}", "=".repeat(50).blue());
            println!("{}", content);
        }
    }
    
    Ok(())
}

fn print_generation_summary(response: &GenerationResponse) {
    println!("\n{}", "Generation Summary:".bold().blue());
    println!("{}", "-".repeat(30).blue());
    println!("{} {}", "Model:".cyan(), response.model);
    println!("{} {}", "Provider:".cyan(), response.provider.as_str());
    
    if let Some(usage) = &response.usage {
        println!("{} {}", "Prompt tokens:".cyan(), usage.prompt_tokens);
        println!("{} {}", "Completion tokens:".cyan(), usage.completion_tokens);
        println!("{} {}", "Total tokens:".cyan(), usage.total_tokens);
    }
    
    println!("{} {} characters", "Content length:".cyan(), response.content.len());
}

impl ContentType {
    pub fn as_str(&self) -> &str {
        match self {
            ContentType::Blog => "blog",
            ContentType::Documentation => "documentation",
            ContentType::Report => "report",
            ContentType::Summary => "summary",
            ContentType::Email => "email",
            ContentType::Proposal => "proposal",
            ContentType::Custom => "custom",
        }
    }
}

impl AIProvider {
    pub fn as_str(&self) -> &str {
        match self {
            AIProvider::OpenAI => "openai",
            AIProvider::Claude => "claude",
            AIProvider::Local => "local",
        }
    }
}