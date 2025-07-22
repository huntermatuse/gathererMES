use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub data: T,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub error: String,
}

impl<T> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            success: true,
            timestamp: Utc::now(),
            data,
        }
    }
}

impl ErrorResponse {
    pub fn new(error: String) -> Self {
        Self {
            success: false,
            timestamp: Utc::now(),
            error,
        }
    }

    pub fn from_str(error: &str) -> Self {
        Self::new(error.to_string())
    }
}

// unified response enum
#[derive(Serialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Success(SuccessResponse<T>),
    Error(ErrorResponse),
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self::Success(SuccessResponse::new(data))
    }

    pub fn error(error: String) -> Self {
        Self::Error(ErrorResponse::new(error))
    }

    pub fn error_str(error: &str) -> Self {
        Self::Error(ErrorResponse::from_str(error))
    }
}

// Usage examples
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct User {
        id: u32,
        name: String,
    }

    #[test]
    fn test_responses() {
        let user = User {
            id: 1,
            name: "Alice".to_string(),
        };

        let success = SuccessResponse::new(user);
        let error = ErrorResponse::from_str("User not found");

        // Solution 1: Explicit type annotation
        let api_success = ApiResponse::success("Hello World");
        let api_error: ApiResponse<String> = ApiResponse::error_str("Invalid request");

        // Solution 2: Turbofish syntax
        let _api_error2 = ApiResponse::<User>::error_str("User not found");

        // Solution 3: Use individual response types when you don't need the enum
        let _individual_error = ErrorResponse::from_str("Individual error");

        // These would serialize to JSON properly
        println!("{}", serde_json::to_string_pretty(&success).unwrap());
        println!("{}", serde_json::to_string_pretty(&error).unwrap());
        println!("{}", serde_json::to_string_pretty(&api_success).unwrap());
        println!("{}", serde_json::to_string_pretty(&api_error).unwrap());
    }
}
