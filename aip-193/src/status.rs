use serde::{Deserialize, Serialize};

use crate::{Code, ErrorInfo};

/// The `Status` type defines a logical error model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    /// The status code (google.rpc.Code enum value).
    pub code: Code,

    /// A developer-facing error message in English.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub message: String,

    /// A list of messages that carry the error details.
    #[serde(default, skip_serializing_if = "StatusDetails::is_empty")]
    pub details: StatusDetails,
}

/// A generic container for arbitrary serialized messages.
///
/// This is used to store error details in the `details` field of the `Status` type.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_info: Option<ErrorInfo>,
}

impl StatusDetails {
    pub fn is_empty(&self) -> bool {
        self.error_info.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ Status Tests ============

    #[test]
    fn test_status_default() {
        let status = Status {
            code: Code::Ok,
            message: String::new(),
            details: StatusDetails::default(),
        };
        assert_eq!(status.code, Code::Ok);
        assert!(status.message.is_empty());
        assert!(status.details.is_empty());
    }

    #[test]
    fn test_status_with_code_and_message() {
        let status = Status {
            code: Code::NotFound,
            message: "User not found".to_string(),
            details: StatusDetails::default(),
        };

        assert_eq!(status.code, Code::NotFound);
        assert_eq!(status.message, "User not found");
    }

    #[test]
    fn test_status_with_error_info() {
        let error_info =
            ErrorInfo::new("USER_NOT_FOUND", "myapp.example.com").with_metadata("user_id", "123");

        let status = Status {
            code: Code::NotFound,
            message: "User not found".to_string(),
            details: StatusDetails {
                error_info: Some(error_info),
            },
        };

        assert!(!status.details.is_empty());
        let info = status.details.error_info.as_ref().unwrap();
        assert_eq!(info.reason, "USER_NOT_FOUND");
        assert_eq!(info.domain, "myapp.example.com");
        assert_eq!(info.metadata.get("user_id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_status_clone() {
        let status = Status {
            code: Code::InvalidArgument,
            message: "Invalid argument".to_string(),
            details: StatusDetails {
                error_info: Some(ErrorInfo::new("INVALID_EMAIL", "auth.example.com")),
            },
        };

        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_status_equality() {
        let status1 = Status {
            code: Code::NotFound,
            message: "Not found".to_string(),
            details: StatusDetails::default(),
        };

        let status2 = Status {
            code: Code::NotFound,
            message: "Not found".to_string(),
            details: StatusDetails::default(),
        };

        let status3 = Status {
            code: Code::InvalidArgument,
            message: "Not found".to_string(),
            details: StatusDetails::default(),
        };

        assert_eq!(status1, status2);
        assert_ne!(status1, status3);
    }

    // ============ Status Serde Tests ============

    #[test]
    fn test_status_serialize_minimal() {
        let status = Status {
            code: Code::Ok,
            message: String::new(),
            details: StatusDetails::default(),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#"{"code":"OK"}"#);
    }

    #[test]
    fn test_status_serialize_with_message() {
        let status = Status {
            code: Code::NotFound,
            message: "Resource not found".to_string(),
            details: StatusDetails::default(),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains(r#""code":"NOT_FOUND""#));
        assert!(json.contains(r#""message":"Resource not found""#));
        assert!(!json.contains("details"));
    }

    #[test]
    fn test_status_serialize_full() {
        let status = Status {
            code: Code::PermissionDenied,
            message: "Permission denied".to_string(),
            details: StatusDetails {
                error_info: Some(
                    ErrorInfo::new("ACCESS_DENIED", "iam.example.com")
                        .with_metadata("resource", "projects/123"),
                ),
            },
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains(r#""code":"PERMISSION_DENIED""#));
        assert!(json.contains(r#""message":"Permission denied""#));
        assert!(json.contains(r#""details""#));
        assert!(json.contains(r#""error_info""#));
        assert!(json.contains(r#""reason":"ACCESS_DENIED""#));
    }

    #[test]
    fn test_status_deserialize_minimal() {
        let json = r#"{"code":"OK"}"#;
        let status: Status = serde_json::from_str(json).unwrap();

        assert_eq!(status.code, Code::Ok);
        assert!(status.message.is_empty());
        assert!(status.details.is_empty());
    }

    #[test]
    fn test_status_deserialize_full() {
        let json = r#"{
            "code": "NOT_FOUND",
            "message": "User not found",
            "details": {
                "error_info": {
                    "reason": "USER_NOT_FOUND",
                    "domain": "users.example.com",
                    "metadata": {
                        "user_id": "usr_123"
                    }
                }
            }
        }"#;

        let status: Status = serde_json::from_str(json).unwrap();
        assert_eq!(status.code, Code::NotFound);
        assert_eq!(status.message, "User not found");

        let error_info = status.details.error_info.as_ref().unwrap();
        assert_eq!(error_info.reason, "USER_NOT_FOUND");
        assert_eq!(error_info.domain, "users.example.com");
        assert_eq!(
            error_info.metadata.get("user_id"),
            Some(&"usr_123".to_string())
        );
    }

    #[test]
    fn test_status_roundtrip() {
        let original = Status {
            code: Code::ResourceExhausted,
            message: "Rate limit exceeded".to_string(),
            details: StatusDetails {
                error_info: Some(
                    ErrorInfo::new("RATE_LIMITED", "api.example.com")
                        .with_metadata("retry_after", "30")
                        .with_metadata("quota_limit", "100"),
                ),
            },
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Status = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    // ============ StatusDetails Tests ============

    #[test]
    fn test_status_details_default() {
        let details = StatusDetails::default();
        assert!(details.error_info.is_none());
        assert!(details.is_empty());
    }

    #[test]
    fn test_status_details_with_error_info() {
        let details = StatusDetails {
            error_info: Some(ErrorInfo::new("ERROR", "test.com")),
        };

        assert!(!details.is_empty());
        assert!(details.error_info.is_some());
    }

    #[test]
    fn test_status_details_is_empty() {
        let empty = StatusDetails::default();
        assert!(empty.is_empty());

        let not_empty = StatusDetails {
            error_info: Some(ErrorInfo::default()),
        };
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn test_status_details_serialize_empty() {
        let details = StatusDetails::default();
        let json = serde_json::to_string(&details).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_status_details_serialize_with_error_info() {
        let details = StatusDetails {
            error_info: Some(ErrorInfo::new("TEST_ERROR", "test.com")),
        };

        let json = serde_json::to_string(&details).unwrap();
        assert!(json.contains(r#""error_info""#));
        assert!(json.contains(r#""reason":"TEST_ERROR""#));
    }

    #[test]
    fn test_status_details_clone() {
        let details = StatusDetails {
            error_info: Some(ErrorInfo::new("ERROR", "test.com")),
        };

        let cloned = details.clone();
        assert_eq!(details, cloned);
    }

    #[test]
    fn test_status_details_equality() {
        let details1 = StatusDetails {
            error_info: Some(ErrorInfo::new("ERROR", "test.com")),
        };
        let details2 = StatusDetails {
            error_info: Some(ErrorInfo::new("ERROR", "test.com")),
        };
        let details3 = StatusDetails {
            error_info: Some(ErrorInfo::new("OTHER", "test.com")),
        };

        assert_eq!(details1, details2);
        assert_ne!(details1, details3);
    }

    // ============ Status Code Mapping Tests ============

    #[test]
    fn test_status_common_codes() {
        // Test common gRPC status codes
        let codes = [
            (Code::Ok, "OK"),
            (Code::Cancelled, "CANCELLED"),
            (Code::Unknown, "UNKNOWN"),
            (Code::InvalidArgument, "INVALID_ARGUMENT"),
            (Code::DeadlineExceeded, "DEADLINE_EXCEEDED"),
            (Code::NotFound, "NOT_FOUND"),
            (Code::AlreadyExists, "ALREADY_EXISTS"),
            (Code::PermissionDenied, "PERMISSION_DENIED"),
            (Code::ResourceExhausted, "RESOURCE_EXHAUSTED"),
            (Code::FailedPrecondition, "FAILED_PRECONDITION"),
            (Code::Aborted, "ABORTED"),
            (Code::OutOfRange, "OUT_OF_RANGE"),
            (Code::Unimplemented, "UNIMPLEMENTED"),
            (Code::Internal, "INTERNAL"),
            (Code::Unavailable, "UNAVAILABLE"),
            (Code::DataLoss, "DATA_LOSS"),
            (Code::Unauthenticated, "UNAUTHENTICATED"),
        ];

        for (code, name) in codes {
            let status = Status {
                code,
                message: format!("Testing {}", name),
                details: StatusDetails::default(),
            };
            assert_eq!(status.code, code);
        }
    }
}
