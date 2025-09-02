use anyhow::Result;
use clap::Args;

/// dox 설정 관리
#[derive(Args, Debug)]
pub struct ConfigArgs {
    /// 설정 파일 초기화
    #[arg(long, conflicts_with_all = ["list", "get", "set", "unset"])]
    pub init: bool,
    
    /// 모든 설정 값 나열
    #[arg(long, conflicts_with_all = ["init", "get", "set", "unset"])]
    pub list: bool,
    
    /// 특정 설정 값 가져오기
    #[arg(long, value_name = "키", conflicts_with_all = ["init", "list", "set", "unset"])]
    pub get: Option<String>,
    
    /// 설정 값 지정
    #[arg(long, value_name = "키=값", conflicts_with_all = ["init", "list", "get", "unset"])]
    pub set: Option<String>,
    
    /// 설정 값 제거
    #[arg(long, value_name = "키", conflicts_with_all = ["init", "list", "get", "set"])]
    pub unset: Option<String>,
}

pub async fn execute(args: ConfigArgs) -> Result<()> {
    use dox_core::utils::{config::Config, ui};
    
    if args.init {
        ui::print_info("설정 파일을 초기화하는 중...");
        Config::init()?;
        ui::print_success("설정 파일이 성공적으로 초기화되었습니다");
    } else if args.list {
        let config = Config::load()?;
        ui::print_header("현재 설정");
        println!("{}", config.display());
    } else if let Some(key) = args.get {
        let config = Config::load()?;
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
        
        let mut config = Config::load()?;
        config.set(parts[0], parts[1])?;
        config.save()?;
        ui::print_success(&format!("{} = {} 설정됨", parts[0], parts[1]));
    } else if let Some(key) = args.unset {
        let mut config = Config::load()?;
        config.unset(&key)?;
        config.save()?;
        ui::print_success(&format!("'{}' 설정 키가 제거되었습니다", key));
    } else {
        ui::print_info("--help를 사용하여 사용 가능한 설정 명령어를 확인하세요");
    }
    
    Ok(())
}