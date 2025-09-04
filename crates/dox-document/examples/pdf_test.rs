//! PDF 고급 기능 테스트 도구

use dox_document::pdf::PdfProvider;
use dox_document::DocumentProvider;
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("사용법: cargo run --example pdf_test <PDF파일경로> [옵션]");
        println!("옵션:");
        println!("  --basic     : 기본 텍스트 추출");
        println!("  --advanced  : 고급 텍스트 추출 (레이아웃 보존)");
        println!("  --tables    : 테이블 추출");
        println!("  --metadata  : 메타데이터 추출");
        println!("  --encryption: 암호화 상태 확인");
        println!("  --stats     : 추출 통계");
        println!("  --html      : HTML 출력 테스트 (테이블 레이아웃 보존)");
        println!("  --all       : 모든 정보 추출");
        return Ok(());
    }

    let pdf_path = &args[1];
    let option = args.get(2).map(|s| s.as_str()).unwrap_or("--all");

    println!("🔍 PDF 파일 분석: {}", pdf_path);
    println!("📊 옵션: {}", option);
    println!("{}", "=".repeat(50));

    let path = Path::new(pdf_path);
    if !path.exists() {
        println!("❌ 파일을 찾을 수 없습니다: {}", pdf_path);
        return Ok(());
    }

    // PDF Provider 생성 (고급 기능 활성화)
    let provider = match PdfProvider::open_layout_critical(path) {
        Ok(p) => {
            println!("✅ PDF 파일 열기 성공");
            p
        }
        Err(e) => {
            println!("❌ PDF 파일 열기 실패: {}", e);
            return Ok(());
        }
    };

    match option {
        "--basic" => test_basic_extraction(&provider)?,
        "--advanced" => test_advanced_extraction(&provider)?,
        "--tables" => test_table_extraction(&provider)?,
        "--metadata" => test_metadata_extraction(&provider)?,
        "--encryption" => test_encryption_check(&provider)?,
        "--stats" => test_extraction_stats(&provider)?,
        "--html" => test_html_output(&provider)?,
        "--all" => test_all_features(&provider)?,
        _ => {
            println!("❌ 알 수 없는 옵션: {}", option);
            return Ok(());
        }
    }

    Ok(())
}

fn test_basic_extraction(provider: &PdfProvider) -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 기본 텍스트 추출 테스트");
    println!("{}", "-".repeat(30));

    match provider.get_text() {
        Ok(text) => {
            println!("✅ 텍스트 추출 성공");
            println!("📏 텍스트 길이: {} 문자", text.len());
            println!("📝 처음 200자:");
            println!("{}", text.chars().take(200).collect::<String>());
            if text.len() > 200 {
                println!("...(더 많은 내용 생략)");
            }
        }
        Err(e) => {
            println!("❌ 텍스트 추출 실패: {}", e);
        }
    }

    Ok(())
}

fn test_advanced_extraction(provider: &PdfProvider) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 고급 텍스트 추출 테스트 (레이아웃 보존)");
    println!("{}", "-".repeat(30));

    match provider.get_advanced_text() {
        Ok(text) => {
            println!("✅ 고급 텍스트 추출 성공");
            println!("📏 텍스트 길이: {} 문자", text.len());
            println!("📝 처음 300자 (구조 포함):");
            println!("{}", text.chars().take(300).collect::<String>());

            // 테이블 마커 확인
            let table_count = text.matches("[TABLE]").count();
            if table_count > 0 {
                println!("🔍 감지된 테이블: {}개", table_count);
            }
        }
        Err(e) => {
            println!("❌ 고급 텍스트 추출 실패: {}", e);
            println!("🔄 기본 추출로 대체 시도...");
            test_basic_extraction(provider)?;
        }
    }

    Ok(())
}

