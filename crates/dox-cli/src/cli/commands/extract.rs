use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[cfg(feature = "pdf")]
extern crate pdf_extract;

/// 문서에서 텍스트 추출
///
/// 지원 파일 형식:
///   • .docx (Microsoft Word)
///   • .pptx (Microsoft PowerPoint)
///   • .xlsx (Microsoft Excel)
#[cfg_attr(feature = "pdf", doc = "///   • .pdf (PDF 문서)")]
///
/// 예시:
///   # Word 문서에서 텍스트 추출
///   dox extract -i report.docx
///   
///   # JSON 형식으로 메타데이터와 함께 추출
///   dox extract -i presentation.pptx --format json --with-metadata
#[cfg_attr(feature = "pdf", doc = "///   ")]
#[cfg_attr(feature = "pdf", doc = "///   # PDF 문서에서 텍스트 추출")]
#[cfg_attr(
    feature = "pdf",
    doc = "///   dox extract -i document.pdf -o output.txt"
)]
#[derive(Args, Debug)]
pub struct ExtractArgs {
    /// 입력 문서 파일 또는 디렉토리 경로
    ///
    #[cfg_attr(
        feature = "pdf",
        doc = "/// 지원 형식: .docx (Word), .pptx (PowerPoint), .pdf, .xlsx (Excel)"
    )]
    #[cfg_attr(
        not(feature = "pdf"),
        doc = "/// 지원 형식: .docx (Word), .pptx (PowerPoint), .xlsx (Excel)"
    )]
    #[arg(short, long, value_name = "경로")]
    pub input: PathBuf,

    /// 출력 파일 경로 (지정하지 않으면 표준출력)
    #[arg(short, long, value_name = "파일")]
    pub output: Option<PathBuf>,

    /// 출력 형식
    ///
    /// • text: 일반 텍스트 (서식 없음)
    /// • json: 구조화된 JSON (메타데이터 포함 가능)
    /// • markdown: 마크다운 형식 (제목, 목록 등 보존)
    #[arg(
        long,
        value_enum,
        default_value = "text",
        help = "출력 형식\n  • text: 일반 텍스트 (서식 없음)\n  • json: 구조화된 JSON (메타데이터 포함 가능)\n  • markdown: 마크다운 형식 (제목, 목록 등 보존)"
    )]
    pub format: ExtractFormat,

    /// 출력에 메타데이터 포함
    ///
    /// 문서의 작성자, 생성일, 수정일, 페이지 수 등의 정보를 포함합니다.
    /// JSON 형식에서 가장 유용합니다.
    #[arg(long)]
    pub with_metadata: bool,

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

    /// 출력 디렉토리 (여러 파일 처리시)
    ///
    /// 지정하지 않으면 입력 파일과 같은 위치에 저장됩니다.
    #[arg(long, value_name = "경로")]
    pub output_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ExtractFormat {
    Text,
    Json,
    Markdown,
}

pub async fn execute(args: ExtractArgs) -> Result<()> {
    use dox_core::utils::ui;

    // Verify input path exists
    if !args.input.exists() {
        ui::print_error(&format!(
            "입력 경로를 찾을 수 없습니다: {}",
            args.input.display()
        ));
        return Err(anyhow::anyhow!("Path not found: {}", args.input.display()));
    }

    // Find all document files
    let files = if args.input.is_file() {
        // Single file processing
        if is_supported_document(&args.input) {
            vec![args.input.clone()]
        } else {
            ui::print_error(&format!(
                "지원되지 않는 파일 형식입니다: {}",
                args.input.display()
            ));
            return Err(anyhow::anyhow!("Unsupported file format"));
        }
    } else {
        // Directory processing
        find_document_files(&args.input, args.recursive, args.exclude.as_deref())?
    };

    if files.is_empty() {
        ui::print_warning("처리할 문서를 찾을 수 없습니다");
        return Ok(());
    }

    ui::print_header(&format!("{}개 문서 처리", files.len()));

    // Process files
    let results = if args.concurrent && files.len() > 1 {
        process_concurrent(files, &args).await?
    } else {
        process_sequential(files, &args).await?
    };

    // Print summary
    print_summary(&results, &args);

    Ok(())
}

/// Check if a file is a supported document type
fn is_supported_document(path: &std::path::Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some("docx") | Some("pptx") | Some("pdf") | Some("xlsx") => true,
        _ => false,
    }
}

