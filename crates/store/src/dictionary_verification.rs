//! Explicit, read-only verification for persisted dictionary stores.

use std::{collections::BTreeSet, path::Path};

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    Dictionary, DictionaryDocument, RegistryDocument, UnverifiedDictionaryDocument,
};

use crate::dictionary::{
    self, DictionaryStoreError, check_foreign_keys, check_projection, check_schema, read_canonical,
    read_metadata,
};

/// A verification scope accepted by the API and command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DictionaryStoreVerificationCheck {
    /// Compare the exact versioned SQLite schema.
    Schema,
    /// Validate the header and its agreement with canonical metadata.
    Metadata,
    /// Compare the canonical and capture byte digests and legacy column.
    Hashes,
    /// Apply the complete strict dictionary model validation.
    Model,
    /// Run SQLite's foreign-key check.
    Fks,
    /// Compare every relational row and cell with the canonical model.
    Projection,
    /// Run every check in schema, metadata, hashes, model, fks, projection order.
    All,
}

/// Outcome of one requested verification check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DictionaryStoreCheckStatus {
    /// The named invariant was checked and passed.
    Passed,
    /// The named invariant was checked and failed.
    Failed,
    /// The check could not run because an explicit prerequisite was unavailable.
    Blocked,
}

/// One check outcome in a verification report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictionaryStoreVerificationResult {
    /// Stable check name.
    pub check: String,
    /// Whether the check passed, failed, or was blocked.
    pub status: DictionaryStoreCheckStatus,
    /// Concise diagnostic or success detail.
    pub diagnostic: String,
}

/// Result of a selected explicit store verification operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictionaryStoreVerificationReport {
    /// Operation name.
    pub operation: String,
    /// Verification report version.
    pub version: u32,
    /// True only when every emitted check row passed.
    pub ok: bool,
    /// Selected result rows; `all` has six rows in fixed order.
    pub checks: Vec<DictionaryStoreVerificationResult>,
    /// Outcomes of prerequisites used by selected checks.
    pub prerequisites: Vec<DictionaryStoreVerificationResult>,
}

/// Explicit verification of one persisted grouped dictionary store.
#[derive(Debug, Default)]
pub struct DictionaryStoreVerification;

#[derive(Clone)]
struct Cached {
    metadata: Result<std::collections::BTreeMap<String, String>, String>,
    canonical: Result<(Vec<u8>, Option<Vec<u8>>), String>,
    draft: Result<UnverifiedDictionaryDocument, String>,
    model: Result<Dictionary, String>,
}

impl DictionaryStoreVerification {
    /// Verify the requested scope on one independent read-only SQLite snapshot.
    ///
    /// # Errors
    ///
    /// Returns a filesystem or SQLite error when the store cannot be opened.
    pub fn verify(
        path: impl AsRef<Path>,
        requested: DictionaryStoreVerificationCheck,
    ) -> Result<DictionaryStoreVerificationReport, DictionaryStoreError> {
        let path = path.as_ref();
        let database_path = path.join("dictionary.sqlite3");
        if !database_path.is_file() {
            return Err(DictionaryStoreError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("expected {}", database_path.display()),
            )));
        }
        let connection = Connection::open_with_flags(
            &database_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        connection.execute_batch("BEGIN DEFERRED;")?;
        let checks: Vec<DictionaryStoreVerificationCheck> = match requested {
            DictionaryStoreVerificationCheck::All => vec![
                DictionaryStoreVerificationCheck::Schema,
                DictionaryStoreVerificationCheck::Metadata,
                DictionaryStoreVerificationCheck::Hashes,
                DictionaryStoreVerificationCheck::Model,
                DictionaryStoreVerificationCheck::Fks,
                DictionaryStoreVerificationCheck::Projection,
            ],
            check => vec![check],
        };
        let needs_metadata = checks.iter().any(|check| {
            matches!(
                check,
                DictionaryStoreVerificationCheck::Metadata
                    | DictionaryStoreVerificationCheck::Hashes
            )
        });
        let needs_canonical = checks.iter().any(|check| {
            !matches!(
                check,
                DictionaryStoreVerificationCheck::Schema | DictionaryStoreVerificationCheck::Fks
            )
        });
        let needs_draft = checks.iter().any(|check| {
            matches!(
                check,
                DictionaryStoreVerificationCheck::Metadata
                    | DictionaryStoreVerificationCheck::Hashes
            )
        });
        let needs_model = checks.iter().any(|check| {
            matches!(
                check,
                DictionaryStoreVerificationCheck::Model
                    | DictionaryStoreVerificationCheck::Projection
            )
        });
        let metadata = if needs_metadata {
            read_metadata(&connection).map_err(|e| e.to_string())
        } else {
            Err("not required for selected scope".into())
        };
        let canonical = if needs_canonical {
            read_canonical(&connection).map_err(|e| e.to_string())
        } else {
            Err("not required for selected scope".into())
        };
        let draft = if needs_draft {
            canonical
                .as_ref()
                .map_err(Clone::clone)
                .and_then(|(bytes, _)| {
                    DictionaryDocument::<RegistryDocument>::from_unverified_json_bytes(bytes)
                        .map_err(|e| e.to_string())
                })
        } else {
            Err("not required for selected scope".into())
        };
        let model = if needs_model {
            canonical
                .as_ref()
                .map_err(Clone::clone)
                .and_then(|(bytes, _)| {
                    Dictionary::from_json_bytes(bytes).map_err(|e| e.to_string())
                })
        } else {
            Err("not required for selected scope".into())
        };
        let cached = Cached {
            metadata,
            canonical,
            draft,
            model,
        };
        let mut prerequisites = Vec::new();
        let mut results = Vec::new();
        for check in checks {
            let result = run_check(&connection, check, &cached, &mut prerequisites);
            results.push(result);
        }
        let ok = results
            .iter()
            .all(|row| row.status == DictionaryStoreCheckStatus::Passed);
        Ok(DictionaryStoreVerificationReport {
            operation: "verify".into(),
            version: 1,
            ok,
            checks: results,
            prerequisites,
        })
    }
}

