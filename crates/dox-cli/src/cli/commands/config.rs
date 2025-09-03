use anyhow::Result;
use clap::Args;
use std::path::Path;

/// dox 설정 관리
#[derive(Args, Debug)]
pub struct ConfigArgs {
    /// 설정 파일 초기화
    #[arg(long, conflicts_with_all = ["list", "get", "set", "unset", "validate", "show_path", "reset", "edit"])]
    pub init: bool,

    /// 모든 설정 값 나열
    #[arg(long, conflicts_with_all = ["init", "get", "set", "unset", "validate", "show_path", "reset", "edit"])]
    pub list: bool,

    /// 특정 설정 값 가져오기
    #[arg(long, value_name = "키", conflicts_with_all = ["init", "list", "set", "unset", "validate", "show_path", "reset", "edit"])]
    pub get: Option<String>,

    /// 설정 값 지정
    #[arg(long, value_name = "키=값", conflicts_with_all = ["init", "list", "get", "unset", "validate", "show_path", "reset", "edit"])]
    pub set: Option<String>,

    /// 설정 값 제거
    #[arg(long, value_name = "키", conflicts_with_all = ["init", "list", "get", "set", "validate", "show_path", "reset", "edit"])]
    pub unset: Option<String>,

    /// 설정 파일 유효성 검사
    #[arg(long, conflicts_with_all = ["init", "list", "get", "set", "unset", "show_path", "reset", "edit"])]
    pub validate: bool,

    /// 설정 파일 경로 표시
    #[arg(long, conflicts_with_all = ["init", "list", "get", "set", "unset", "validate", "reset", "edit"])]
    pub show_path: bool,

    /// 설정 파일을 기본값으로 재설정
    #[arg(long, conflicts_with_all = ["init", "list", "get", "set", "unset", "validate", "show_path", "edit"])]
    pub reset: bool,

    /// 기본 편집기로 설정 파일 열기
    #[arg(long, conflicts_with_all = ["init", "list", "get", "set", "unset", "validate", "show_path", "reset"])]
    pub edit: bool,
}

/// Helper function to load config with optional custom path
fn load_config_with_path(config_path: Option<&Path>) -> Result<dox_core::utils::config::Config> {
    use dox_core::utils::config::Config;

    if let Some(path) = config_path {
        Config::load_from(path)
    } else {
        Config::load()
    }
}

pub async fn execute(args: ConfigArgs, config_path: Option<&Path>) -> Result<()> {
    use dox_core::utils::{config::Config, ui};

    if args.init {
        ui::print_info("설정 파일을 초기화하는 중...");
        Config::init()?;
        ui::print_success("설정 파일이 성공적으로 초기화되었습니다");
    } else if args.list {
        let config = load_config_with_path(config_path)?;
        ui::print_header("현재 설정");

        // Use colored display if colors are enabled
        if std::env::var("NO_COLOR").is_ok() || !colored::control::SHOULD_COLORIZE.should_colorize()
        {
            println!("{}", config.display());
        } else {
            println!("{}", config.display_colored());
        }
    } else if let Some(key) = args.get {
        let config = load_config_with_path(config_path)?;
        match config.get(&key) {
            Some(value) => println!("{}", value),
            None => ui::print_error(&format!("'{}' 설정 키를 찾을 수 없습니다", key)),
        }
    } else if let Some(key_value) = args.set {
        let parts: Vec<&str> = key_value.splitn(2, '=').collect();
        if parts.len() != 2 {
            ui::print_error("잘못된 형식입니다. 사용법: --set 키=값");
            return Ok(());
        }

        let mut config = load_config_with_path(config_path)?;
        config.set(parts[0], parts[1])?;
        config.save()?;
        ui::print_success(&format!("{} = {} 설정됨", parts[0], parts[1]));
    } else if let Some(key) = args.unset {
        let mut config = load_config_with_path(config_path)?;
        config.unset(&key)?;
        config.save()?;
        ui::print_success(&format!("'{}' 설정 키가 제거되었습니다", key));
    } else if args.validate {
        match load_config_with_path(config_path) {
            Ok(_) => ui::print_success("설정 파일이 유효합니다"),
            Err(err) => {
                ui::print_error(&format!("설정 파일 유효성 검사 실패: {}", err));
                return Err(err);
            }
        }
    } else if args.show_path {
        if let Some(path) = config_path {
            println!("{}", path.display());
        } else {
            match Config::default_path() {
                Ok(path) => println!("{}", path.display()),
                Err(err) => {
                    ui::print_error(&format!(
                        "기본 설정 파일 경로를 확인할 수 없습니다: {}",
                        err
                    ));
                    return Err(err);
                }
            }
        }
    } else if args.reset {
        ui::print_info("설정 파일을 기본값으로 재설정하는 중...");
        Config::init()?;
        ui::print_success("설정 파일이 기본값으로 재설정되었습니다");
    } else if args.edit {
        let path = if let Some(path) = config_path {
            path.to_path_buf()
        } else {
            Config::default_path()?
        };

        // 설정 파일이 없으면 생성
        if !path.exists() {
            ui::print_info("설정 파일이 존재하지 않아 초기화합니다...");
            Config::init()?;
        }

        // 시스템 기본 편집기로 파일 열기
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else if cfg!(target_os = "macos") {
                "open".to_string()
            } else {
                "nano".to_string()
            }
        });

        let mut cmd = std::process::Command::new(&editor);
        cmd.arg(&path);

        if editor == "open" {
            cmd.arg("-t"); // macOS에서 텍스트 편집기로 열기
        }

        match cmd.status() {
            Ok(status) if status.success() => {
                ui::print_success("편집기에서 설정 파일을 열었습니다");
            }
            Ok(_) => {
                ui::print_error("편집기를 실행했지만 오류가 발생했습니다");
            }
            Err(err) => {
                ui::print_error(&format!("편집기 실행 실패: {}", err));
                ui::print_info(&format!(
                    "수동으로 설정 파일을 편집하세요: {}",
                    path.display()
                ));
            }
        }
    } else {
        ui::print_info("--help를 사용하여 사용 가능한 설정 명령어를 확인하세요");
    }

    Ok(())
}