/// Find all document files in a directory
fn find_document_files(
    path: &std::path::Path,
    recursive: bool,
    exclude: Option<&str>,
) -> Result<Vec<std::path::PathBuf>> {
    use glob::Pattern;
    use walkdir::WalkDir;

    let mut files = Vec::new();
    let walker = if recursive {
        WalkDir::new(path)
    } else {
        WalkDir::new(path).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Skip if matches exclude pattern
        if let Some(pattern) = exclude {
            if let Some(file_name) = path.file_name() {
                if Pattern::new(pattern)?.matches(file_name.to_str().unwrap_or("")) {
                    continue;
                }
            }
        }

        if path.is_file() && is_supported_document(path) {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

#[derive(Debug, Default)]
struct ProcessResults {
    files_processed: usize,
    files_succeeded: usize,
    files_failed: usize,
    total_size: u64,
}

/// Process files sequentially
async fn process_sequential(
    files: Vec<std::path::PathBuf>,
    args: &ExtractArgs,
) -> Result<ProcessResults> {
    use dox_core::utils::ui;
    
    let mut results = ProcessResults::default();
    let progress = ui::create_progress_bar(files.len() as u64, "문서 추출 중");

    for (i, file) in files.iter().enumerate() {
        progress.set_message(format!("처리 중: {}", file.display()));
        
        match process_single_file(file, args).await {
            Ok(size) => {
                results.files_succeeded += 1;
                results.total_size += size;
            }
            Err(e) => {
                ui::print_error(&format!("처리 실패 {}: {}", file.display(), e));
                results.files_failed += 1;
            }
        }
        
        results.files_processed += 1;
        progress.set_position((i + 1) as u64);
    }

    progress.finish_with_message("추출 완료");
    Ok(results)
}

/// Process files concurrently
async fn process_concurrent(
    files: Vec<std::path::PathBuf>,
    args: &ExtractArgs,
) -> Result<ProcessResults> {
    use dox_core::utils::ui;
    use futures::stream::{self, StreamExt};
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

    let max_workers = args.max_workers.min(files.len());
    let progress = ui::create_progress_bar(
        files.len() as u64,
        &format!("병렬 추출 중 ({}개 작업자)", max_workers),
    );
    let completed = Arc::new(AtomicUsize::new(0));

    let results = stream::iter(files)
        .map(|file| {
            let args = args.clone();
            let progress = progress.clone();
            let completed = Arc::clone(&completed);
            async move {
                let result = process_single_file(&file, &args).await
                    .map(|size| (1, 1, 0, size))
                    .unwrap_or_else(|_| (1, 0, 1, 0));
                
                let current = completed.fetch_add(1, Ordering::SeqCst) + 1;
                progress.set_position(current as u64);
                
                result
            }
        })
        .buffer_unordered(max_workers)
        .fold(
            ProcessResults::default(),
            |mut acc, (processed, succeeded, failed, size)| async move {
                acc.files_processed += processed;
                acc.files_succeeded += succeeded;
                acc.files_failed += failed;
                acc.total_size += size;
                acc
            },
        )
        .await;

    progress.finish_with_message("병렬 추출 완료");
    Ok(results)
}

/// Process a single file
async fn process_single_file(file: &std::path::Path, args: &ExtractArgs) -> Result<u64> {
    use dox_document::extract::extractors::UniversalExtractor;
    use dox_document::OutputFormatter;
    use std::fs;

    // Extract content from document
    let extract_result = UniversalExtractor::extract_from_path(file)?;

    if !extract_result.success {
        if let Some(ref error) = extract_result.error {
            return Err(anyhow::anyhow!("Extraction error: {}", error));
        }
    }

    // Convert ExtractFormat enum from clap to our internal enum
    let output_format = match args.format {
        ExtractFormat::Text => dox_document::ExtractFormat::Text,
        ExtractFormat::Json => dox_document::ExtractFormat::Json,
        ExtractFormat::Markdown => dox_document::ExtractFormat::Markdown,
    };

    // Format the output
    let formatted_output = OutputFormatter::format(&extract_result, output_format)?;

    // Determine output path
    let output_path = determine_output_path(file, args)?;

    // Write to file if output path specified
    if let Some(path) = output_path {
        // Create directory if needed
        if let Some(parent_dir) = path.parent() {
            fs::create_dir_all(parent_dir)?;
        }

        fs::write(&path, &formatted_output)?;
        
        // Print individual file success (only in sequential mode)
        if !args.concurrent {
            use dox_core::utils::ui;
            ui::print_success(&format!(
                "저장됨: {} → {}",
                file.display(),
                path.display()
            ));
        }
    } else if args.input.is_file() {
        // Single file to stdout
        println!("{}", formatted_output);
    }

    Ok(formatted_output.len() as u64)
}

/// Determine output path for a file
fn determine_output_path(
    input_file: &std::path::Path,
    args: &ExtractArgs,
) -> Result<Option<std::path::PathBuf>> {
    // If single file and no output specified, use stdout
    if args.input.is_file() && args.output.is_none() && args.output_dir.is_none() {
        return Ok(None);
    }

    // If specific output file specified (single file mode)
    if let Some(ref output) = args.output {
        return Ok(Some(output.clone()));
    }

    // Determine output directory
    let output_dir = args.output_dir.as_ref().cloned().unwrap_or_else(|| {
        input_file.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
    });

    // Generate output filename with appropriate extension
    let input_stem = input_file.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("extracted");
    
    let extension = args.format.extension();
    let output_filename = format!("{}.{}", input_stem, extension);
    
    Ok(Some(output_dir.join(output_filename)))
}

impl ExtractFormat {
    fn extension(&self) -> &'static str {
        match self {
            ExtractFormat::Text => "txt",
            ExtractFormat::Json => "json",
            ExtractFormat::Markdown => "md",
        }
    }
}

// Add Clone derive for ExtractArgs

impl Clone for ExtractArgs {
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            output: self.output.clone(),
            format: self.format,
            with_metadata: self.with_metadata,
            recursive: self.recursive,
            exclude: self.exclude.clone(),
            concurrent: self.concurrent,
            max_workers: self.max_workers,
            output_dir: self.output_dir.clone(),
        }
    }
}

/// Print processing summary
fn print_summary(results: &ProcessResults, _args: &ExtractArgs) {
    use dox_core::utils::ui;

    ui::print_header("추출 완료");
    ui::print_success(&format!(
        "{}개 파일 처리 완료 (성공: {}, 실패: {})",
        results.files_processed, results.files_succeeded, results.files_failed
    ));
    
    if results.total_size > 0 {
        ui::print_info(&format!(
            "총 추출된 텍스트 크기: {}",
            ui::format_size(results.total_size)
        ));
    }

    if results.files_failed > 0 {
        ui::print_warning(&format!(
            "{}개 파일에서 오류가 발생했습니다",
            results.files_failed
        ));
    }
}
