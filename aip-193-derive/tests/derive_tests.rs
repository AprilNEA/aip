//! Integration tests for aip-193-derive crate
//!
//! These tests verify that the `#[derive(IntoStatus)]` macro generates
//! correct implementations of the `IntoStatus` trait.

#![allow(dead_code)]

use aip_193::{Code, IntoStatus, Status};
use aip_193_derive::IntoStatus;
use serde::{Deserialize, Serialize};
use strum::AsRefStr;

// ============================================================================
// Basic Derive Tests
// ============================================================================

/// Simple error enum with unit variants
#[derive(Debug, Clone, IntoStatus, AsRefStr)]
#[status(domain = "test.example.com")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum SimpleError {
    #[status(code = NotFound)]
    NotFound,

    #[status(code = InvalidArgument)]
    InvalidInput,

    #[status(code = Internal)]
    InternalError,
}

#[test]
fn test_simple_error_code() {
    assert_eq!(SimpleError::NotFound.code(), Code::NotFound);
    assert_eq!(SimpleError::InvalidInput.code(), Code::InvalidArgument);
    assert_eq!(SimpleError::InternalError.code(), Code::Internal);
}

#[test]
fn test_simple_error_reason() {
    assert_eq!(SimpleError::NotFound.reason(), "NOT_FOUND");
    assert_eq!(SimpleError::InvalidInput.reason(), "INVALID_INPUT");
    assert_eq!(SimpleError::InternalError.reason(), "INTERNAL_ERROR");
}

#[test]
fn test_simple_error_domain() {
    assert_eq!(SimpleError::NotFound.domain(), "test.example.com");
    assert_eq!(SimpleError::InvalidInput.domain(), "test.example.com");
    assert_eq!(SimpleError::InternalError.domain(), "test.example.com");
}

#[test]
fn test_simple_error_default_message() {
    // Default message is the variant name
    assert_eq!(SimpleError::NotFound.message(), "NotFound");
    assert_eq!(SimpleError::InvalidInput.message(), "InvalidInput");
    assert_eq!(SimpleError::InternalError.message(), "InternalError");
}

#[test]
fn test_simple_error_metadata() {
    // Unit variants have empty metadata
    assert!(SimpleError::NotFound.metadata().is_empty());
    assert!(SimpleError::InvalidInput.metadata().is_empty());
}

// ============================================================================
// Custom Message Tests
// ============================================================================

/// Error enum with custom messages
#[derive(Debug, Clone, IntoStatus, AsRefStr)]
#[status(domain = "api.myservice.com")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum ErrorWithMessage {
    #[status(code = NotFound, message = "The requested resource was not found")]
    ResourceNotFound,

    #[status(code = InvalidArgument, message = "Invalid request parameters")]
    BadRequest,

    #[status(code = PermissionDenied, message = "You don't have permission to access this resource")]
    Forbidden,
}

#[test]
fn test_custom_message() {
    assert_eq!(
        ErrorWithMessage::ResourceNotFound.message(),
        "The requested resource was not found"
    );
    assert_eq!(
        ErrorWithMessage::BadRequest.message(),
        "Invalid request parameters"
    );
    assert_eq!(
        ErrorWithMessage::Forbidden.message(),
        "You don't have permission to access this resource"
    );
}

// ============================================================================
// Struct Variant with Metadata Tests
// ============================================================================

/// Error enum with struct variants and metadata
#[derive(Debug, Clone, IntoStatus, AsRefStr)]
#[status(domain = "users.myapp.com")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum UserError {
    #[status(code = NotFound, message = "User not found")]
    UserNotFound {
        #[status(metadata)]
        user_id: String,
    },

    #[status(code = InvalidArgument, message = "Invalid email format")]
    InvalidEmail {
        #[status(metadata)]
        email: String,
        #[status(metadata)]
        reason: String,
    },

    #[status(code = AlreadyExists, message = "User already exists")]
    UserAlreadyExists {
        #[status(metadata)]
        email: String,
    },

    #[status(code = PermissionDenied)]
    AccessDenied {
        #[status(metadata)]
        user_id: String,
        #[status(metadata)]
        resource: String,
        #[status(metadata)]
        action: String,
    },
}

