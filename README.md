# dox - Document Automation CLI 🚀 (Rust Edition)

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat&logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/pyhub-apps/dox/workflows/CI/badge.svg)](https://github.com/pyhub-apps/dox/actions)

A powerful CLI tool for document automation, text replacement, and AI-powered content generation. Process Word/PowerPoint documents efficiently with beautiful progress tracking and colored output.

> **Note**: This is a Rust port of the original [dox-golang](https://github.com/pyhub-kr/pyhub-documents-cli) project, offering improved performance and smaller binary size.

## ✨ Features

### 🔄 Bulk Text Replacement
- Replace text across multiple Word (.docx) and PowerPoint (.pptx) files
- YAML-based rule configuration for easy management
- Recursive directory processing with pattern exclusion
- Concurrent processing for improved performance
- Automatic backup creation before modifications

### 📝 Document Creation
- Convert Markdown files to Word or PowerPoint documents
- Template-based document generation
- Style and format preservation
- Support for complex document structures

### 🤖 AI Content Generation
- Generate content using OpenAI (GPT) or Claude AI
- Multiple content types: blogs, reports, summaries, emails, proposals
- Support for latest models: GPT-4, Claude 3

### 📋 Template Processing
- Process Word/PowerPoint templates with placeholders
- YAML/JSON-based value injection
- Support for complex data structures

## 📦 Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/pyhub-apps/dox/releases) page.

### Build from Source

```bash
# Clone the repository
git clone https://github.com/pyhub-apps/dox.git
cd dox

# Build release version
cargo build --release

# Install globally
cargo install --path .
```

## 🚀 Quick Start

### Text Replacement

```bash
# Create a rules file (rules.yml)
cat > rules.yml << EOF
- old: "2023"
  new: "2024"
- old: "Version 1.0"
  new: "Version 2.0"
EOF

# Replace in documents
dox replace --rules rules.yml --path ./docs
```

### Document Creation

```bash
# Convert Markdown to Word
dox create --from report.md --output report.docx

# Convert Markdown to PowerPoint
dox create --from presentation.md --output slides.pptx
```

### AI Content Generation

```bash
# Set API key
export OPENAI_API_KEY="your-key"

# Generate content
dox generate --type blog --prompt "Rust best practices" --output blog.md
```

## 🔧 Development Status

This Rust port is currently under active development. The following features are being implemented:

- [x] Project setup and basic structure
- [ ] Replace command (in progress)
- [ ] Create command
- [ ] Template command
- [ ] Generate command
- [ ] Extract command
- [ ] Configuration management
- [ ] Internationalization (i18n)

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Original [dox-golang](https://github.com/pyhub-kr/pyhub-documents-cli) project
- [PyHub Korea](https://pyhub.kr) team