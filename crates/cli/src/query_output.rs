//! Private transport boundary for query responses.

/// A query result before it is written to stdout.
pub(crate) enum QueryOutput {
    /// JSON is the only initial S05 output mode; readable rendering is a later pass.
    Json(serde_json::Value),
}

impl QueryOutput {
    pub(crate) fn as_json(&self) -> &serde_json::Value {
        match self {
            Self::Json(value) => value,
        }
    }
}