#[test]
fn test_struct_variant_code() {
    let err = UserError::UserNotFound {
        user_id: "123".to_string(),
    };
    assert_eq!(err.code(), Code::NotFound);

    let err = UserError::InvalidEmail {
        email: "bad".to_string(),
        reason: "missing @".to_string(),
    };
    assert_eq!(err.code(), Code::InvalidArgument);
}

#[test]
fn test_struct_variant_metadata() {
    let err = UserError::UserNotFound {
        user_id: "usr_123".to_string(),
    };
    let metadata = err.metadata();
    assert_eq!(metadata.get("user_id"), Some(&"usr_123".to_string()));
    assert_eq!(metadata.len(), 1);
}

#[test]
fn test_struct_variant_multiple_metadata() {
    let err = UserError::InvalidEmail {
        email: "invalid-email".to_string(),
        reason: "missing domain".to_string(),
    };
    let metadata = err.metadata();
    assert_eq!(metadata.get("email"), Some(&"invalid-email".to_string()));
    assert_eq!(metadata.get("reason"), Some(&"missing domain".to_string()));
    assert_eq!(metadata.len(), 2);
}

#[test]
fn test_struct_variant_many_metadata_fields() {
    let err = UserError::AccessDenied {
        user_id: "user_456".to_string(),
        resource: "documents/secret".to_string(),
        action: "read".to_string(),
    };
    let metadata = err.metadata();
    assert_eq!(metadata.get("user_id"), Some(&"user_456".to_string()));
    assert_eq!(metadata.get("resource"), Some(&"documents/secret".to_string()));
    assert_eq!(metadata.get("action"), Some(&"read".to_string()));
    assert_eq!(metadata.len(), 3);
}

// ============================================================================
// Custom Metadata Key Tests
// ============================================================================

/// Error enum with custom metadata key names
#[derive(Debug, Clone, IntoStatus, AsRefStr)]
#[status(domain = "api.example.com")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum ErrorWithCustomKeys {
    #[status(code = ResourceExhausted, message = "Rate limit exceeded")]
    RateLimited {
        #[status(metadata, metadata_key = "retry_after_seconds")]
        retry_after: u32,

        #[status(metadata, metadata_key = "quota_limit")]
        limit: u32,

        #[status(metadata, metadata_key = "quota_used")]
        used: u32,
    },
}

#[test]
fn test_custom_metadata_keys() {
    let err = ErrorWithCustomKeys::RateLimited {
        retry_after: 60,
        limit: 1000,
        used: 1000,
    };
    let metadata = err.metadata();

    // Keys should use custom names
    assert_eq!(metadata.get("retry_after_seconds"), Some(&"60".to_string()));
    assert_eq!(metadata.get("quota_limit"), Some(&"1000".to_string()));
    assert_eq!(metadata.get("quota_used"), Some(&"1000".to_string()));

    // Original field names should not be present
    assert!(metadata.get("retry_after").is_none());
    assert!(metadata.get("limit").is_none());
    assert!(metadata.get("used").is_none());
}

// ============================================================================
// Mixed Variant Styles Tests
// ============================================================================

/// Error enum mixing unit and struct variants
#[derive(Debug, Clone, IntoStatus, AsRefStr)]
#[status(domain = "mixed.example.com")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum MixedError {
    #[status(code = Unknown)]
    Unknown,

    #[status(code = NotFound)]
    ItemNotFound {
        #[status(metadata)]
        item_id: String,
    },

    #[status(code = Internal, message = "Database connection failed")]
    DatabaseError,

    #[status(code = Unavailable)]
    ServiceDown {
        #[status(metadata)]
        service_name: String,
        #[status(metadata, metadata_key = "retry_in_ms")]
        retry_ms: u64,
    },
}