fn result(
    check: &str,
    status: DictionaryStoreCheckStatus,
    diagnostic: impl Into<String>,
) -> DictionaryStoreVerificationResult {
    DictionaryStoreVerificationResult {
        check: check.into(),
        status,
        diagnostic: diagnostic.into(),
    }
}

fn run_check(
    connection: &Connection,
    check: DictionaryStoreVerificationCheck,
    cached: &Cached,
    prerequisites: &mut Vec<DictionaryStoreVerificationResult>,
) -> DictionaryStoreVerificationResult {
    use DictionaryStoreCheckStatus::{Blocked, Failed, Passed};
    use DictionaryStoreVerificationCheck as C;
    let name = match check {
        C::Schema => "schema",
        C::Metadata => "metadata",
        C::Hashes => "hashes",
        C::Model => "model",
        C::Fks => "fks",
        C::Projection => "projection",
        C::All => "all",
    };
    let outcome = match check {
        C::Schema => check_schema(connection)
            .map(|()| "exact schema catalog matches".to_owned())
            .map_err(|e| e.to_string()),
        C::Metadata => match verify_metadata_header(cached) {
            Err(error) => Err(error),
            Ok(()) => match &cached.draft {
                Err(error) => {
                    prerequisites.push(result("canonical_document", Blocked, error.clone()));
                    return result(
                        name,
                        Blocked,
                        "canonical typed draft prerequisite is unavailable",
                    );
                }
                Ok(draft) => verify_metadata_draft(cached, draft),
            },
        },
        C::Hashes => match verify_hash_header(cached) {
            Err(error) => Err(error),
            Ok(()) => match &cached.draft {
                Err(error) => {
                    prerequisites.push(result("canonical_document", Blocked, error.clone()));
                    return result(
                        name,
                        Blocked,
                        "canonical typed draft prerequisite is unavailable",
                    );
                }
                Ok(draft) => verify_hash_draft(cached, draft),
            },
        },
        C::Model => cached
            .model
            .as_ref()
            .map(|_| "strict dictionary model validation passed".to_owned())
            .map_err(Clone::clone),
        C::Fks => check_foreign_keys(connection)
            .map(|()| "foreign_key_check returned no violations".to_owned())
            .map_err(|e| e.to_string()),
        C::Projection => match (&cached.model, &cached.canonical) {
            (Ok(model), Ok((bytes, _))) => check_projection(connection, model, bytes)
                .map(|()| "all persisted rows and cells match the canonical model".to_owned())
                .map_err(|e| e.to_string()),
            (Err(error), _) => {
                prerequisites.push(result(
                    "model",
                    Blocked,
                    format!("projection requires a validated model: {error}"),
                ));
                return result(name, Blocked, "validated model prerequisite is unavailable");
            }
            (_, Err(error)) => {
                prerequisites.push(result(
                    "canonical",
                    Blocked,
                    format!("projection requires readable canonical bytes: {error}"),
                ));
                return result(name, Blocked, "canonical bytes prerequisite is unavailable");
            }
        },
        C::All => Err("all is an aggregate scope and cannot be evaluated as one check".into()),
    };
    match outcome {
        Ok(diagnostic) => result(name, Passed, diagnostic),
        Err(diagnostic) => result(name, Failed, diagnostic),
    }
}

