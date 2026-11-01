use std::path::PathBuf;

struct NativePathReport { encoding: String, utf8: Option<String> }
#[derive(Debug)]
enum WorkbenchError {
    UnsupportedNativeEncoding { encoding: String },
    InvalidBinding { detail: String },
}

// Exact active non-Unix let-return / method / Result<PathBuf> shape,
// copied as unconditional host Rust. This is not a Windows target build.
fn decode_native_path(report: &NativePathReport) -> Result<PathBuf, WorkbenchError> {
    let path = return Err(WorkbenchError::UnsupportedNativeEncoding {
        encoding: report.encoding.clone(),
    });
    if !path.is_absolute() || path.to_str() != report.utf8.as_deref() {
        return Err(WorkbenchError::InvalidBinding {
            detail: "native path absolute/UTF-8 fields disagree".into(),
        });
    }
    Ok(path)
}
