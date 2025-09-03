pub mod messages;

use once_cell::sync::Lazy;
use std::sync::RwLock;

/// 현재 언어 설정 (기본값: 한국어)
static CURRENT_LANG: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new("ko".to_string()));

/// 언어 설정
pub fn set_language(lang: &str) {
    if let Ok(mut current) = CURRENT_LANG.write() {
        *current = lang.to_string();
    }
}

/// 현재 언어 가져오기
pub fn get_language() -> String {
    CURRENT_LANG
        .read()
        .map(|lang| lang.clone())
        .unwrap_or_else(|_| "ko".to_string())
}

/// 메시지 가져오기 (현재는 한글만 지원)
pub fn t(key: &str) -> &'static str {
    messages::get(key)
}
