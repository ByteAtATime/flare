use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Once;

#[link(name = "SoulverWrapper")]
unsafe extern "C" {
    fn initialize_soulver(resources_path: *const c_char);
    fn evaluate(expression: *const c_char) -> *mut c_char;
    fn free_string(ptr: *mut c_char);
}

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let resources_path = std::path::PathBuf::from(manifest_dir)
            .join("SoulverWrapper/Vendor/SoulverCore-linux/SoulverCore_SoulverCore.resources");

        if let Ok(c_path) = CString::new(resources_path.to_string_lossy().as_bytes()) {
            unsafe {
                initialize_soulver(c_path.as_ptr());
            }
        }
    });
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SoulverResult {
    pub value: String,
    #[serde(rename = "type")]
    pub result_type: String,
    #[allow(dead_code)]
    pub error: Option<String>,
}

pub fn calculate(expression: &str) -> Option<SoulverResult> {
    init();

    let c_expr = CString::new(expression).ok()?;
    let result_ptr = unsafe { evaluate(c_expr.as_ptr()) };

    if result_ptr.is_null() {
        return None;
    }

    let result_str = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
    unsafe { free_string(result_ptr) };

    serde_json::from_str(&result_str).ok()
}
