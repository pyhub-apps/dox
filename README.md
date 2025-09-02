# dox - 문서 자동화 CLI 🚀 (Rust 에디션)

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat&logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/pyhub-apps/dox/workflows/CI/badge.svg)](https://github.com/pyhub-apps/dox/actions)

문서 자동화, 텍스트 치환, AI 기반 콘텐츠 생성을 위한 강력한 CLI 도구입니다. Word/PowerPoint 문서를 아름다운 진행 표시와 컬러 출력으로 효율적으로 처리합니다.

> **참고**: 이 프로젝트는 원본 [dox-golang](https://github.com/pyhub-kr/pyhub-documents-cli) 프로젝트의 Rust 포팅 버전으로, 향상된 성능과 더 작은 바이너리 크기를 제공합니다.

## ✨ 주요 기능

### 🔄 대량 텍스트 치환
- 여러 Word (.docx) 및 PowerPoint (.pptx) 파일에서 텍스트 일괄 치환
- 쉬운 관리를 위한 YAML 기반 규칙 설정
- 패턴 제외 기능을 포함한 재귀적 디렉토리 처리
- 향상된 성능을 위한 병렬 처리
- 수정 전 자동 백업 생성

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

### 📊 텍스트 추출
- Word, PowerPoint, PDF, Excel 문서에서 텍스트 추출
- 다양한 출력 형식 지원 (텍스트, JSON, Markdown)
- 메타데이터 포함 옵션

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
EOF

# 문서에서 치환
dox replace -r rules.yml -p ./docs

# 미리보기 모드로 실행
dox replace -r rules.yml -p ./docs --dry-run

# 특정 파일 제외
dox replace -r rules.yml -p . --exclude "*.tmp" --exclude "backup/*"
```

### 문서 생성

```bash
# Markdown을 Word로 변환
dox create -f report.md -o report.docx

# Markdown을 PowerPoint로 변환
dox create -f presentation.md -o slides.pptx

# 템플릿 사용
dox create -f content.md -o report.docx -t template.docx
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

### 텍스트 추출

```bash
# Word 문서에서 텍스트 추출
dox extract -i report.docx

# JSON 형식으로 메타데이터와 함께 추출
dox extract -i presentation.pptx --format json --with-metadata

# 추출 결과를 파일로 저장
dox extract -i document.pdf -o output.txt
```

### 템플릿 처리

```bash
# 값 파일과 함께 템플릿 처리
dox template -t template.docx -o result.docx --values data.yaml

# 개별 값 설정
dox template -t template.pptx -o result.pptx --set "name=홍길동" --set "date=2025-09-02"
```

## 🔧 개발 현황

이 Rust 포팅 버전은 현재 활발히 개발 중입니다. 다음 기능들이 구현되고 있습니다:

- [x] 프로젝트 설정 및 기본 구조
- [x] 멀티 크레이트 아키텍처 마이그레이션
- [x] 한글 메시지 지원 및 i18n 시스템
- [x] HeadVer 버전 관리 시스템
- [x] GitHub Actions 릴리즈 자동화
- [ ] Replace 명령어 (진행 중)
- [ ] Create 명령어
- [ ] Template 명령어
- [ ] Generate 명령어 (AI 통합)
- [ ] Extract 명령어
- [x] 설정 관리

## 📋 지원 파일 형식

- **문서**: .docx (Word), .pptx (PowerPoint), .pdf, .xlsx (Excel)
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

이 프로젝트는 MIT 라이선스 하에 배포됩니다 - 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

## 🙏 감사의 말

- 원본 [dox-golang](https://github.com/pyhub-kr/pyhub-documents-cli) 프로젝트
- [PyHub Korea](https://pyhub.kr) 팀
- 모든 기여자들

## 📞 문의

- **이슈**: [GitHub Issues](https://github.com/pyhub-apps/dox/issues)
- **이메일**: support@pyhub.kr