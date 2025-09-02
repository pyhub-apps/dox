use std::collections::HashMap;
use once_cell::sync::Lazy;

/// 한글 메시지 맵
static MESSAGES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // === 공통 메시지 ===
    m.insert("app.name", "dox");
    m.insert("app.description", "문서 자동화 및 AI 기반 콘텐츠 생성 CLI");
    m.insert("app.version", "버전");
    
    // === 명령어 설명 ===
    m.insert("cmd.replace", "YAML 규칙 파일을 사용하여 문서의 텍스트 치환");
    m.insert("cmd.create", "Markdown 파일에서 문서 생성");
    m.insert("cmd.template", "플레이스홀더가 포함된 문서 템플릿 처리");
    m.insert("cmd.generate", "AI를 사용하여 콘텐츠 생성");
    m.insert("cmd.extract", "문서에서 텍스트 추출");
    m.insert("cmd.config", "설정 관리");
    
    // === 공통 옵션 ===
    m.insert("opt.config", "설정 파일 경로");
    m.insert("opt.verbose", "상세 출력 활성화");
    m.insert("opt.quiet", "오류 외 출력 억제");
    m.insert("opt.no_color", "색상 출력 비활성화");
    m.insert("opt.lang", "인터페이스 언어 설정 (en, ko)");
    m.insert("opt.help", "도움말 표시");
    
    // === Replace 명령어 옵션 ===
    m.insert("opt.replace.rules", "치환 규칙이 포함된 YAML 파일");
    m.insert("opt.replace.path", "대상 파일 또는 디렉토리 경로");
    m.insert("opt.replace.dry_run", "실제 변경 없이 미리보기");
    m.insert("opt.replace.backup", "수정 전 백업 파일 생성");
    m.insert("opt.replace.recursive", "하위 디렉토리까지 재귀적으로 처리");
    m.insert("opt.replace.exclude", "제외할 파일의 glob 패턴");
    m.insert("opt.replace.concurrent", "병렬 처리 활성화");
    m.insert("opt.replace.max_workers", "최대 병렬 작업자 수");
    m.insert("opt.replace.show_diff", "각 변경사항의 차이점 표시");
    
    // === Extract 명령어 옵션 ===
    m.insert("opt.extract.input", "입력 문서 경로");
    m.insert("opt.extract.output", "출력 파일 경로 (지정하지 않으면 표준출력)");
    m.insert("opt.extract.format", "출력 형식");
    m.insert("opt.extract.with_metadata", "출력에 메타데이터 포함");
    
    // === Create 명령어 옵션 ===
    m.insert("opt.create.from", "입력 Markdown 파일");
    m.insert("opt.create.output", "출력 문서 경로");
    m.insert("opt.create.template", "스타일링용 템플릿 문서");
    m.insert("opt.create.format", "출력 형식 (확장자에서 자동 감지)");
    m.insert("opt.create.force", "기존 파일을 묻지 않고 덮어쓰기");
    
    // === Template 명령어 옵션 ===
    m.insert("opt.template.template", "템플릿 파일 경로");
    m.insert("opt.template.output", "출력 파일 경로");
    m.insert("opt.template.values", "값이 포함된 YAML/JSON 파일");
    m.insert("opt.template.set", "개별 값 설정 (key=value)");
    m.insert("opt.template.force", "기존 파일을 묻지 않고 덮어쓰기");
    
    // === Generate 명령어 옵션 ===
    m.insert("opt.generate.prompt", "생성 프롬프트");
    m.insert("opt.generate.content_type", "생성할 콘텐츠 유형");
    m.insert("opt.generate.output", "출력 파일 경로 (지정하지 않으면 표준출력)");
    m.insert("opt.generate.model", "사용할 AI 모델");
    m.insert("opt.generate.max_tokens", "응답의 최대 토큰 수");
    m.insert("opt.generate.temperature", "창의성 수준 (0.0-1.0)");
    m.insert("opt.generate.provider", "AI 제공업체 (모델에서 자동 감지)");
    m.insert("opt.generate.api_key", "API 키 (환경 변수 사용 가능)");
    
    // === Config 명령어 옵션 ===
    m.insert("opt.config.init", "설정 파일 초기화");
    m.insert("opt.config.list", "모든 설정 값 나열");
    m.insert("opt.config.get", "특정 설정 값 가져오기");
    m.insert("opt.config.set", "설정 값 지정");
    m.insert("opt.config.unset", "설정 값 제거");
    
    // === UI 메시지 ===
    m.insert("ui.header", "제목");
    m.insert("ui.info", "정보");
    m.insert("ui.success", "성공");
    m.insert("ui.warning", "경고");
    m.insert("ui.error", "오류");
    m.insert("ui.step", "단계");
    m.insert("ui.progress", "진행 중");
    m.insert("ui.processing", "처리 중");
    m.insert("ui.done", "완료");
    m.insert("ui.failed", "실패");
    m.insert("ui.cancelled", "취소됨");
    
    // === 상태 메시지 ===
    m.insert("status.starting", "시작 중...");
    m.insert("status.loading", "로딩 중...");
    m.insert("status.saving", "저장 중...");
    m.insert("status.extracting", "추출 중...");
    m.insert("status.generating", "생성 중...");
    m.insert("status.replacing", "치환 중...");
    m.insert("status.creating", "생성 중...");
    m.insert("status.processing_template", "템플릿 처리 중...");
    m.insert("status.completed", "완료되었습니다");
    m.insert("status.failed", "실패했습니다");
    
    // === 프롬프트 메시지 ===
    m.insert("prompt.confirm", "계속하시겠습니까?");
    m.insert("prompt.overwrite", "파일이 이미 존재합니다. 덮어쓰시겠습니까?");
    m.insert("prompt.select", "선택하세요");
    m.insert("prompt.enter_value", "값을 입력하세요");
    m.insert("prompt.yes", "예");
    m.insert("prompt.no", "아니오");
    
    // === 에러 메시지 ===
    m.insert("error.file_not_found", "파일을 찾을 수 없습니다");
    m.insert("error.invalid_format", "잘못된 형식입니다");
    m.insert("error.permission_denied", "권한이 거부되었습니다");
    m.insert("error.io_error", "입출력 오류가 발생했습니다");
    m.insert("error.parse_error", "구문 분석 오류가 발생했습니다");
    m.insert("error.api_error", "API 오류가 발생했습니다");
    m.insert("error.not_implemented", "아직 구현되지 않았습니다");
    m.insert("error.command_failed", "명령 실행이 실패했습니다");
    
    // === 성공 메시지 ===
    m.insert("success.file_created", "파일이 생성되었습니다");
    m.insert("success.file_saved", "파일이 저장되었습니다");
    m.insert("success.text_replaced", "텍스트가 치환되었습니다");
    m.insert("success.content_generated", "콘텐츠가 생성되었습니다");
    m.insert("success.text_extracted", "텍스트가 추출되었습니다");
    m.insert("success.config_saved", "설정이 저장되었습니다");
    
    // === 도움말 확장 메시지 ===
    m.insert("help.usage", "사용법");
    m.insert("help.commands", "명령어");
    m.insert("help.options", "옵션");
    m.insert("help.examples", "예시");
    m.insert("help.supported_formats", "지원 형식");
    m.insert("help.default_value", "기본값");
    m.insert("help.possible_values", "가능한 값");
    
    // === Extract 명령어 상세 도움말 ===
    m.insert("help.extract.formats", "• text: 일반 텍스트 (서식 없음)\n• json: 구조화된 JSON (메타데이터 포함 가능)\n• markdown: 마크다운 형식 (제목, 목록 등 보존)");
    m.insert("help.extract.supported_files", "지원 파일: .docx (Word), .pptx (PowerPoint), .pdf, .xlsx (Excel)");
    m.insert("help.extract.example1", "# Word 문서에서 텍스트 추출\ndox extract -i report.docx");
    m.insert("help.extract.example2", "# JSON 형식으로 메타데이터와 함께 추출\ndox extract -i presentation.pptx --format json --with-metadata");
    
    // === Replace 명령어 상세 도움말 ===
    m.insert("help.replace.yaml_format", "YAML 규칙 파일 형식:\nreplacements:\n  - old: \"기존 텍스트\"\n    new: \"새 텍스트\"\n  - old: \"{{날짜}}\"\n    new: \"2025-09-02\"");
    m.insert("help.replace.example1", "# 단일 파일 치환\ndox replace -r rules.yaml -p document.docx");
    m.insert("help.replace.example2", "# 디렉토리 재귀적 치환 (미리보기)\ndox replace -r rules.yaml -p ./docs --recursive --dry-run");
    m.insert("help.replace.exclude_pattern", "제외 패턴 예시: --exclude \"*.tmp\" --exclude \"backup/*\"");
    
    // === Create 명령어 상세 도움말 ===
    m.insert("help.create.markdown_features", "지원 Markdown 기능:\n• 제목 (# ## ###)\n• 목록 (-, *, 1.)\n• 표\n• 코드 블록\n• 이미지\n• 링크");
    m.insert("help.create.example1", "# Markdown에서 Word 문서 생성\ndox create -f README.md -o output.docx");
    m.insert("help.create.example2", "# 템플릿 사용하여 생성\ndox create -f content.md -o report.docx -t template.docx");
    
    // === Generate 명령어 상세 도움말 ===
    m.insert("help.generate.providers", "AI 제공업체 설정:\n• OpenAI: OPENAI_API_KEY 환경변수 설정\n• Claude: ANTHROPIC_API_KEY 환경변수 설정");
    m.insert("help.generate.content_types", "콘텐츠 유형:\n• blog: 블로그 포스트\n• report: 보고서\n• summary: 요약\n• email: 이메일\n• proposal: 제안서\n• custom: 사용자 정의");
    m.insert("help.generate.example1", "# 블로그 포스트 생성\ndox generate -p \"Rust 프로그래밍 입문\" -t blog");
    m.insert("help.generate.example2", "# GPT-4로 보고서 생성\ndox generate -p \"2025년 시장 분석\" -t report --model gpt-4");
    
    m
});

/// 메시지 가져오기
pub fn get(key: &str) -> &'static str {
    MESSAGES.get(key).copied().unwrap_or("[translation missing]")
}