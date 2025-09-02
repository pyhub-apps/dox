use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// 헤더 메시지 출력
pub fn print_header(message: &str) {
    println!("\n{}", message.bold().blue());
    println!("{}", "=".repeat(message.chars().count()).blue());
}

/// 정보 메시지 출력
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue(), message);
}

/// 성공 메시지 출력
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green(), message.green());
}

/// 경고 메시지 출력
pub fn print_warning(message: &str) {
    eprintln!("{} {}", "⚠".yellow(), message.yellow());
}

/// 오류 메시지 출력
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red(), message.red());
}

/// 프로세스 단계 출력
pub fn print_step(current: usize, total: usize, message: &str) {
    println!(
        "{} [{}/{}] {}",
        "→".cyan(),
        current,
        total,
        message
    );
}

/// Create a progress bar for file processing
pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Create a spinner for long-running operations
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⣾⣽⣻⢿⡿⣟⣯⣷"),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

/// Display a diff between old and new text
pub fn print_diff(old: &str, new: &str, _context_lines: usize) {
    use similar::{ChangeTag, TextDiff};
    
    let diff = TextDiff::from_lines(old, new);
    
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-".red(),
            ChangeTag::Insert => "+".green(),
            ChangeTag::Equal => " ".normal(),
        };
        print!("{} {}", sign, change);
    }
}

/// 사용자 확인 프롬프트
pub fn confirm(message: &str, default: bool) -> bool {
    use dialoguer::Confirm;
    
    let prompt = if message.contains("?") {
        message.to_string()
    } else {
        format!("{}?", message)
    };
    
    Confirm::new()
        .with_prompt(&prompt)
        .default(default)
        .interact()
        .unwrap_or(default)
}

/// 사용자 입력 프롬프트
pub fn prompt(message: &str, default: Option<&str>) -> String {
    use dialoguer::Input;
    
    let prompt = if message.ends_with(':') || message.ends_with('>') {
        message.to_string()
    } else {
        format!("{}: ", message)
    };
    
    let mut input = Input::<String>::new().with_prompt(&prompt);
    
    if let Some(default_value) = default {
        input = input.default(default_value.to_string());
    }
    
    input.interact_text().unwrap_or_default()
}

/// 옵션 목록에서 선택
pub fn select<T: ToString>(message: &str, items: &[T], default: usize) -> usize {
    use dialoguer::Select;
    
    let prompt = if message.ends_with(':') || message.ends_with('>') {
        message.to_string()
    } else {
        format!("{}: ", message)
    };
    
    Select::new()
        .with_prompt(&prompt)
        .items(items)
        .default(default)
        .interact()
        .unwrap_or(default)
}

/// Display a table of data
pub fn print_table(headers: &[&str], rows: Vec<Vec<String>>) {
    use prettytable::{Cell, Row, Table};
    
    let mut table = Table::new();
    
    // Add header
    let header_cells: Vec<Cell> = headers
        .iter()
        .map(|h| Cell::new(h).style_spec("Fb"))
        .collect();
    table.add_row(Row::new(header_cells));
    
    // Add data rows
    for row_data in rows {
        let cells: Vec<Cell> = row_data
            .iter()
            .map(|d| Cell::new(d))
            .collect();
        table.add_row(Row::new(cells));
    }
    
    table.printstd();
}

/// Format file size in human-readable format
pub fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Format duration in human-readable format
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
    }
}