fn test_table_extraction(provider: &PdfProvider) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 테이블 추출 테스트");
    println!("{}", "-".repeat(30));

    match provider.extract_tables() {
        Ok(tables) => {
            println!("✅ 테이블 추출 성공");
            println!("📊 발견된 테이블: {}개", tables.len());

            for (i, table) in tables.iter().enumerate().take(3) {
                // 최대 3개까지만 표시
                println!("\n📋 테이블 #{}", i + 1);
                println!("   크기: {}행 × {}열", table.rows, table.cols);
                println!("   신뢰도: {:.1}%", table.confidence * 100.0);

                if !table.data.is_empty() {
                    println!("   데이터 미리보기:");
                    for (row_idx, row) in table.data.iter().enumerate().take(3) {
                        println!("   {}: {}", row_idx + 1, row.join(" | "));
                    }
                    if table.data.len() > 3 {
                        println!("   ...(총 {}행)", table.data.len());
                    }
                }
            }

            if tables.len() > 3 {
                println!("   ...(총 {}개 테이블)", tables.len());
            }
        }
        Err(e) => {
            println!("❌ 테이블 추출 실패: {}", e);
        }
    }

    Ok(())
}

fn test_metadata_extraction(provider: &PdfProvider) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 메타데이터 추출 테스트");
    println!("{}", "-".repeat(30));

    match provider.get_metadata() {
        Ok(metadata) => {
            println!("✅ 메타데이터 추출 성공");
            println!("📖 제목: {}", metadata.title.as_deref().unwrap_or("없음"));
            println!("👤 저자: {}", metadata.author.as_deref().unwrap_or("없음"));
            println!("📄 주제: {}", metadata.subject.as_deref().unwrap_or("없음"));
            println!(
                "🛠️ 생성 프로그램: {}",
                metadata.creator.as_deref().unwrap_or("없음")
            );
            println!(
                "📅 생성일: {}",
                metadata.created.as_deref().unwrap_or("없음")
            );
            println!(
                "📅 수정일: {}",
                metadata.modified.as_deref().unwrap_or("없음")
            );
            println!("📖 총 페이지: {}", metadata.page_count);
        }
        Err(e) => {
            println!("❌ 메타데이터 추출 실패: {}", e);
        }
    }

    Ok(())
}

fn test_encryption_check(provider: &PdfProvider) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 암호화 상태 확인 테스트");
    println!("{}", "-".repeat(30));

    match provider.check_encryption() {
        Ok(encryption_info) => {
            if encryption_info.is_encrypted {
                println!("🔐 이 PDF는 암호화되어 있습니다");
                println!(
                    "🔧 보안 핸들러: {}",
                    encryption_info
                        .security_handler
                        .as_deref()
                        .unwrap_or("알 수 없음")
                );
                println!(
                    "🔑 알고리즘: {}",
                    encryption_info.algorithm.as_deref().unwrap_or("알 수 없음")
                );

                println!("\n📜 권한 정보:");
                println!(
                    "  🖨️  인쇄: {}",
                    if encryption_info.permissions.print {
                        "허용"
                    } else {
                        "금지"
                    }
                );
                println!(
                    "  ✏️  수정: {}",
                    if encryption_info.permissions.modify {
                        "허용"
                    } else {
                        "금지"
                    }
                );
                println!(
                    "  📋 복사: {}",
                    if encryption_info.permissions.copy {
                        "허용"
                    } else {
                        "금지"
                    }
                );
                println!(
                    "  💬 주석: {}",
                    if encryption_info.permissions.annotate {
                        "허용"
                    } else {
                        "금지"
                    }
                );

                // 일반적인 패스워드 시도
                println!("\n🔐 일반적인 패스워드 시도 중...");
                match provider.try_common_passwords() {
                    Ok(Some(password)) => {
                        println!("✅ 패스워드 발견: '{}'", password);
                    }
                    Ok(None) => {
                        println!("❌ 일반적인 패스워드로 잠금 해제 실패");
                    }
                    Err(e) => {
                        println!("❌ 패스워드 시도 중 오류: {}", e);
                    }
                }
            } else {
                println!("🔓 이 PDF는 암호화되어 있지 않습니다");
            }
        }
        Err(e) => {
            println!("❌ 암호화 상태 확인 실패: {}", e);
        }
    }

    Ok(())
}

