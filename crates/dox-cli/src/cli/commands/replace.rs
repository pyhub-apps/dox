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
/// 예식:
///   # 단일 파일 치환
///   dox replace -r rules.yaml -p document.docx
///   
///   # 디렉토리 재귀적 치환 (미리보기)
///   dox replace -r rules.yaml -p ./docs --recursive --dry-run
///   
///   # 특정 파일 제외하고 치환
///   dox replace -r rules.yaml -p . --exclude "*.tmp" --exclude "backup/*"
#[derive(Args, Debug)]
pub struct ReplaceArgs {
    /// 치환 규칙이 포함된 YAML 파일
    /// 
    /// 형식: replacements 키 아래에 old/new 쌍의 목록
    #[arg(short, long, value_name = "파일")]
    pub rules: PathBuf,
    
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
}

pub async fn execute(args: ReplaceArgs) -> Result<()> {
    use dox_document::replace::Replacer;
    use dox_core::utils::ui;
    
    // Load replacement rules
    let rules = dox_document::replace::load_rules(&args.rules)?;
    
    if rules.is_empty() {
        ui::print_warning("파일에서 치환 규칙을 찾을 수 없습니다");
        return Ok(());
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
    let replacer = Replacer::new(rules);
    
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
        ui::print_error(&format!("{}개 파일에서 오류가 발생했습니다", results.errors));
    }
    
    Ok(())
}