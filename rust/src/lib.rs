mod ffi;

use std::ffi::{CStr, CString};
use std::path::Path;

use serde::Deserialize;

/// A TypeScript diagnostic (error, warning, suggestion, or message).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub file: Option<String>,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub message: String,
    pub code: i32,
    pub category: DiagnosticCategory,
}

/// The category of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCategory {
    Error,
    Warning,
    Suggestion,
    Message,
}

/// Errors returned by the library.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Go runtime error: {0}")]
    GoError(String),

    #[error("FFI returned null pointer")]
    NullPointer,

    #[error("invalid UTF-8 in FFI response: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("failed to parse JSON response: {0}")]
    Json(#[from] serde_json::Error),
}

// Internal JSON representation matching the Go side.
#[derive(Deserialize)]
struct ResultJson {
    diagnostics: Vec<DiagnosticJson>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
struct DiagnosticJson {
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    line: u32,
    #[serde(default)]
    column: u32,
    #[serde(default)]
    end_line: u32,
    #[serde(default)]
    end_column: u32,
    message: String,
    code: i32,
    category: String,
}

/// Type-check a TypeScript project given the path to its `tsconfig.json`.
///
/// Returns a list of diagnostics (errors, warnings, etc.) found during type-checking.
/// An empty list means the project is clean.
pub fn check_project(config_path: &Path) -> Result<Vec<Diagnostic>, Error> {
    let path_str = config_path.to_string_lossy();
    let c_path =
        CString::new(path_str.as_bytes()).expect("config_path contains interior null byte");

    let ptr = unsafe { ffi::TsgoCheckProject(c_path.as_ptr()) };
    if ptr.is_null() {
        return Err(Error::NullPointer);
    }

    let json_str = unsafe { CStr::from_ptr(ptr) }.to_str()?;
    let result: ResultJson = serde_json::from_str(json_str)?;

    // Free the C string allocated by Go.
    unsafe { ffi::TsgoFree(ptr) };

    if let Some(err) = result.error {
        return Err(Error::GoError(err));
    }

    let diagnostics = result
        .diagnostics
        .into_iter()
        .map(|d| Diagnostic {
            file: d.file,
            line: d.line,
            column: d.column,
            end_line: d.end_line,
            end_column: d.end_column,
            message: d.message,
            code: d.code,
            category: match d.category.as_str() {
                "warning" => DiagnosticCategory::Warning,
                "suggestion" => DiagnosticCategory::Suggestion,
                "message" => DiagnosticCategory::Message,
                _ => DiagnosticCategory::Error,
            },
        })
        .collect();

    Ok(diagnostics)
}

/// Returns the version string of the underlying tsgo library.
pub fn version() -> String {
    let ptr = unsafe { ffi::TsgoVersion() };
    if ptr.is_null() {
        return String::from("unknown");
    }
    let s = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::TsgoFree(ptr) };
    s
}
