use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

/// AI를 사용하여 콘텐츠 생성 (OpenAI 또는 Claude)
///
/// AI 제공업체 설정:
///   • OpenAI: OPENAI_API_KEY 환경변수 설정
///   • Claude: ANTHROPIC_API_KEY 환경변수 설정
///
/// 예시:
///   # 블로그 포스트 생성
///   dox generate -p "Rust 프로그래밍 입문" -t blog
///   
///   # GPT-4로 보고서 생성
///   dox generate -p "2025년 시장 분석" -t report --model gpt-4
///   
///   # Claude로 이메일 생성
///   dox generate -p "프로젝트 업데이트 공유" -t email --model claude-3-5-sonnet-20241022
#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// 생성 프롬프트
    #[arg(short, long)]
    pub prompt: String,

    /// 생성할 콘텐츠 유형
    ///
    /// • blog: 블로그 포스트
    /// • documentation: 기술 문서
    /// • report: 보고서
    /// • summary: 요약
    /// • email: 이메일
    /// • proposal: 제안서
    /// • custom: 사용자 정의
    #[arg(short = 't', long, value_enum, default_value = "custom")]
    pub content_type: ContentType,

    /// 출력 파일 경로 (지정하지 않으면 표준출력)
    #[arg(short, long, value_name = "파일")]
    pub output: Option<PathBuf>,

    /// 사용할 AI 모델
    #[arg(long, default_value = "gpt-3.5-turbo")]
    pub model: String,

    /// 응답의 최대 토큰 수
    #[arg(long, default_value = "2000")]
    pub max_tokens: usize,

    /// 창의성 수준 (0.0-1.0)
    #[arg(long, default_value = "0.7")]
    pub temperature: f32,

    /// AI 제공업체 (모델에서 자동 감지)
    #[arg(long, value_enum)]
    pub provider: Option<AIProvider>,

    /// API 키 (환경 변수 사용 가능)
    #[arg(long)]
    pub api_key: Option<String>,

    /// 생성할 콘텐츠 언어
    #[arg(long, default_value = "ko")]
    pub language: String,

    /// 대상 독자
    #[arg(long, default_value = "일반")]
    pub audience: String,

    /// 글의 톤
    #[arg(long, default_value = "전문적")]
    pub tone: String,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ContentType {
    Blog,
    Documentation,
    Report,
    Summary,
    Email,
    Proposal,
    Custom,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum AIProvider {
    OpenAI,
    Claude,
}

pub async fn execute(args: GenerateArgs) -> Result<()> {
    use dox_core::generate::{ContentGenerator, GenerationRequest, openai::OpenAIProvider, claude::ClaudeProvider};
    use dox_core::utils::ui;
    use std::fs;

    ui::print_info(&format!(
        "{} 콘텐츠를 생성하는 중...",
        args.content_type.as_str_ko()
    ));

    // Create generation request
    let request = GenerationRequest {
        prompt: args.prompt.clone(),
        content_type: convert_content_type(args.content_type),
        model: args.model.clone(),
        max_tokens: args.max_tokens,
        temperature: args.temperature,
        language: args.language.clone(),
        audience: args.audience.clone(),
        tone: args.tone.clone(),
        context: None,
        stream: false, // TODO: Implement streaming
        provider_params: std::collections::HashMap::new(),
    };

    // Create AI provider
    let provider = match detect_provider(&args.model) {
        "openai" => {
            // Try to get API key from args, environment, or config
            let api_key = get_api_key("openai", args.api_key.as_deref())?;
            Box::new(OpenAIProvider::new(api_key)) as Box<dyn ContentGenerator>
        }
        "claude" => {
            // Try to get API key from args, environment, or config
            let api_key = get_api_key("claude", args.api_key.as_deref())?;
            Box::new(ClaudeProvider::new(api_key)) as Box<dyn ContentGenerator>
        }
        provider_name => {
            return Err(anyhow::anyhow!("지원되지 않는 AI 제공업체: {}", provider_name));
        }
    };

    // Show generation info
    ui::print_info(&format!(
        "🤖 {} 모델로 {} 콘텐츠 생성 중...",
        request.model,
        request.content_type.as_str()
    ));

    // Generate content
    let response = provider.generate(&request).await?;

    // Show token usage if available
    if let Some(usage) = &response.usage {
        ui::print_info(&format!(
            "📊 토큰 사용량: {} (프롬프트: {}, 완성: {})",
            usage.total_tokens, usage.prompt_tokens, usage.completion_tokens
        ));
    }

    // Output content
    if let Some(mut output_path) = args.output {
        // Add appropriate extension if not present
        if output_path.extension().is_none() {
            let extension = get_file_extension(args.content_type);
            output_path = output_path.with_extension(extension);
        }
        
        // Save to file
        fs::write(&output_path, &response.content)?;
        ui::print_success(&format!(
            "✅ 콘텐츠가 생성되어 {}에 저장되었습니다",
            output_path.display()
        ));
    } else {
        // Print to stdout
        println!("\n{}", response.content);
    }

    Ok(())
}

/// Convert CLI content type to core content type
fn convert_content_type(cli_type: ContentType) -> dox_core::generate::ContentType {
    match cli_type {
        ContentType::Blog => dox_core::generate::ContentType::Blog,
        ContentType::Documentation => dox_core::generate::ContentType::Documentation,
        ContentType::Report => dox_core::generate::ContentType::Report,
        ContentType::Summary => dox_core::generate::ContentType::Summary,
        ContentType::Email => dox_core::generate::ContentType::Email,
        ContentType::Proposal => dox_core::generate::ContentType::Proposal,
        ContentType::Custom => dox_core::generate::ContentType::Custom,
    }
}

/// Detect AI provider from model name
fn detect_provider(model: &str) -> &str {
    if model.starts_with("gpt-") {
        "openai"
    } else if model.starts_with("claude-") {
        "claude"
    } else {
        "openai" // Default to OpenAI
    }
}

/// Get API key from various sources
fn get_api_key(provider: &str, cli_key: Option<&str>) -> Result<String> {
    // Priority: CLI arg > environment variable > config file
    if let Some(key) = cli_key {
        return Ok(key.to_string());
    }

    let env_var = match provider {
        "openai" => "OPENAI_API_KEY",
        "claude" => "ANTHROPIC_API_KEY",
        _ => return Err(anyhow::anyhow!("알 수 없는 제공업체: {}", provider)),
    };

    std::env::var(env_var)
        .map_err(|_| anyhow::anyhow!("{} 환경변수 또는 --api-key 옵션이 필요합니다", env_var))
}

/// Get appropriate file extension for content type
fn get_file_extension(content_type: ContentType) -> &'static str {
    match content_type {
        ContentType::Blog => "md",
        ContentType::Documentation => "md",
        ContentType::Report => "md",
        ContentType::Summary => "txt",
        ContentType::Email => "txt",
        ContentType::Proposal => "md",
        ContentType::Custom => "txt",
    }
}

impl ContentType {
    fn as_str(&self) -> &str {
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

    fn as_str_ko(&self) -> &str {
        match self {
            ContentType::Blog => "블로그",
            ContentType::Documentation => "문서",
            ContentType::Report => "보고서",
            ContentType::Summary => "요약",
            ContentType::Email => "이메일",
            ContentType::Proposal => "제안서",
            ContentType::Custom => "사용자 정의",
        }
    }
}
