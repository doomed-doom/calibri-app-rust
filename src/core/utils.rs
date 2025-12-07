use crate::core::bindings::*;
use std::ffi::CStr;

pub fn empty_status() -> OpStatus {
    OpStatus {
        Success: 0,
        Error: 0,
        ErrorMsg: [0; 512],
    }
}

pub fn status_message(status: &OpStatus) -> String {
    unsafe {
        CStr::from_ptr(status.ErrorMsg.as_ptr())
            .to_string_lossy()
            .into_owned()
    }
}

pub fn extract_str(buf: &[std::os::raw::c_char]) -> String {
    let slice = unsafe { CStr::from_ptr(buf.as_ptr()) };
    slice.to_string_lossy().trim().to_string()
}
