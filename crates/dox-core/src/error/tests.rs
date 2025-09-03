#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::*;

    #[test]
    fn test_error_creation() {
        let err = DoxError::file_not_found("/path/to/file");
        assert!(matches!(err, DoxError::FileNotFound { .. }));
        assert_eq!(err.code(), ErrorCode::FileNotFound);
    }

    #[test]
    fn test_error_context() {
        let err = DoxError::IoError {
            message: "Failed to read".to_string(),
        };
        let err_with_context = err.with_context("While processing config file");

        match err_with_context {
            DoxError::IoError { message } => {
                assert!(message.contains("While processing config file"));
                assert!(message.contains("Failed to read"));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_recovery_check() {
        let recoverable = DoxError::NetworkError {
            message: "Connection timeout".to_string(),
        };
        assert!(recoverable.is_recoverable());

        let non_recoverable = DoxError::FileNotFound {
            path: "/path/to/file".into(),
        };
        assert!(!non_recoverable.is_recoverable());
    }

    #[test]
    fn test_user_friendly_messages() {
        let err = DoxError::MissingApiKey {
            provider: "OpenAI".to_string(),
        };
        let message = err.user_message();

        assert!(message.contains("Missing API key for OpenAI"));
        assert!(message.contains("Suggestion:"));
        assert!(message.contains("dox config api-key"));
    }

    #[test]
    fn test_error_codes() {
        let errors = vec![
            (DoxError::file_not_found("test"), ErrorCode::FileNotFound),
            (
                DoxError::permission_denied("test"),
                ErrorCode::PermissionDenied,
            ),
            (DoxError::config("test"), ErrorCode::ConfigError),
            (DoxError::missing_api_key("test"), ErrorCode::MissingApiKey),
        ];

        for (error, expected_code) in errors {
            assert_eq!(error.code(), expected_code);
        }
    }

    #[test]
    fn test_error_conversion_from_io() {
        use std::io;

        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let dox_err: DoxError = io_err.into();

        match dox_err {
            DoxError::IoError { message } => {
                assert!(message.contains("not found"));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_conversion_from_json() {
        let json = r#"{ invalid json }"#;
        let result: Result<serde_json::Value, DoxError> =
            serde_json::from_str(json).map_err(Into::into);

        assert!(result.is_err());
        match result.unwrap_err() {
            DoxError::ParseError { message } => {
                assert!(message.contains("JSON parse error"));
            }
            _ => panic!("Wrong error type"),
        }
    }
}
