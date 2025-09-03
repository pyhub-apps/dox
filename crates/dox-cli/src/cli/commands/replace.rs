use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

/// Word 및 PowerPoint 문서의 텍스트 치환
///
/// YAML 규칙 파일 형식:
/// ```yaml
/// replacements:
///   - old: "기존 텍스트"
///     new: "새 텍스트"
///   - old: "{{date}}"
///     new: "2025-09-02"
/// ```
///
/// 예시:
///   # YAML 규칙으로 일괄 치환
///   dox replace -r rules.yaml -p document.docx
///   
///   # 단일 텍스트 치환
///   dox replace -f "{{이름}}" -t "김철수" -p document.docx
///   
///   # AI 스마트 교체
///   dox replace -f "기존 요약" -t "새로운 요약" -p report.docx --ai-smart --ai-context "기술 보고서"
///   
///   # 디렉토리 재귀적 치환 (미리보기)
///   dox replace -r rules.yaml -p ./docs --recursive --dry-run
#[derive(Args, Debug)]
pub struct ReplaceArgs {
    /// 치환 규칙이 포함된 YAML 파일
    ///
    /// 형식: replacements 키 아래에 old/new 쌍의 목록
    #[arg(short, long, value_name = "파일")]
    pub rules: Option<PathBuf>,

    /// 대상 파일 또는 디렉토리 경로
    #[arg(short, long, value_name = "경로")]
    pub path: PathBuf,

    /// 실제 변경 없이 미리보기
    #[arg(long)]
    pub dry_run: bool,

    /// 수정 전 백업 파일 생성
    #[arg(long)]
    pub backup: bool,

    /// 하위 디렉토리까지 재귀적으로 처리
    #[arg(long, default_value = "true")]
    pub recursive: bool,

    /// 제외할 파일의 glob 패턴
    ///
    /// 예: "*.tmp", "backup/*", "~$*"
    #[arg(long, value_name = "패턴")]
    pub exclude: Option<String>,

    /// 병렬 처리 활성화
    #[arg(long)]
    pub concurrent: bool,

    /// 최대 병렬 작업자 수
    #[arg(long, value_name = "수", default_value = "4")]
    pub max_workers: usize,

    /// 각 변경사항의 차이점 표시
    #[arg(long)]
    pub show_diff: bool,

    /// AI 기반 스마트 교체 활성화
    #[arg(long)]
    pub ai_smart: bool,

    /// AI 교체에 사용할 컨텍스트 정보
    #[arg(long, value_name = "컨텍스트")]
    pub ai_context: Option<String>,

    /// 사용할 AI 모델
    #[arg(long, default_value = "gpt-3.5-turbo")]
    pub ai_model: String,

    /// 단일 텍스트 교체 (YAML 파일 대신)
    #[arg(short = 'f', long, value_name = "찾을텍스트")]
    pub find: Option<String>,

    /// 교체할 텍스트 (--find와 함께 사용)
    #[arg(short = 't', long, value_name = "교체텍스트")]
    pub to: Option<String>,
}

pub async fn execute(args: ReplaceArgs) -> Result<()> {
    use dox_core::utils::ui;
    use dox_document::replace::{Replacer, Rule};
    use dox_document::validate_file_access;

    // Validate arguments
    if args.find.is_some() != args.to.is_some() {
        return Err(anyhow::anyhow!(
            "--find와 --to 옵션은 함께 사용해야 합니다"
        ));
    }

    if args.find.is_none() && args.rules.is_none() {
        return Err(anyhow::anyhow!(
            "--rules 파일을 지정하거나 --find/--to 옵션을 사용해야 합니다"
        ));
    }

    // Load replacement rules
    let rules = if let (Some(find), Some(to)) = (&args.find, &args.to) {
        // Single replacement mode
        vec![Rule::new(find.clone(), to.clone())]
    } else if let Some(rules_path) = &args.rules {
        // Load from YAML file
        dox_document::replace::load_rules(rules_path)?
    } else {
        vec![] // This shouldn't happen due to validation above
    };

    if rules.is_empty() {
        ui::print_warning("파일에서 치환 규칙을 찾을 수 없습니다");
        return Ok(());
    }

    // Validate file access before processing
    if args.path.is_file() {
        if let Err(e) = validate_file_access(&args.path) {
            ui::print_error(&format!("파일 접근 오류: {}", e));
            return Err(e.into());
        }
    }

    // Display rules in dry-run mode
    if args.dry_run {
        ui::print_header("적용할 치환 규칙");
        for (i, rule) in rules.iter().enumerate() {
            ui::print_step(
                i + 1,
                rules.len(),
                &format!("'{}' → '{}'로 치환", rule.old, rule.new),
            );
        }
    }

    // Create replacer instance
    let replacer = if args.ai_smart {
        // Get API key and create AI-enhanced replacer
        match get_ai_api_key(&args.ai_model) {
            Ok(api_key) => {
                ui::print_info("🤖 AI 스마트 교체 모드 활성화");
                match Replacer::with_smart_replacement(
                    rules.clone(),
                    args.ai_model.clone(),
                    api_key,
                    args.ai_context.clone(),
                ) {
                    Ok(smart_replacer) => smart_replacer,
                    Err(e) => {
                        ui::print_warning(&format!(
                            "AI 초기화 실패, 일반 모드로 전환: {}", e
                        ));
                        Replacer::new(rules)
                    }
                }
            }
            Err(e) => {
                ui::print_warning(&format!("API 키 없음, 일반 모드로 전환: {}", e));
                Replacer::new(rules)
            }
        }
    } else {
        Replacer::new(rules)
    };

    // Process documents
    let options = dox_document::replace::ReplaceOptions {
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
    ui::print_header("요약");
    ui::print_success(&format!(
        "{}개 파일에서 {}개 항목을 치환했습니다",
        results.files_processed, results.total_replacements
    ));

    if results.errors > 0 {
        ui::print_error(&format!(
            "{}개 파일에서 오류가 발생했습니다",
            results.errors
        ));
    }

    Ok(())
}

/// Get API key for AI functionality
fn get_ai_api_key(model: &str) -> Result<String> {
    let env_var = if model.starts_with("gpt-") {
        "OPENAI_API_KEY"
    } else if model.starts_with("claude-") {
        "ANTHROPIC_API_KEY"
    } else {
        return Err(anyhow::anyhow!("지원되지 않는 AI 모델: {}", model));
    };

    std::env::var(env_var)
        .map_err(|_| anyhow::anyhow!("{} 환경변수가 필요합니다", env_var))
}
