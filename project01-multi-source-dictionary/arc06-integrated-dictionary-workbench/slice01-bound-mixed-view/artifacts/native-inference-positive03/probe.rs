use std::path::PathBuf;
struct NativePathReport { encoding: String, hex: String, utf8: Option<String> }
#[derive(Debug)]
enum WorkbenchError { UnsupportedNativeEncoding { encoding: String }, InvalidBinding { detail: String } }
fn decode_native_path(report: &NativePathReport) -> Result<PathBuf, WorkbenchError> {
    if !report.hex.len().is_multiple_of(2)
        || !report
            .hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(WorkbenchError::InvalidBinding {
            detail: "native path hex is malformed".into(),
        });
    }
    let bytes = report
        .hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                _ => 0,
            };
            digit(pair[0]) * 16 + digit(pair[1])
        })
        .collect::<Vec<_>>();
    if bytes.contains(&0) {
        return Err(WorkbenchError::InvalidBinding {
            detail: "native path contains NUL".into(),
        });
    }
    #[cfg(any())]
    {
        let path = {
            use std::os::unix::ffi::OsStringExt;
            if report.encoding != "unix-bytes" {
                return Err(WorkbenchError::UnsupportedNativeEncoding {
                    encoding: report.encoding.clone(),
                });
            }
            PathBuf::from(std::ffi::OsString::from_vec(bytes))
        };
        if !path.is_absolute() || path.to_str() != report.utf8.as_deref() {
            return Err(WorkbenchError::InvalidBinding {
                detail: "native path absolute/UTF-8 fields disagree".into(),
            });
        }
        Ok(path)
    }
    #[cfg(not(any()))]
    {
        Err(WorkbenchError::UnsupportedNativeEncoding {
            encoding: report.encoding.clone(),
        })
    }
}
