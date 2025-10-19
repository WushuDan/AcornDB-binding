#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CStr;

/// Return last error as Rust String (copies bytes and frees the original)
pub unsafe fn last_error_string() -> String {
    let ptr = acorn_error_message();
    if ptr.is_null() { return String::new(); }
    let result = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    acorn_free_error_string(ptr);
    result
}
