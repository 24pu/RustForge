// types/auth.rs

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub message: String,
    pub token: String,
}

#[derive(Clone, Debug)]
pub struct UserInfo {
    pub is_logged_in: bool,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
}

impl UserInfo {
    pub fn anonymous() -> Self {
        UserInfo {
            is_logged_in: false,
            user_id: None,
            user_name: None,
        }
    }
}