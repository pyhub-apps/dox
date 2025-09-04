# dox - 문서 자동화 CLI 🚀 (Rust 에디션)

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat&logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-Proprietary-red.svg)](LICENSE)
[![CI](https://github.com/pyhub-apps/dox/workflows/CI/badge.svg)](https://github.com/pyhub-apps/dox/actions)

문서 자동화, 텍스트 치환, AI 기반 콘텐츠 생성을 위한 강력한 CLI 도구입니다. Word/PowerPoint/Excel/PDF 문서를 아름다운 진행 표시와 컬러 출력으로 효율적으로 처리합니다.

> 🎉 **최신 업데이트**: Extract 명령어에 Excel 지원, 병렬 처리, 배치 기능이 추가되었습니다!

> **참고**: 이 프로젝트는 원본 [dox-golang](https://github.com/pyhub-kr/pyhub-documents-cli) 프로젝트의 Rust 포팅 버전으로, 향상된 성능과 더 작은 바이너리 크기를 제공합니다.

## ✨ 주요 기능

### 🔄 스마트 텍스트 치환
- 여러 Word (.docx), PowerPoint (.pptx), 텍스트 (.txt) 파일에서 텍스트 일괄 치환
- **🤖 AI 스마트 교체**: 컨텍스트 인식으로 더 자연스러운 교체
- 쉬운 관리를 위한 YAML 기반 규칙 설정
- 단일 텍스트 교체 모드 지원 (`--find/--to`)
- 패턴 제외 기능을 포함한 재귀적 디렉토리 처리
- 향상된 성능을 위한 병렬 처리
- 수정 전 자동 백업 생성
- 파일 잠금 상태 자동 감지 및 보호

### 📝 문서 생성
- Markdown 파일을 Word 또는 PowerPoint 문서로 변환
- 템플릿 기반 문서 생성
- 스타일 및 서식 보존
- 복잡한 문서 구조 지원

### 🤖 AI 콘텐츠 생성
- OpenAI (GPT) 또는 Claude AI를 사용한 콘텐츠 생성
- 다양한 콘텐츠 유형: 블로그, 보고서, 요약, 이메일, 제안서
- 최신 모델 지원: GPT-4, Claude 3

### 📋 템플릿 처리
- 플레이스홀더가 포함된 Word/PowerPoint 템플릿 처리
- YAML/JSON 기반 값 주입
- 복잡한 데이터 구조 지원

### 📊 텍스트 추출 🆕
- **다중 형식 지원**: Word, PowerPoint, PDF, Excel 문서에서 텍스트 추출
- **배치 처리**: 디렉토리 전체 문서를 한 번에 처리
- **병렬 처리**: 다중 파일을 동시에 처리하여 속도 향상
- **다양한 출력 형식**: 텍스트, JSON, Markdown 지원
- **메타데이터 추출**: 문서 정보 포함 옵션
- **고급 필터링**: glob 패턴으로 파일 제외 기능

## 🌏 한글 지원

dox는 기본적으로 **한글 인터페이스**를 제공합니다. 모든 명령어 도움말, 오류 메시지, 진행 상황이 한글로 표시됩니다.

```bash
$ dox --help
문서 자동화 및 AI 기반 콘텐츠 생성 CLI

사용법: dox [옵션] <명령어>

명령어:
  replace   YAML 규칙 파일을 사용하여 문서의 텍스트 치환
  create    Markdown 파일에서 문서 생성
  template  플레이스홀더가 포함된 문서 템플릿 처리
  generate  AI를 사용하여 콘텐츠 생성
  extract   문서에서 텍스트 추출
  config    설정 관리
```

## 📦 설치

### 사전 빌드된 바이너리

[Releases](https://github.com/pyhub-apps/dox/releases) 페이지에서 사용 중인 플랫폼용 최신 릴리즈를 다운로드하세요.

### 소스에서 빌드

```bash
# 저장소 클론
git clone https://github.com/pyhub-apps/dox.git
cd dox

# 릴리즈 버전 빌드
cargo build --release

# 전역 설치
cargo install --path .

# 또는 직접 실행
./target/release/dox
```

## 🚀 빠른 시작

### 텍스트 치환

```bash
# 규칙 파일 생성 (rules.yml)
cat > rules.yml << EOF
replacements:
  - old: "2023년"
    new: "2024년"
  - old: "버전 1.0"
    new: "버전 2.0"
  - old: "Hello"
    new: "안녕하세요"
EOF

# 단일 파일 치환
dox replace -r rules.yml -p document.docx

# 디렉토리 재귀적 치환
dox replace -r rules.yml -p ./docs --recursive

# 미리보기 모드 (실제 변경 없음)
dox replace -r rules.yml -p ./docs --dry-run

# 백업 파일 생성
dox replace -r rules.yml -p ./docs --backup

# 병렬 처리 (빠른 처리)
dox replace -r rules.yml -p ./docs --concurrent --max-workers 8

# 차이점 표시
dox replace -r rules.yml -p ./docs --show-diff --dry-run

# 특정 파일 제외
dox replace -r rules.yml -p . --exclude "*.tmp" --exclude "backup/*"

# 진행률 표시와 함께 실행
dox replace -r rules.yml -p ./large-project --concurrent --verbose
```

#### 🤖 AI 스마트 교체 기능 (신규!)

AI가 문서의 컨텍스트를 분석하여 더 자연스럽고 정확한 교체를 제공합니다.

```bash
# API 키 설정 (OpenAI 또는 Claude)
export OPENAI_API_KEY="your-openai-key"
# 또는
export ANTHROPIC_API_KEY="your-claude-key"

# 단일 텍스트 AI 스마트 교체
dox replace --find "{{이름}}" --to "홍길동" -p document.docx \
  --ai-smart --ai-context "한국 전통 소설의 주인물"

# YAML 규칙과 AI 결합 (각 규칙을 AI가 개별 분석)
dox replace -r rules.yml -p presentation.pptx \
  --ai-smart --ai-context "공식 사업 계획서" --ai-model "gpt-4"

# Claude AI 사용
dox replace --find "placeholder text" --to "enhanced content" \
  -p report.docx --ai-smart --ai-model "claude-3-sonnet" \
  --ai-context "기술 문서"
```

**AI 스마트 교체의 장점:**
- **컨텍스트 인식**: 문서 주변 텍스트를 분석하여 적절한 교체
- **자연스러운 표현**: 문법과 어조에 맞는 자연스러운 표현으로 변환
- **스타일 일관성**: 문서의 전체적인 스타일과 톤 유지
- **지능적 서식**: 따옴표, 대소문자, 구두점 등을 컨텍스트에 맞게 조정

**예시 결과:**
```
# 일반 교체
{{이름}} → 홍길동

# AI 스마트 교체 (컨텍스트: "공식 문서")  
{{이름}} → "홍길동"  (따옴표 자동 추가)

# AI 스마트 교체 (컨텍스트: "기술 보고서")
placeholder → "고급 기능"  (더 적절한 표현으로 변환)
```

#### 단일 텍스트 교체 모드

YAML 파일 없이 간단한 교체도 가능합니다:

```bash
# 기본 단일 교체
dox replace --find "old text" --to "new text" -p document.docx

# AI와 함께 단일 교체
dox replace --find "{{제목}}" --to "프로젝트 현황" -p report.docx \
  --ai-smart --ai-context "월간 업무 보고서"

# 여러 파일에 동일한 교체 적용
dox replace --find "version 1.0" --to "version 2.0" -p ./docs \
  --recursive --backup
```

#### 파일 잠금 보호 🔒

Office 문서가 다른 프로그램에서 열려있을 때 안전하게 처리합니다:

```bash
# 자동 감지 및 경고
$ dox replace -r rules.yml -p document.docx
❌ 파일 접근 오류: 파일이 다른 프로그램에서 사용 중입니다. 먼저 파일을 닫아주세요

# 임시 파일 자동 필터링  
$ dox replace -r rules.yml -p ./
# ~$Document.docx 같은 Office 임시 파일은 자동으로 제외됩니다
```

### 📄 문서 생성 (Create 명령어) ✨

강력한 Markdown to Office 변환기로 Markdown 파일을 전문적인 Word 문서나 PowerPoint 프레젠테이션으로 변환합니다.

#### 🎯 주요 기능
- **완전한 Office Open XML 지원**: 표준 준수로 Microsoft Office와 완벽 호환
- **한글 폰트 지원**: 맑은 고딕을 기본으로 한 아름다운 한글 문서
- **자동 포맷 감지**: 파일 확장자로 출력 형식 자동 결정
- **YAML Frontmatter 지원**: 문서 메타데이터 (제목, 저자, 날짜) 자동 처리
- **고급 Markdown 지원**: 제목, 단락, 코드 블록, 목록, 인용문 등 모든 요소

#### 기본 사용법

```bash
# Markdown을 Word 문서로 변환
dox create --from report.md --output report.docx

# Markdown을 PowerPoint 프레젠테이션으로 변환
dox create --from presentation.md --output slides.pptx

# 출력 형식 명시적 지정
dox create -f content.md -o document --format docx

# 기존 파일 강제 덮어쓰기
dox create -f content.md -o report.docx --force
```

#### 📋 Markdown 예시

Create 명령어는 다양한 Markdown 요소를 지원합니다:

```markdown
---
title: "프로젝트 보고서"
author: "홍길동"  
date: "2024-09-04"
---

# 프로젝트 개요

이 문서는 **dox Create 명령어**의 기능을 소개합니다.

## 주요 특징

### 코드 블록 지원
```rust
fn main() {
    println!("Hello, dox!");
}
```

### 목록 지원
- 순서 없는 목록
- **강조된 텍스트**
- *기울임꼴 텍스트*

1. 첫 번째 항목
2. 두 번째 항목
3. 세 번째 항목

### 인용문
> "dox는 문서 자동화의 새로운 표준입니다."

---

## 결론
Create 명령어로 전문적인 Office 문서를 쉽게 만드세요!
```

#### 📊 변환 결과

**Word 문서 (.docx)**:
- 제목 스타일과 계층 구조 자동 적용
- 한글 친화적 폰트 설정 (맑은 고딕)
- 코드 블록은 Courier New 폰트로 처리
- 목록과 인용문 스타일 자동 변환

**PowerPoint 프레젠테이션 (.pptx)**:
- 제목 슬라이드 자동 생성
- 섹션별 내용 구조화
- 한글 텍스트 최적화
- 프로페셔널한 테마 적용

#### 실제 활용 사례

```bash
# 기술 문서 작성
dox create -f api-guide.md -o API가이드.docx

# 회의 자료 생성  
dox create -f meeting-notes.md -o 회의자료.pptx

# 보고서 배치 생성
for md in reports/*.md; do
  name=$(basename "$md" .md)
  dox create -f "$md" -o "outputs/${name}.docx"
done

# 프레젠테이션 제작 파이프라인
dox create -f slides.md -o presentation.pptx --force
echo "✅ 프레젠테이션이 생성되었습니다: presentation.pptx"
```

#### 🛠️ 고급 옵션

```bash
# 상세한 진행 상황 표시
dox create -f large-document.md -o output.docx -v

# 템플릿 사용 (향후 지원 예정)
dox create -f content.md -o report.docx --template company-template.docx

# 배치 처리와 함께 사용
find . -name "*.md" -exec dox create -f {} -o {}.docx \;
```

### AI 콘텐츠 생성

```bash
# API 키 설정
export OPENAI_API_KEY="your-key"
# 또는
export ANTHROPIC_API_KEY="your-key"

# 콘텐츠 생성
dox generate -p "Rust 프로그래밍 입문" -t blog -o blog.md

# GPT-4로 보고서 생성
dox generate -p "2025년 시장 분석" -t report --model gpt-4
```

### 📊 텍스트 추출 (신규 업데이트!) 

#### 지원 파일 형식
- **Word** (.docx), **PowerPoint** (.pptx), **PDF** (.pdf)
- **Excel** (.xlsx) ← 새로 추가! 🆕

#### 기본 사용법

```bash
# 단일 파일에서 텍스트 추출
dox extract -i report.docx
dox extract -i spreadsheet.xlsx  # Excel 지원!
dox extract -i presentation.pptx
dox extract -i document.pdf

# 출력 형식 선택
dox extract -i report.docx --format text      # 일반 텍스트 (기본값)
dox extract -i report.docx --format json      # JSON 형식 
dox extract -i report.docx --format markdown  # 마크다운 형식

# 메타데이터 포함
dox extract -i document.pdf --format json --with-metadata

# 파일로 저장
dox extract -i presentation.pptx -o output.txt
dox extract -i spreadsheet.xlsx -o data.json --format json
```

#### 디렉토리 배치 처리 🚀

```bash
# 디렉토리 전체 문서 처리
dox extract -i ./documents

# 하위 폴더까지 재귀 처리 (기본값)
dox extract -i ./project --recursive

# 특정 파일 제외
dox extract -i ./documents --exclude "*.tmp"
dox extract -i ./project --exclude "backup/*" --exclude "~$*"

# 출력 디렉토리 지정
dox extract -i ./documents --output-dir ./extracted

# 개별 파일명으로 저장
dox extract -i ./documents --format json --output-dir ./results
# 결과: report.json, presentation.json, spreadsheet.json 등
```

#### 고성능 병렬 처리 ⚡

```bash
# 병렬 처리 활성화 (빠른 속도)
dox extract -i ./large-project --concurrent

# 워커 수 조정 (기본값: 4)
dox extract -i ./documents --concurrent --max-workers 8

# 진행률 표시와 함께
dox extract -i ./big-directory --concurrent -v
```

#### 실제 사용 시나리오

```bash
# 회계 자료에서 데이터 추출
dox extract -i ./accounting/*.xlsx --format json --output-dir ./data

# 보고서 모음에서 텍스트만 추출
dox extract -i ./reports --exclude "temp/*" --format text

# 프레젠테이션 내용을 마크다운으로 변환
dox extract -i ./slides --format markdown --output-dir ./md-files

# 대용량 문서 폴더를 병렬로 빠르게 처리
dox extract -i ./all-documents --concurrent --max-workers 8 \
  --exclude "*.tmp" --exclude "~$*" --format json --output-dir ./extracted
```

#### Excel 파일 특별 기능 📈

Excel 파일 처리 시 특별한 기능들:

```bash
# Excel 파일에서 모든 시트 내용 추출
dox extract -i budget.xlsx

# JSON으로 시트별 구조화된 데이터 얻기
dox extract -i financial-report.xlsx --format json --with-metadata

# 여러 Excel 파일을 일괄 처리
dox extract -i ./spreadsheets --concurrent --output-dir ./csv-data
```

출력 형태:
```
=== Sheet1 ===
항목    1월    2월    3월
매출    1000   1200   1100
비용    800    900    850

=== Summary ===
총계    200    300    250
```

### 템플릿 처리

```bash
# 값 파일과 함께 템플릿 처리
dox template -t template.docx -o result.docx --values data.yaml

# 개별 값 설정
dox template -t template.pptx -o result.pptx --set "name=홍길동" --set "date=2025-09-02"
```

### ⚙️ 설정 관리

dox는 다층적인 설정 시스템을 제공하여 사용성을 극대화합니다.

#### 우선순위
1. **CLI 플래그** (최우선)
2. **사용자 지정 설정 파일** (`--config` 플래그)
3. **기본 설정 파일**
4. **환경변수**
5. **기본값**

#### 설정 파일 위치
- **macOS**: `~/Library/Application Support/dox/config.toml`
- **Linux**: `~/.config/dox/config.toml` 
- **Windows**: `%APPDATA%/dox/config.toml`

#### 기본 설정 관리

```bash
# 설정 파일 초기화
dox config --init

# 현재 설정 보기
dox config --list

# 특정 값 조회
dox config --get global.lang
dox config --get openai.api_key

# 값 설정
dox config --set global.verbose=true
dox config --set openai.api_key=sk-xxx
dox config --set generate.model=gpt-4

# 값 제거
dox config --unset openai.api_key
```

#### 사용자 정의 설정 파일

```bash
# 특정 설정 파일 사용
dox --config ~/work/dox-work.toml generate -p "업무 보고서"

# 프로젝트별 설정
dox --config ./project-config.toml replace -r rules.yml -p ./docs
```

#### 설정 예시 (config.toml)

```toml
[global]
verbose = false
quiet = false
lang = "ko"
no_color = false

[replace]
backup = true
recursive = true
concurrent = true
max_workers = 4

[extract]
format = "text"
with_metadata = false
recursive = true
concurrent = false
max_workers = 4

[generate]
model = "gpt-3.5-turbo"
max_tokens = 2000
temperature = 0.7
content_type = "blog"

[openai]
api_key = "sk-your-openai-key"
model = "gpt-4"

[claude]
api_key = "sk-ant-your-claude-key"
model = "claude-3-sonnet"
```

#### CLI 플래그와 설정 파일 통합

```bash
# 설정 파일에서 verbose=false이지만, CLI 플래그가 우선
dox -v config --list  # 상세 출력으로 실행

# 설정 파일에서 quiet=true이지만, CLI 플래그가 우선
dox -v generate -p "테스트"  # 여전히 상세 출력

# 사용자 정의 설정과 CLI 플래그 조합
dox --config ~/quiet-config.toml -v extract -i doc.pdf  # verbose 우선
```

## 🔧 개발 현황

이 Rust 포팅 버전은 현재 활발히 개발 중입니다. 다음 기능들이 구현되고 있습니다:

- [x] 프로젝트 설정 및 기본 구조
- [x] 멀티 크레이트 아키텍처 마이그레이션
- [x] 한글 메시지 지원 및 i18n 시스템
- [x] HeadVer 버전 관리 시스템
- [x] GitHub Actions 릴리즈 자동화
- [x] Replace 명령어 (AI 스마트 교체, 파일 잠금 보호 포함)
- [x] Create 명령어 ✨ (Markdown을 Word/PowerPoint로 변환, Office Open XML 지원)
- [ ] Template 명령어
- [x] Generate 명령어 (OpenAI, Claude AI 통합, 한국어 최적화)
- [x] Extract 명령어 ✨ (Excel 지원, 병렬 처리, 배치 기능)
- [x] 설정 관리

## 📋 지원 파일 형식

- **문서**: .docx (Word), .pptx (PowerPoint), .pdf, .xlsx (Excel), .txt (텍스트)
- **입력**: .md (Markdown), .yaml/.yml (YAML), .json (JSON)
- **출력**: text, json, markdown

## 🛠️ 빌드 정보

- **실행 파일 경로**: `./target/release/dox`
- **바이너리 크기**: 약 3.1MB (최적화된 릴리즈 빌드)
- **최소 Rust 버전**: 1.75+

## 🤝 기여하기

기여를 환영합니다! Pull Request를 자유롭게 제출해 주세요.

### 개발 환경 설정

```bash
# 개발 모드로 빌드
cargo build

# 테스트 실행
cargo test

# 코드 포맷팅
cargo fmt

# 린트 검사
cargo clippy
```

## 📝 라이선스

이 소프트웨어는 독점 상업용 라이선스로 보호됩니다. 모든 권리는 PyHub Korea에 있습니다.

상업적 사용을 위해서는 별도의 라이선스 계약이 필요합니다.
자세한 내용은 [LICENSE](LICENSE) 파일을 참조하거나 me@pyhub.kr로 문의하시기 바랍니다.

## 🙏 감사의 말

- 원본 [dox-golang](https://github.com/pyhub-kr/pyhub-documents-cli) 프로젝트
- [PyHub Korea](https://pyhub.kr) 팀
- 모든 기여자들

## 📞 문의

- **이슈**: [GitHub Issues](https://github.com/pyhub-apps/dox/issues)
- **이메일**: me@pyhub.kr