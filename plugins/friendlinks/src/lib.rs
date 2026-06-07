use std::alloc::{Layout, alloc};

const PLUGIN_METADATA: &str = r#"{
    "name": "friendlinks",
    "version": "1.0.0",
    "author": "Admin",
    "description": "友情链接插件"
}"#;

static mut LAST_LEN: usize = 0;

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

#[no_mangle]
pub extern "C" fn execute(_ptr: *const u8, _len: usize) -> *mut u8 {
    let output = "{}".to_string();
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

#[no_mangle]
pub extern "C" fn is_page_protected(_ptr: *const u8, _len: usize) -> *mut u8 {
    let output = "false".to_string();
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