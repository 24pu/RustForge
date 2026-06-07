use serde::{Serialize, Deserialize};
use std::alloc::{Layout, alloc};
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

// ========== 插件元数据 ==========
const PLUGIN_METADATA: &str = r#"{
    "name": "user-center",
    "version": "1.0.0",
    "author": "RustForge Team",
    "description": "用户中心插件，提供个人资料等功能"
}"#;

static mut LAST_LEN: usize = 0;

// ========== 导出函数 ==========

#[no_mangle]
pub extern "C" fn plugin_metadata() -> *mut u8 {
    let bytes = PLUGIN_METADATA.as_bytes();
    let len = bytes.len();
    unsafe {
        let layout = Layout::array::<u8>(len).unwrap();
        let ptr = alloc(layout) as *mut u8;
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
        LAST_LEN = len;
        ptr
    }
}

#[no_mangle]
pub extern "C" fn get_last_result_len() -> usize {
    unsafe { LAST_LEN }
}

/// API 调用处理
#[no_mangle]
pub extern "C" fn execute(ptr: *const u8, len: usize) -> *mut u8 {
    let input_bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let input_str = match std::str::from_utf8(input_bytes) {
        Ok(s) => s,
        Err(_) => return empty_error(),
    };
    let input: serde_json::Value = match serde_json::from_str(input_str) {
        Ok(v) => v,
        Err(_) => return empty_error(),
    };
    let action = input.get("action").and_then(|v| v.as_str()).unwrap_or("");
    let user_id = input.get("user_id").and_then(|v| v.as_str()).unwrap_or("");
    let result = match action {
        "get_profile" => handle_get_profile(user_id),
        "update_profile" => handle_update_profile(user_id, &input),
        "change_password" => handle_change_password(user_id, &input),
        "get_orders" => handle_get_orders(user_id),
        "logout" => serde_json::json!({ "message": "Logout successful" }),
        _ => serde_json::json!({ "error": "Unknown action" }),
    };
    let output = result.to_string();
    let bytes = output.into_bytes();
    let result_len = bytes.len();
    unsafe {
        let layout = match Layout::array::<u8>(result_len) {
            Ok(l) => l,
            Err(_) => return std::ptr::null_mut(),
        };
        let ptr = alloc(layout) as *mut u8;
        if ptr.is_null() {
            return std::ptr::null_mut();
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, result_len);
        LAST_LEN = result_len;
        ptr
    }
}

fn empty_error() -> *mut u8 {
    let err = serde_json::json!({ "error": "Invalid request" }).to_string();
    let bytes = err.into_bytes();
    let len = bytes.len();
    unsafe {
        let layout = Layout::array::<u8>(len).unwrap();
        let ptr = alloc(layout) as *mut u8;
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
        LAST_LEN = len;
        ptr
    }
}

// ========== 业务逻辑 ==========

#[derive(Clone, Serialize, Deserialize)]
struct UserProfile {
    nickname: String,
    email: String,
    phone: String,
}

lazy_static! {
    static ref USER_DB: Mutex<HashMap<String, UserProfile>> = {
        let mut m = HashMap::new();
        m.insert("05396fcf-3493-4680-9c7b-dbb064049900".to_string(), UserProfile {
            nickname: "管理员".to_string(),
            email: "admin@example.com".to_string(),
            phone: "13888888888".to_string(),
        });
        m.insert("test-id".to_string(), UserProfile {
            nickname: "测试用户".to_string(),
            email: "test@example.com".to_string(),
            phone: "13999999999".to_string(),
        });
        Mutex::new(m)
    };
}

fn handle_get_profile(user_id: &str) -> serde_json::Value {
    if user_id.is_empty() {
        return serde_json::json!({ "error": "Not authenticated" });
    }
    let db = USER_DB.lock().unwrap();
    if let Some(profile) = db.get(user_id) {
        serde_json::json!({
            "nickname": profile.nickname,
            "email": profile.email,
            "phone": profile.phone
        })
    } else {
        serde_json::json!({
            "nickname": "新用户",
            "email": "",
            "phone": ""
        })
    }
}

fn handle_update_profile(user_id: &str, input: &serde_json::Value) -> serde_json::Value {
    if user_id.is_empty() {
        return serde_json::json!({ "success": false, "error": "Not authenticated" });
    }
    let nickname = input.get("nickname").and_then(|v| v.as_str()).unwrap_or("");
    let phone = input.get("phone").and_then(|v| v.as_str()).unwrap_or("");
    if nickname.is_empty() {
        return serde_json::json!({ "success": false, "error": "昵称不能为空" });
    }
    if phone.len() != 11 {
        return serde_json::json!({ "success": false, "error": "手机号格式错误" });
    }
    let mut db = USER_DB.lock().unwrap();
    if let Some(profile) = db.get_mut(user_id) {
        profile.nickname = nickname.to_string();
        profile.phone = phone.to_string();
    } else {
        db.insert(user_id.to_string(), UserProfile {
            nickname: nickname.to_string(),
            email: format!("{}@example.com", user_id),
            phone: phone.to_string(),
        });
    }
    serde_json::json!({ "success": true, "message": "资料更新成功" })
}

fn handle_change_password(user_id: &str, input: &serde_json::Value) -> serde_json::Value {
    if user_id.is_empty() {
        return serde_json::json!({ "success": false, "error": "Not authenticated" });
    }
    let old = input.get("old_password").and_then(|v| v.as_str()).unwrap_or("");
    let new_pwd = input.get("new_password").and_then(|v| v.as_str()).unwrap_or("");
    if old != "123456" && user_id == "test-id" {
        return serde_json::json!({ "success": false, "error": "当前密码错误" });
    }
    if new_pwd.len() < 6 {
        return serde_json::json!({ "success": false, "error": "新密码长度至少6位" });
    }
    serde_json::json!({ "success": true, "message": "密码修改成功" })
}

fn handle_get_orders(user_id: &str) -> serde_json::Value {
    if user_id.is_empty() {
        return serde_json::json!({ "error": "Not authenticated" });
    }
    let orders = if user_id == "05396fcf-3493-4680-9c7b-dbb064049900" {
        vec![
            serde_json::json!({ "id": "ORD-001", "amount": "99.00", "status": "已完成", "created_at": "2024-01-15" }),
            serde_json::json!({ "id": "ORD-002", "amount": "199.00", "status": "待发货", "created_at": "2024-01-20" }),
        ]
    } else {
        vec![
            serde_json::json!({ "id": "ORD-003", "amount": "49.00", "status": "已完成", "created_at": "2024-02-01" }),
        ]
    };
    serde_json::json!({ "orders": orders })
}

#[no_mangle]
pub extern "C" fn is_page_protected(ptr: *const u8, len: usize) -> *mut u8 {
    let page_name = unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        std::str::from_utf8(slice).unwrap_or("")
    };
    let protected = match page_name {
        "profile" | "orders" | "password" => "true",
        _ => "false",
    };
    let bytes = protected.as_bytes();
    let result_len = bytes.len();
    unsafe {
        let layout = std::alloc::Layout::array::<u8>(result_len).unwrap();
        let ptr = std::alloc::alloc(layout) as *mut u8;
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, result_len);
        LAST_LEN = result_len;
        ptr
    }
}