#[test]
fn test_mixed_variants() {
    // Unit variant
    let err = MixedError::Unknown;
    assert_eq!(err.code(), Code::Unknown);
    assert!(err.metadata().is_empty());

    // Struct variant with metadata
    let err = MixedError::ItemNotFound {
        item_id: "item_789".to_string(),
    };
    assert_eq!(err.code(), Code::NotFound);
    assert_eq!(err.metadata().get("item_id"), Some(&"item_789".to_string()));

    // Unit variant with custom message
    let err = MixedError::DatabaseError;
    assert_eq!(err.code(), Code::Internal);
    assert_eq!(err.message(), "Database connection failed");

    // Struct variant with custom key
    let err = MixedError::ServiceDown {
        service_name: "payment-service".to_string(),
        retry_ms: 5000,
    };
    assert_eq!(err.code(), Code::Unavailable);
    let metadata = err.metadata();
    assert_eq!(metadata.get("service_name"), Some(&"payment-service".to_string()));
    assert_eq!(metadata.get("retry_in_ms"), Some(&"5000".to_string()));
}

// ============================================================================
// Conversion to Status Tests
// ============================================================================

#[test]
fn test_into_status_conversion() {
    let err = UserError::UserNotFound {
        user_id: "usr_abc".to_string(),
    };

    let status: Status = err.into();

    assert_eq!(status.code, Code::NotFound as i32);
    assert_eq!(status.message, "User not found");

    let error_info = status.details.error_info.as_ref().unwrap();
    assert_eq!(error_info.reason, "USER_NOT_FOUND");
    assert_eq!(error_info.domain, "users.myapp.com");
    assert_eq!(error_info.metadata.get("user_id"), Some(&"usr_abc".to_string()));
}

