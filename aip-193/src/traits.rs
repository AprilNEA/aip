use std::collections::HashMap;

use crate::{Code, ErrorInfo, Status, StatusDetails};

/// Trait for converting custom error types to Status
pub trait IntoStatus {
    /// The error code
    fn code(&self) -> Code;

    /// Human-readable error message
    fn message(&self) -> String;

    /// Error reason in UPPER_SNAKE_CASE
    fn reason(&self) -> &str;

    /// Error domain (e.g., "myapp.example.com")
    fn domain(&self) -> &str;

    /// Optional metadata
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

impl<T: IntoStatus> From<T> for Status {
    fn from(err: T) -> Self {
        Status {
            code: err.code().into(),
            message: err.message(),
            details: StatusDetails {
                error_info: Some(ErrorInfo {
                    reason: err.reason().to_string(),
                    domain: err.domain().to_string(),
                    metadata: err.metadata(),
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ Test Error Types ============

    /// A simple error type for testing
    #[derive(Debug, Clone)]
    struct SimpleError {
        kind: ErrorKind,
    }

    #[derive(Debug, Clone, Copy)]
    enum ErrorKind {
        NotFound,
        InvalidArgument,
        PermissionDenied,
    }

    impl IntoStatus for SimpleError {
        fn code(&self) -> Code {
            match self.kind {
                ErrorKind::NotFound => Code::NotFound,
                ErrorKind::InvalidArgument => Code::InvalidArgument,
                ErrorKind::PermissionDenied => Code::PermissionDenied,
            }
        }

        fn message(&self) -> String {
            match self.kind {
                ErrorKind::NotFound => "Resource not found".to_string(),
                ErrorKind::InvalidArgument => "Invalid argument provided".to_string(),
                ErrorKind::PermissionDenied => "Permission denied".to_string(),
            }
        }

        fn reason(&self) -> &str {
            match self.kind {
                ErrorKind::NotFound => "RESOURCE_NOT_FOUND",
                ErrorKind::InvalidArgument => "INVALID_ARGUMENT",
                ErrorKind::PermissionDenied => "PERMISSION_DENIED",
            }
        }

        fn domain(&self) -> &str {
            "test.example.com"
        }
    }

    /// An error type with metadata for testing
    #[derive(Debug, Clone)]
    struct ErrorWithMetadata {
        user_id: String,
        resource_id: String,
    }

    impl IntoStatus for ErrorWithMetadata {
        fn code(&self) -> Code {
            Code::NotFound
        }

        fn message(&self) -> String {
            format!("User {} cannot access resource {}", self.user_id, self.resource_id)
        }

        fn reason(&self) -> &str {
            "ACCESS_DENIED"
        }

        fn domain(&self) -> &str {
            "authz.example.com"
        }

        fn metadata(&self) -> HashMap<String, String> {
            let mut map = HashMap::new();
            map.insert("user_id".to_string(), self.user_id.clone());
            map.insert("resource_id".to_string(), self.resource_id.clone());
            map
        }
    }

    // ============ IntoStatus Trait Tests ============

    #[test]
    fn test_into_status_simple_error() {
        let err = SimpleError {
            kind: ErrorKind::NotFound,
        };

        assert_eq!(err.code(), Code::NotFound);
        assert_eq!(err.message(), "Resource not found");
        assert_eq!(err.reason(), "RESOURCE_NOT_FOUND");
        assert_eq!(err.domain(), "test.example.com");
        assert!(err.metadata().is_empty());
    }

    #[test]
    fn test_into_status_with_metadata() {
        let err = ErrorWithMetadata {
            user_id: "user_123".to_string(),
            resource_id: "doc_456".to_string(),
        };

        let metadata = err.metadata();
        assert_eq!(metadata.get("user_id"), Some(&"user_123".to_string()));
        assert_eq!(metadata.get("resource_id"), Some(&"doc_456".to_string()));
    }

    #[test]
    fn test_into_status_default_metadata() {
        let err = SimpleError {
            kind: ErrorKind::InvalidArgument,
        };

        // Default implementation should return empty HashMap
        let metadata = err.metadata();
        assert!(metadata.is_empty());
    }

    // ============ From<T: IntoStatus> for Status Tests ============

    #[test]
    fn test_from_simple_error_to_status() {
        let err = SimpleError {
            kind: ErrorKind::NotFound,
        };

        let status: Status = err.into();

        assert_eq!(status.code, 5); // NOT_FOUND = 5
        assert_eq!(status.message, "Resource not found");

        let error_info = status.details.error_info.as_ref().unwrap();
        assert_eq!(error_info.reason, "RESOURCE_NOT_FOUND");
        assert_eq!(error_info.domain, "test.example.com");
        assert!(error_info.metadata.is_empty());
    }

    #[test]
    fn test_from_error_with_metadata_to_status() {
        let err = ErrorWithMetadata {
            user_id: "user_123".to_string(),
            resource_id: "doc_456".to_string(),
        };

        let status: Status = err.into();

        assert_eq!(status.code, 5); // NOT_FOUND = 5
        assert!(status.message.contains("user_123"));
        assert!(status.message.contains("doc_456"));

        let error_info = status.details.error_info.as_ref().unwrap();
        assert_eq!(error_info.reason, "ACCESS_DENIED");
        assert_eq!(error_info.domain, "authz.example.com");
        assert_eq!(
            error_info.metadata.get("user_id"),
            Some(&"user_123".to_string())
        );
        assert_eq!(
            error_info.metadata.get("resource_id"),
            Some(&"doc_456".to_string())
        );
    }

    #[test]
    fn test_from_all_error_kinds() {
        let test_cases = [
            (ErrorKind::NotFound, 5, "NOT_FOUND"),
            (ErrorKind::InvalidArgument, 3, "INVALID_ARGUMENT"),
            (ErrorKind::PermissionDenied, 7, "PERMISSION_DENIED"),
        ];

        for (kind, expected_code, _code_name) in test_cases {
            let err = SimpleError { kind };
            let status: Status = err.into();
            assert_eq!(status.code, expected_code);
        }
    }

    #[test]
    fn test_status_from_function() {
        let err = SimpleError {
            kind: ErrorKind::InvalidArgument,
        };

        let status = Status::from(err);

        assert_eq!(status.code, 3); // INVALID_ARGUMENT = 3
        assert_eq!(status.message, "Invalid argument provided");
    }

    #[test]
    fn test_status_details_always_has_error_info() {
        let err = SimpleError {
            kind: ErrorKind::PermissionDenied,
        };

        let status: Status = err.into();

        // From<IntoStatus> should always set error_info
        assert!(status.details.error_info.is_some());
        assert!(!status.details.is_empty());
    }

    // ============ Serialization Tests ============

    #[test]
    fn test_status_from_into_status_serializes() {
        let err = ErrorWithMetadata {
            user_id: "usr_001".to_string(),
            resource_id: "res_002".to_string(),
        };

        let status: Status = err.into();
        let json = serde_json::to_string(&status).unwrap();

        assert!(json.contains(r#""code":5"#));
        assert!(json.contains(r#""reason":"ACCESS_DENIED""#));
        assert!(json.contains(r#""domain":"authz.example.com""#));
        assert!(json.contains(r#""user_id":"usr_001""#));
        assert!(json.contains(r#""resource_id":"res_002""#));
    }

    #[test]
    fn test_status_roundtrip_with_into_status() {
        let err = SimpleError {
            kind: ErrorKind::NotFound,
        };

        let status: Status = err.into();
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: Status = serde_json::from_str(&json).unwrap();

        assert_eq!(status, deserialized);
    }

    // ============ Edge Cases ============

    #[test]
    fn test_empty_message() {
        struct EmptyMessageError;

        impl IntoStatus for EmptyMessageError {
            fn code(&self) -> Code {
                Code::Unknown
            }
            fn message(&self) -> String {
                String::new()
            }
            fn reason(&self) -> &str {
                "UNKNOWN"
            }
            fn domain(&self) -> &str {
                "test.com"
            }
        }

        let status: Status = EmptyMessageError.into();
        assert!(status.message.is_empty());
    }

    #[test]
    fn test_unicode_in_fields() {
        struct UnicodeError;

        impl IntoStatus for UnicodeError {
            fn code(&self) -> Code {
                Code::InvalidArgument
            }
            fn message(&self) -> String {
                "无效参数：名前が無効です 🚫".to_string()
            }
            fn reason(&self) -> &str {
                "INVALID_INPUT"
            }
            fn domain(&self) -> &str {
                "api.例え.com"
            }
        }

        let status: Status = UnicodeError.into();
        assert!(status.message.contains("无效参数"));
        assert!(status.message.contains("🚫"));

        let error_info = status.details.error_info.as_ref().unwrap();
        assert!(error_info.domain.contains("例え"));
    }

    #[test]
    fn test_large_metadata() {
        struct LargeMetadataError;

        impl IntoStatus for LargeMetadataError {
            fn code(&self) -> Code {
                Code::Internal
            }
            fn message(&self) -> String {
                "Error with large metadata".to_string()
            }
            fn reason(&self) -> &str {
                "INTERNAL_ERROR"
            }
            fn domain(&self) -> &str {
                "test.com"
            }
            fn metadata(&self) -> HashMap<String, String> {
                let mut map = HashMap::new();
                for i in 0..100 {
                    map.insert(format!("key_{}", i), format!("value_{}", i));
                }
                map
            }
        }

        let status: Status = LargeMetadataError.into();
        let error_info = status.details.error_info.as_ref().unwrap();
        assert_eq!(error_info.metadata.len(), 100);
        assert_eq!(error_info.metadata.get("key_50"), Some(&"value_50".to_string()));
    }
}
