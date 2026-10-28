//! Private transport boundary for query responses.

/// A query result before it is written to stdout.
pub(crate) enum QueryOutput {
    /// Existing JSON output, retained as a typed Value for byte-compatible writing.
    Json(serde_json::Value),
    /// Already-rendered deterministic UTF-8 text.
    Readable(String),
}