#[test]
fn test_status_from_conversion() {
    let err = SimpleError::InternalError;
    let status = Status::from(err);

    assert_eq!(status.code, Code::Internal as i32);
    assert_eq!(status.message, "InternalError");

    let error_info = status.details.error_info.as_ref().unwrap();
    assert_eq!(error_info.reason, "INTERNAL_ERROR");
    assert_eq!(error_info.domain, "test.example.com");
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_status_json_serialization() {
    let err = UserError::InvalidEmail {
        email: "not-an-email".to_string(),
        reason: "missing @ symbol".to_string(),
    };

    let status: Status = err.into();
    let json = serde_json::to_string(&status).unwrap();

    assert!(json.contains(r#""code":3"#)); // InvalidArgument = 3
    assert!(json.contains(r#""message":"Invalid email format""#));
    assert!(json.contains(r#""reason":"INVALID_EMAIL""#));
    assert!(json.contains(r#""domain":"users.myapp.com""#));
    assert!(json.contains(r#""email":"not-an-email""#));
    assert!(json.contains(r#""reason":"missing @ symbol""#));
}

#[test]
fn test_status_json_roundtrip() {
    let err = ErrorWithCustomKeys::RateLimited {
        retry_after: 30,
        limit: 100,
        used: 100,
    };

    let status: Status = err.into();
    let json = serde_json::to_string(&status).unwrap();
    let parsed: Status = serde_json::from_str(&json).unwrap();

    assert_eq!(status, parsed);
}

// ============================================================================
// All Code Types Tests
// ============================================================================

/// Test all possible Code variants
#[derive(Debug, Clone, IntoStatus, AsRefStr)]
#[status(domain = "codes.test.com")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum AllCodesError {
    #[status(code = Ok)]
    Success,
    #[status(code = Cancelled)]
    Cancelled,
    #[status(code = Unknown)]
    Unknown,
    #[status(code = InvalidArgument)]
    InvalidArgument,
    #[status(code = DeadlineExceeded)]
    DeadlineExceeded,
    #[status(code = NotFound)]
    NotFound,
    #[status(code = AlreadyExists)]
    AlreadyExists,
    #[status(code = PermissionDenied)]
    PermissionDenied,
    #[status(code = ResourceExhausted)]
    ResourceExhausted,
    #[status(code = FailedPrecondition)]
    FailedPrecondition,
    #[status(code = Aborted)]
    Aborted,
    #[status(code = OutOfRange)]
    OutOfRange,
    #[status(code = Unimplemented)]
    Unimplemented,
    #[status(code = Internal)]
    Internal,
    #[status(code = Unavailable)]
    Unavailable,
    #[status(code = DataLoss)]
    DataLoss,
    #[status(code = Unauthenticated)]
    Unauthenticated,
}

#[test]
fn test_all_code_mappings() {
    let test_cases = [
        (AllCodesError::Success, Code::Ok),
        (AllCodesError::Cancelled, Code::Cancelled),
        (AllCodesError::Unknown, Code::Unknown),
        (AllCodesError::InvalidArgument, Code::InvalidArgument),
        (AllCodesError::DeadlineExceeded, Code::DeadlineExceeded),
        (AllCodesError::NotFound, Code::NotFound),
        (AllCodesError::AlreadyExists, Code::AlreadyExists),
        (AllCodesError::PermissionDenied, Code::PermissionDenied),
        (AllCodesError::ResourceExhausted, Code::ResourceExhausted),
        (AllCodesError::FailedPrecondition, Code::FailedPrecondition),
        (AllCodesError::Aborted, Code::Aborted),
        (AllCodesError::OutOfRange, Code::OutOfRange),
        (AllCodesError::Unimplemented, Code::Unimplemented),
        (AllCodesError::Internal, Code::Internal),
        (AllCodesError::Unavailable, Code::Unavailable),
        (AllCodesError::DataLoss, Code::DataLoss),
        (AllCodesError::Unauthenticated, Code::Unauthenticated),
    ];

    for (err, expected_code) in test_cases {
        assert_eq!(err.code(), expected_code, "Failed for {:?}", err);
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test with Unicode in messages and metadata
#[derive(Debug, Clone, IntoStatus, AsRefStr)]
#[status(domain = "i18n.example.com")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum I18nError {
    #[status(code = InvalidArgument, message = "入力が無効です")]
    InvalidInput {
        #[status(metadata)]
        field: String,
    },
}

#[test]
fn test_unicode_support() {
    let err = I18nError::InvalidInput {
        field: "名前".to_string(),
    };

    assert_eq!(err.message(), "入力が無効です");
    assert_eq!(err.metadata().get("field"), Some(&"名前".to_string()));

    // Should serialize correctly
    let status: Status = err.into();
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("入力が無効です"));
    assert!(json.contains("名前"));
}

/// Test with fields that have special Display implementations
#[derive(Debug, Clone, IntoStatus, AsRefStr)]
#[status(domain = "numeric.example.com")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum NumericError {
    #[status(code = OutOfRange)]
    ValueOutOfRange {
        #[status(metadata)]
        min: i64,
        #[status(metadata)]
        max: i64,
        #[status(metadata)]
        actual: i64,
    },
}

#[test]
fn test_numeric_metadata() {
    let err = NumericError::ValueOutOfRange {
        min: -100,
        max: 100,
        actual: 150,
    };

    let metadata = err.metadata();
    assert_eq!(metadata.get("min"), Some(&"-100".to_string()));
    assert_eq!(metadata.get("max"), Some(&"100".to_string()));
    assert_eq!(metadata.get("actual"), Some(&"150".to_string()));
}

// ============================================================================
// Real-World Scenarios
// ============================================================================

/// Realistic API error enum
#[derive(Debug, Clone, IntoStatus, AsRefStr, Serialize, Deserialize)]
#[status(domain = "api.mycompany.com")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiError {
    // Authentication errors
    #[status(code = Unauthenticated, message = "Authentication required")]
    Unauthenticated,

    #[status(code = Unauthenticated, message = "Invalid or expired token")]
    InvalidToken {
        #[status(metadata, metadata_key = "token_type")]
        token_type: String,
    },

    // Authorization errors
    #[status(code = PermissionDenied, message = "Insufficient permissions")]
    InsufficientPermissions {
        #[status(metadata)]
        required_permission: String,
        #[status(metadata)]
        resource: String,
    },

    // Validation errors
    #[status(code = InvalidArgument, message = "Validation failed")]
    ValidationFailed {
        #[status(metadata)]
        field: String,
        #[status(metadata)]
        constraint: String,
    },

    // Resource errors
    #[status(code = NotFound, message = "Resource not found")]
    NotFound {
        #[status(metadata, metadata_key = "resource_type")]
        resource_type: String,
        #[status(metadata, metadata_key = "resource_id")]
        id: String,
    },

    #[status(code = AlreadyExists, message = "Resource already exists")]
    AlreadyExists {
        #[status(metadata, metadata_key = "resource_type")]
        resource_type: String,
        #[status(metadata)]
        identifier: String,
    },

    // Rate limiting
    #[status(code = ResourceExhausted, message = "Rate limit exceeded")]
    RateLimited {
        #[status(metadata, metadata_key = "retry_after_seconds")]
        retry_after: u32,
    },

    // Server errors
    #[status(code = Internal, message = "Internal server error")]
    InternalError,

    #[status(code = Unavailable, message = "Service temporarily unavailable")]
    ServiceUnavailable {
        #[status(metadata, metadata_key = "expected_recovery_time")]
        recovery_time: String,
    },
}

#[test]
fn test_realistic_api_errors() {
    // Authentication error
    let err = ApiError::InvalidToken {
        token_type: "Bearer".to_string(),
    };
    let status: Status = err.into();
    assert_eq!(status.code, Code::Unauthenticated as i32);
    let info = status.details.error_info.unwrap();
    assert_eq!(info.metadata.get("token_type"), Some(&"Bearer".to_string()));

    // Validation error
    let err = ApiError::ValidationFailed {
        field: "email".to_string(),
        constraint: "must be a valid email address".to_string(),
    };
    let status: Status = err.into();
    assert_eq!(status.code, Code::InvalidArgument as i32);
    let info = status.details.error_info.unwrap();
    assert_eq!(info.metadata.get("field"), Some(&"email".to_string()));
    assert_eq!(
        info.metadata.get("constraint"),
        Some(&"must be a valid email address".to_string())
    );

    // Not found with custom keys
    let err = ApiError::NotFound {
        resource_type: "User".to_string(),
        id: "usr_12345".to_string(),
    };
    let status: Status = err.into();
    assert_eq!(status.code, Code::NotFound as i32);
    let info = status.details.error_info.unwrap();
    assert_eq!(info.metadata.get("resource_type"), Some(&"User".to_string()));
    assert_eq!(info.metadata.get("resource_id"), Some(&"usr_12345".to_string()));

    // Rate limiting
    let err = ApiError::RateLimited { retry_after: 30 };
    let status: Status = err.into();
    assert_eq!(status.code, Code::ResourceExhausted as i32);
    let info = status.details.error_info.unwrap();
    assert_eq!(
        info.metadata.get("retry_after_seconds"),
        Some(&"30".to_string())
    );
}

#[test]
fn test_api_error_json_response() {
    let err = ApiError::InsufficientPermissions {
        required_permission: "documents.write".to_string(),
        resource: "projects/123/documents/456".to_string(),
    };

    let status: Status = err.into();
    let json = serde_json::to_string_pretty(&status).unwrap();

    // This is what an API would return
    assert!(json.contains(r#""code": 7"#)); // PERMISSION_DENIED
    assert!(json.contains(r#""message": "Insufficient permissions""#));
    assert!(json.contains(r#""reason": "INSUFFICIENT_PERMISSIONS""#));
    assert!(json.contains(r#""domain": "api.mycompany.com""#));
    assert!(json.contains(r#""required_permission": "documents.write""#));
    assert!(json.contains(r#""resource": "projects/123/documents/456""#));
}
