use std::os::raw::c_char;

extern "C" {
    pub fn TsgoCheckProject(config_path: *const c_char) -> *mut c_char;
    pub fn TsgoFree(ptr: *mut c_char);
    pub fn TsgoVersion() -> *mut c_char;
}
