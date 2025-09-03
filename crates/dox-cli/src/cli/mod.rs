use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod commands;
use commands::*;

/// 문서 자동화 및 AI 기반 콘텐츠 생성 CLI
#[derive(Parser, Debug)]
#[command(
    name = "dox",
    version,
    author,
    about = "문서 자동화 및 AI 기반 콘텐츠 생성 CLI",
    long_about = None,
    arg_required_else_help = true
)]
pub struct Cli {
    /// 설정 파일 경로
    #[arg(short, long, value_name = "파일", global = true)]
    pub config: Option<PathBuf>,

    /// 상세 출력 활성화
    #[arg(short, long, global = true, conflicts_with = "quiet")]
    pub verbose: bool,

    /// 오류 외 출력 억제
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// 색상 출력 비활성화
    #[arg(long, global = true)]
    pub no_color: bool,

    /// 인터페이스 언어 설정 (en, ko)
    #[arg(long, global = true, value_name = "언어")]
    pub lang: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// YAML 규칙 파일을 사용하여 문서의 텍스트 치환
    Replace(ReplaceArgs),

    /// Markdown 파일에서 문서 생성
    Create(CreateArgs),

    /// 플레이스홀더가 포함된 문서 템플릿 처리
    Template(TemplateArgs),

    /// AI를 사용하여 콘텐츠 생성
    Generate(GenerateArgs),

    /// 문서에서 텍스트 추출
    Extract(ExtractArgs),

    /// 설정 관리
    Config(ConfigArgs),
}

impl Cli {
    pub async fn execute(self) -> Result<()> {
        // Apply global settings
        if self.no_color || std::env::var("NO_COLOR").is_ok() {
            colored::control::set_override(false);
        }

        // Execute the command
        match self.command {
            Commands::Replace(args) => replace::execute(args).await,
            Commands::Create(args) => create::execute(args).await,
            Commands::Template(args) => template::execute(args).await,
            Commands::Generate(args) => generate::execute(args).await,
            Commands::Extract(args) => extract::execute(args).await,
            Commands::Config(args) => config::execute(args).await,
        }
    }
}
