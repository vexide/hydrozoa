use wamr_sys::{log_level_t, wasm_runtime_set_log_level};

#[repr(u32)]
pub enum WamrLogLevel {
    FATAL = 0,
    ERROR = 1,
    WARNING = 2,
    DEBUG = 3,
    VERBOSE = 4,
}

/// Not thread safe, but we don't have threads so it's fine.
pub fn set_log_level(log_level: WamrLogLevel) {
    unsafe {
        wasm_runtime_set_log_level(log_level as log_level_t);
    }
}