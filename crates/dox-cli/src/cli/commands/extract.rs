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
#[cfg_attr(feature = "pdf", doc = "///   dox extract -i document.pdf -o output.txt")]
#[derive(Args, Debug)]
pub struct ExtractArgs {
    /// 입력 문서 경로
    /// 
    #[cfg_attr(feature = "pdf", doc = "/// 지원 형식: .docx (Word), .pptx (PowerPoint), .pdf, .xlsx (Excel)")]
    #[cfg_attr(not(feature = "pdf"), doc = "/// 지원 형식: .docx (Word), .pptx (PowerPoint), .xlsx (Excel)")]
    #[arg(short, long, value_name = "파일")]
    pub input: PathBuf,
    
    /// 출력 파일 경로 (지정하지 않으면 표준출력)
    #[arg(short, long, value_name = "파일")]
    pub output: Option<PathBuf>,
    
    /// 출력 형식
    /// 
    /// • text: 일반 텍스트 (서식 없음)
    /// • json: 구조화된 JSON (메타데이터 포함 가능)
    /// • markdown: 마크다운 형식 (제목, 목록 등 보존)
    #[arg(long, value_enum, default_value = "text", help = "출력 형식\n  • text: 일반 텍스트 (서식 없음)\n  • json: 구조화된 JSON (메타데이터 포함 가능)\n  • markdown: 마크다운 형식 (제목, 목록 등 보존)")]
    pub format: ExtractFormat,
    
    /// 출력에 메타데이터 포함
    /// 
    /// 문서의 작성자, 생성일, 수정일, 페이지 수 등의 정보를 포함합니다.
    /// JSON 형식에서 가장 유용합니다.
    #[arg(long)]
    pub with_metadata: bool,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ExtractFormat {
    Text,
    Json,
    Markdown,
}


pub async fn execute(args: ExtractArgs) -> Result<()> {
    use dox_core::utils::ui;
    use dox_document::extract::extractors::UniversalExtractor;
    use dox_document::OutputFormatter;
    use std::fs;
    
    // Verify input file exists
    if !args.input.exists() {
        ui::print_error(&format!("입력 파일을 찾을 수 없습니다: {}", args.input.display()));
        return Err(anyhow::anyhow!("File not found: {}", args.input.display()));
    }
    
    ui::print_info(&format!(
        "'{}' 파일에서 텍스트를 추출하는 중...",
        args.input.display()
    ));
    
    // Extract content from document
    let extract_result = match UniversalExtractor::extract_from_path(&args.input) {
        Ok(result) => result,
        Err(e) => {
            ui::print_error(&format!("텍스트 추출 실패: {}", e));
            return Err(anyhow::anyhow!("Extraction failed: {}", e));
        }
    };
    
    // Check if extraction was successful
    if !extract_result.success {
        if let Some(ref error) = extract_result.error {
            ui::print_error(&format!("추출 오류: {}", error));
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
    let formatted_output = OutputFormatter::format(&extract_result, output_format)
        .map_err(|e| anyhow::anyhow!("Output formatting failed: {}", e))?;
    
    // Output the result
    if let Some(ref output_path) = args.output {
        // Write to file
        if let Some(parent_dir) = output_path.parent() {
            fs::create_dir_all(parent_dir)
                .map_err(|e| anyhow::anyhow!("Failed to create output directory: {}", e))?;
        }
        
        fs::write(output_path, &formatted_output)
            .map_err(|e| anyhow::anyhow!("Failed to write output file: {}", e))?;
        
        ui::print_success(&format!("추출된 내용이 저장되었습니다: {}", output_path.display()));
        
        // Print summary
        ui::print_info(&format!(
            "문서 형식: {} | 페이지 수: {} | 추출된 텍스트 길이: {} 문자",
            extract_result.format,
            extract_result.metadata.total_pages.max(extract_result.pages.len()),
            formatted_output.len()
        ));
        
        // Show metadata if requested and available
        if args.with_metadata {
            let metadata = &extract_result.metadata;
            ui::print_info("=== 문서 메타데이터 ===");
            if let Some(ref title) = metadata.title {
                ui::print_info(&format!("제목: {}", title));
            }
            if let Some(ref author) = metadata.author {
                ui::print_info(&format!("작성자: {}", author));
            }
            if let Some(ref creator) = metadata.creator {
                ui::print_info(&format!("생성 프로그램: {}", creator));
            }
            if let Some(ref subject) = metadata.subject {
                ui::print_info(&format!("주제: {}", subject));
            }
        }
        
    } else {
        // Write to stdout
        println!("{}", formatted_output);
    }
    
    Ok(())
}