fn test_extraction_stats(provider: &PdfProvider) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 추출 통계 테스트");
    println!("{}", "-".repeat(30));

    match provider.get_extraction_stats() {
        Ok(stats) => {
            println!("✅ 통계 추출 성공");
            println!("📖 총 페이지: {}", stats.total_pages);
            println!("📝 텍스트 블록: {}", stats.text_blocks);
            println!("📊 테이블: {}", stats.tables_detected);
            println!("🖼️ 이미지: {}", stats.images_detected);
            println!("⏱️ 처리 시간: {}ms", stats.extraction_time_ms);
            println!("💾 메모리 사용: {:.1}MB", stats.memory_usage_mb);
            println!(
                "🌊 스트리밍 사용: {}",
                if stats.streaming_used {
                    "예"
                } else {
                    "아니오"
                }
            );
        }
        Err(e) => {
            println!("❌ 통계 추출 실패: {}", e);
        }
    }

    Ok(())
}

fn test_html_output(provider: &PdfProvider) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 HTML 출력 테스트 (테이블 레이아웃 보존)");
    println!("{}", "-".repeat(30));

    use dox_document::extract::extractors::UniversalExtractor;
    use dox_document::{ExtractFormat, OutputFormatter};

    // Extract content from PDF
    match UniversalExtractor::extract_from_path(provider.get_path()) {
        Ok(extract_result) => {
            if extract_result.success {
                println!("✅ PDF 내용 추출 성공");
                println!(
                    "📊 발견된 테이블: {}개",
                    extract_result
                        .pages
                        .iter()
                        .map(|p| p.tables.len())
                        .sum::<usize>()
                );

                // Format as HTML
                match OutputFormatter::format(&extract_result, ExtractFormat::Html) {
                    Ok(html_output) => {
                        println!("✅ HTML 변환 성공");
                        println!("📏 HTML 크기: {} 바이트", html_output.len());

                        // Save to temporary file for inspection
                        let temp_path = format!("/tmp/pdf_test_output_{}.html", std::process::id());

                        match std::fs::write(&temp_path, &html_output) {
                            Ok(()) => {
                                println!("💾 HTML 파일 저장됨: {}", temp_path);
                                println!("🌐 브라우저에서 열어보세요:");
                                println!("   open {}", temp_path);
                                println!();

                                // Show preview of HTML structure
                                let lines: Vec<&str> = html_output.lines().collect();
                                if lines.len() > 20 {
                                    println!("📝 HTML 미리보기 (처음 10줄):");
                                    for line in lines.iter().take(10) {
                                        println!("   {}", line);
                                    }
                                    println!("   ...(중간 생략)...");
                                    println!("📝 HTML 미리보기 (마지막 10줄):");
                                    for line in
                                        lines.iter().rev().take(10).collect::<Vec<_>>().iter().rev()
                                    {
                                        println!("   {}", line);
                                    }
                                } else {
                                    println!("📝 전체 HTML:");
                                    println!("{}", html_output);
                                }
                            }
                            Err(e) => {
                                println!("❌ HTML 파일 저장 실패: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ HTML 변환 실패: {}", e);
                    }
                }
            } else {
                println!("❌ PDF 내용 추출 실패");
                if let Some(ref error) = extract_result.error {
                    println!("   오류: {}", error);
                }
            }
        }
        Err(e) => {
            println!("❌ PDF 처리 실패: {}", e);
        }
    }

    Ok(())
}

fn test_all_features(provider: &PdfProvider) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 모든 기능 종합 테스트");
    println!("{}", "=".repeat(50));

    test_metadata_extraction(provider)?;
    println!();

    test_encryption_check(provider)?;
    println!();

    test_advanced_extraction(provider)?;
    println!();

    test_table_extraction(provider)?;
    println!();

    test_extraction_stats(provider)?;
    println!();

    test_html_output(provider)?;

    println!("\n🎉 모든 테스트 완료!");

    Ok(())
}
