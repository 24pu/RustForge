use std::alloc::{Layout, alloc};
use serde_json::json;

const METADATA: &str = r#"{
    "name": "musictool",
    "version": "1.0.0",
    "author": "RustForge Team",
    "description": "RustForge 调音工具"
}"#;

static mut LAST_LEN: usize = 0;

#[no_mangle]
pub extern "C" fn plugin_metadata() -> *mut u8 {
    let bytes = METADATA.as_bytes();
    unsafe {
        let layout = Layout::array::<u8>(bytes.len()).unwrap();
        let ptr = alloc(layout);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        LAST_LEN = bytes.len();
        ptr
    }
}

#[no_mangle]
pub extern "C" fn get_last_result_len() -> usize {
    unsafe { LAST_LEN }
}

#[no_mangle]
pub extern "C" fn execute(_ptr: *const u8, _len: usize) -> *mut u8 {
    let output = json!({"message": "ok"}).to_string();
    let bytes = output.as_bytes();
    unsafe {
        let layout = Layout::array::<u8>(bytes.len()).unwrap();
        let ptr = alloc(layout);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        LAST_LEN = bytes.len();
        ptr
    }
}

#[no_mangle]
pub extern "C" fn is_page_protected(_ptr: *const u8, _len: usize) -> *mut u8 {
    // 返回 "0" 表示不需要登录保护
    let output = "0".to_string();
    let bytes = output.into_bytes();
    let len = bytes.len();
    unsafe {
        let layout = Layout::array::<u8>(len).unwrap();
        let ptr = alloc(layout) as *mut u8;
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
        LAST_LEN = len;
        ptr
    }
}