use http::StatusCode;

use crate::Code;

// gRPC Code -> HTTP StatusCode
impl From<Code> for StatusCode {
    fn from(code: Code) -> Self {
        match code {
            Code::Ok => StatusCode::OK, // 200
            Code::Cancelled => StatusCode::from_u16(499) // 499 (Non-standard)
                .unwrap_or(StatusCode::BAD_REQUEST),
            Code::Unknown => StatusCode::INTERNAL_SERVER_ERROR, // 500
            Code::InvalidArgument => StatusCode::BAD_REQUEST,   // 400
            Code::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT, // 504
            Code::NotFound => StatusCode::NOT_FOUND,            // 404
            Code::AlreadyExists => StatusCode::CONFLICT,        // 409
            Code::PermissionDenied => StatusCode::FORBIDDEN,    // 403
            Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS, // 429
            Code::FailedPrecondition => StatusCode::BAD_REQUEST, // 400
            Code::Aborted => StatusCode::CONFLICT,              // 409
            Code::OutOfRange => StatusCode::BAD_REQUEST,        // 400
            Code::Unimplemented => StatusCode::NOT_IMPLEMENTED, // 501
            Code::Internal => StatusCode::INTERNAL_SERVER_ERROR, // 500
            Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE, // 503
            Code::DataLoss => StatusCode::INTERNAL_SERVER_ERROR, // 500
            Code::Unauthenticated => StatusCode::UNAUTHORIZED,  // 401
        }
    }
}

// HTTP StatusCode -> gRPC Code
impl From<StatusCode> for Code {
    fn from(status: StatusCode) -> Self {
        match status.as_u16() {
            200..=299 => Code::Ok,
            400 => Code::InvalidArgument,
            401 => Code::Unauthenticated,
            403 => Code::PermissionDenied,
            404 => Code::NotFound,
            409 => Code::AlreadyExists,
            429 => Code::ResourceExhausted,
            499 => Code::Cancelled,
            501 => Code::Unimplemented,
            503 => Code::Unavailable,
            504 => Code::DeadlineExceeded,
            400..=499 => Code::FailedPrecondition, // Other 4xx
            500..=599 => Code::Internal,           // Other 5xx
            _ => Code::Unknown,
        }
    }
}
