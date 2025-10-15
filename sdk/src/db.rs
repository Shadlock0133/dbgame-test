use crate::db_internal::db_log;
use core::ffi::CStr;

/// Print a formatted message to debug output
#[macro_export]
macro_rules! logfmt {
    ($($arg:tt)*) => {
        $crate::db::log(&::std::ffi::CString::new(format!($($arg)*)).unwrap())
    };
}

/// Prints a message to debug output
pub fn log(cstr: &CStr) {
    unsafe {
        db_log(cstr.as_ptr());
    }
}

/// Register custom DreamBox-specific panic handler
pub fn register_panic() {
    std::panic::set_hook(Box::new(|panic_info| {
        logfmt!("FATAL ERROR: {}", panic_info);
    }));
}
