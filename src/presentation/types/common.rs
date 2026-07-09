// types/common.rs

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            message: None,
            error: None,
        }
    }
    
    pub fn error(msg: String) -> Self {
        ApiResponse {
            success: false,
            data: None,
            message: None,
            error: Some(msg),
        }
    }
    
    pub fn message(msg: String) -> Self {
        ApiResponse {
            success: true,
            data: None,
            message: Some(msg),
            error: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