fn verify_metadata_header(cached: &Cached) -> Result<(), String> {
    let metadata = cached.metadata.as_ref().map_err(Clone::clone)?;
    let expected: BTreeSet<&str> = [
        "format",
        "schema_version",
        "complete",
        "canonical_sha256",
        "dictionary_version",
        "registry_version",
        "catalogue_revision",
    ]
    .into_iter()
    .collect();
    let actual: BTreeSet<&str> = metadata.keys().map(String::as_str).collect();
    if actual != expected {
        return Err("metadata key set differs from the required version-1 header".into());
    }
    dictionary::check_metadata(metadata).map_err(|e| e.to_string())?;
    let hash = metadata
        .get("canonical_sha256")
        .map(String::as_str)
        .unwrap_or("");
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("canonical_sha256 is not 64 lowercase hexadecimal characters".into());
    }
    let dictionary_version = metadata
        .get("dictionary_version")
        .and_then(|value| value.parse::<u32>().ok());
    let registry_version = metadata
        .get("registry_version")
        .and_then(|value| value.parse::<u32>().ok());
    if dictionary_version.is_none()
        || registry_version.is_none()
        || metadata
            .get("catalogue_revision")
            .and_then(|value| value.parse::<u64>().ok())
            .is_none()
    {
        return Err(
            "version or catalogue revision metadata is not valid unsigned integer text".into(),
        );
    }
    if dictionary_version != Some(1) {
        return Err("dictionary_version is not supported (expected 1)".into());
    }
    if registry_version != Some(1) {
        return Err("registry_version is not supported (expected 1)".into());
    }
    Ok(())
}

fn verify_metadata_draft(
    cached: &Cached,
    draft: &UnverifiedDictionaryDocument,
) -> Result<String, String> {
    let metadata = cached.metadata.as_ref().map_err(Clone::clone)?;
    for (key, actual) in [
        ("dictionary_version", draft.version.to_string()),
        ("registry_version", draft.registry.version.to_string()),
        (
            "catalogue_revision",
            draft.metadata.catalogue_revision.to_string(),
        ),
    ] {
        if metadata.get(key) != Some(&actual) {
            return Err(format!("{key} does not agree with the canonical document"));
        }
    }
    if metadata
        .get("schema_version")
        .and_then(|x| x.parse::<u32>().ok())
        != Some(1)
    {
        return Err("schema_version is not supported".into());
    }
    Ok("metadata shape, versions, and canonical header agreement passed".into())
}

fn verify_hash_header(cached: &Cached) -> Result<(), String> {
    let metadata = cached.metadata.as_ref().map_err(Clone::clone)?;
    let (bytes, legacy) = cached.canonical.as_ref().map_err(Clone::clone)?;
    let expected = metadata
        .get("canonical_sha256")
        .ok_or("canonical_sha256 is missing")?;
    let actual = format!("{:x}", Sha256::digest(bytes));
    if &actual != expected {
        return Err("canonical document digest differs from metadata".into());
    }
    let _ = legacy;
    Ok(())
}

fn verify_hash_draft(
    cached: &Cached,
    draft: &UnverifiedDictionaryDocument,
) -> Result<String, String> {
    let (_, legacy) = cached.canonical.as_ref().map_err(Clone::clone)?;
    for capture in &draft.captures {
        let digest = format!("{:x}", Sha256::digest(&capture.bytes));
        if digest != capture.identity.sha256 {
            return Err(format!(
                "capture digest mismatch for {}",
                capture.identity.namespace
            ));
        }
    }
    if draft.legacy_archive.as_deref() != legacy.as_deref() {
        return Err("legacy archive column differs from canonical document".into());
    }
    Ok(format!(
        "canonical digest, {} capture digests, and legacy archive column passed",
        draft.captures.len()
    ))
}
