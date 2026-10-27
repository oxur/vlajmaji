//! Local, explicitly configured full-input topology comparison harness.
use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    Dictionary, PackId, RelationTarget, compose_dictionaries, load_sealed_release,
};
use vlajmaji_search::{DictionaryExactOptions, DictionarySearchEngine};
use vlajmaji_store::{
    DictionaryReadTopology, DictionarySnapshotManifest, DictionarySnapshotMember, DictionaryStore,
    DictionaryStoreVerification, DictionaryStoreVerificationCheck,
};

#[derive(Deserialize)]
struct PrepareConfig {
    generation: String,
    output_root: PathBuf,
    unified_state_sha256: String,
    unified_expected_population: BTreeMap<String, u64>,
    packs: Vec<PackInput>,
}
#[derive(Deserialize)]
struct PackInput {
    id: PackId,
    release: PathBuf,
    seal: PathBuf,
    expected_sha256: String,
    expected_population: BTreeMap<String, u64>,
    store: PathBuf,
}
#[derive(Serialize, Deserialize)]
struct RunConfig {
    generation: String,
    logical_state_sha256: String,
    members: Vec<DictionarySnapshotMember>,
    #[serde(default)]
    selected_release_ids: Option<Vec<vlajmaji_lexicon::EntityId>>,
}
#[derive(Clone, Deserialize)]
struct Request {
    id: String,
    #[serde(default = "default_operation")]
    operation: String,
    #[serde(default)]
    spelling: String,
    #[serde(default)]
    sources: Vec<String>,
    page_size: Option<usize>,
    #[serde(default)]
    cursor_from: Option<String>,
    #[serde(default)]
    prime: bool,
    #[serde(default)]
    target: Option<RelationTarget>,
    #[serde(default)]
    expected_error: Option<String>,
}

fn default_operation() -> String {
    "exact".into()
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn population(dictionary: &Dictionary) -> BTreeMap<String, u64> {
    let m = dictionary.metadata();
    let registry = dictionary.registry().as_document();
    let occurrences: usize = dictionary
        .captures()
        .iter()
        .map(|c| c.occurrences.len())
        .sum();
    let fields: usize = dictionary
        .captures()
        .iter()
        .flat_map(|c| &c.occurrences)
        .map(|o| o.fields.len())
        .sum();
    BTreeMap::from([
        ("assertions".into(), m.assertions.len() as u64),
        ("entries".into(), m.entries.len() as u64),
        ("native_fields".into(), fields as u64),
        ("occurrences".into(), occurrences as u64),
        ("captures".into(), dictionary.captures().len() as u64),
        ("contexts".into(), m.contexts.len() as u64),
        ("frames".into(), m.frames.len() as u64),
        ("forms".into(), m.forms.len() as u64),
        ("etymologies".into(), m.etymologies.len() as u64),
        ("relations".into(), m.relations.len() as u64),
        ("extensions".into(), m.extensions.len() as u64),
        ("sources".into(), registry.sources.len() as u64),
        ("contributors".into(), registry.contributors.len() as u64),
        ("distributors".into(), registry.distributors.len() as u64),
        ("aliases".into(), registry.aliases.len() as u64),
        ("lineage".into(), registry.lineage.len() as u64),
        ("snapshots".into(), registry.snapshots.len() as u64),
        ("collections".into(), m.collections.len() as u64),
        ("components".into(), m.components.len() as u64),
        ("releases".into(), m.releases.len() as u64),
        ("code_notices".into(), m.code_notices.len() as u64),
        ("owned_revisions".into(), m.owned_revisions.len() as u64),
        ("owned_heads".into(), m.owned_heads.len() as u64),
        ("editorial_senses".into(), m.editorial_senses.len() as u64),
        ("computations".into(), m.computations.len() as u64),
        ("annotations".into(), m.annotations.len() as u64),
    ])
}
fn compare_population(
    expected: &BTreeMap<String, u64>,
    actual: &BTreeMap<String, u64>,
) -> Result<(), String> {
    if expected == actual {
        return Ok(());
    }
    let categories = expected
        .keys()
        .chain(actual.keys())
        .collect::<std::collections::BTreeSet<_>>();
    if let Some(category) = categories
        .into_iter()
        .find(|category| expected.get(*category) != actual.get(*category))
    {
        Err(format!(
            "population mismatch for {category}: expected {:?}, actual {:?}",
            expected.get(category),
            actual.get(category)
        ))
    } else {
        Err("population maps differ".into())
    }
}
fn load(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(fs::read(path)?)
}
fn require_fresh(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(format!("refusing occupied candidate store {}", path.display()).into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
#[cfg(test)]
fn open_verified_existing(
    path: &Path,
    expected_state: &str,
) -> Result<DictionaryStore, Box<dyn std::error::Error>> {
    let report = DictionaryStoreVerification::verify(path, DictionaryStoreVerificationCheck::All)?;
    if !report.ok {
        return Err(format!("full verification failed for {}", path.display()).into());
    }
    let store = DictionaryStore::open(path)?;
    if store.recorded_state_sha256() != expected_state {
        return Err(format!(
            "existing store state mismatch: expected {expected_state}, found {}",
            store.recorded_state_sha256()
        )
        .into());
    }
    Ok(store)
}
fn render(value: &Value) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytes = serde_json::to_vec(value)?;
    let mut output = bytes;
    output.push(b'\n');
    Ok(output)
}
fn emit_bytes(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = io::stdout().lock();
    out.write_all(bytes)?;
    out.flush()?;
    Ok(())
}
fn prepare(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let cfg: PrepareConfig = serde_json::from_slice(&load(path)?)?;
    if cfg.packs.len() != 4 {
        return Err("prepare requires exactly four pinned packs".into());
    }
    if cfg.output_root.exists() {
        return Err(format!(
            "refusing occupied fresh output root {}",
            cfg.output_root.display()
        )
        .into());
    }
    fs::create_dir(&cfg.output_root)?;
    let mut dictionaries = Vec::new();
    let mut members = Vec::new();
    for input in &cfg.packs {
        eprintln!("stage=load-pack id={} start", input.id);
        let release = load(&input.release)?;
        let seal = load(&input.seal)?;
        if digest(&release) != input.expected_sha256 {
            return Err(format!("release hash mismatch for {}", input.id).into());
        }
        let dictionary = load_sealed_release(input.id, &release, &seal)?;
        let actual_population = population(&dictionary);
        compare_population(&input.expected_population, &actual_population)
            .map_err(|e| format!("{} {e}", input.id))?;
        let db_path = cfg.output_root.join(&input.store);
        eprintln!(
            "stage=store id={} path={} start",
            input.id,
            db_path.display()
        );
        let started = Instant::now();
        require_fresh(&db_path)?;
        let store = DictionaryStore::create(&db_path, &dictionary)?;
        let state = store.recorded_state_sha256().to_owned();
        drop(store);
        eprintln!(
            "stage=store id={} elapsed_ms={} state_sha256={}",
            input.id,
            started.elapsed().as_millis(),
            state
        );
        members.push(DictionarySnapshotMember {
            id: input.id.to_string(),
            path: db_path,
            expected_state_sha256: state,
        });
        dictionaries.push(dictionary);
    }
    eprintln!("stage=compose-unified start");
    let unified = compose_dictionaries(dictionaries)?;
    let unified_population = population(&unified);
    compare_population(&cfg.unified_expected_population, &unified_population)
        .map_err(|e| format!("unified {e}"))?;
    let canonical = unified.to_json_bytes()?;
    let actual = digest(&canonical);
    if actual != cfg.unified_state_sha256 {
        return Err(format!(
            "unified state mismatch expected {} got {actual}",
            cfg.unified_state_sha256
        )
        .into());
    }
    let unified_path = cfg.output_root.join("unified");
    eprintln!("stage=unified path={} start", unified_path.display());
    let started = Instant::now();
    require_fresh(&unified_path)?;
    let store = DictionaryStore::create(&unified_path, &unified)?;
    eprintln!(
        "stage=unified elapsed_ms={} state_sha256={}",
        started.elapsed().as_millis(),
        store.recorded_state_sha256()
    );
    let config = RunConfig {
        generation: cfg.generation.clone(),
        logical_state_sha256: actual,
        members,
        selected_release_ids: None,
    };
    fs::write(
        cfg.output_root.join("run-manifest.json"),
        serde_json::to_vec_pretty(&config)?,
    )?;
    eprintln!(
        "stage=prepare-complete unified={} members={}",
        unified_path.display(),
        config.members.len()
    );
    Ok(())
}
fn verify_existing(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let cfg: PrepareConfig = serde_json::from_slice(&load(path)?)?;
    if cfg.packs.is_empty() || !cfg.output_root.is_dir() {
        return Err(
            "verify-existing requires a present owned output root and at least one named pack"
                .into(),
        );
    }
    for input in &cfg.packs {
        let release = load(&input.release)?;
        let seal = load(&input.seal)?;
        if digest(&release) != input.expected_sha256 {
            return Err(format!("pinned input hash changed for {}", input.id).into());
        }
        let source = load_sealed_release(input.id, &release, &seal)?;
        compare_population(&input.expected_population, &population(&source))
            .map_err(|e| format!("{} {e}", input.id))?;
        let store_path = cfg.output_root.join(&input.store);
        eprintln!("stage=verify-existing id={} start", input.id);
        let started = Instant::now();
        let report = DictionaryStoreVerification::verify(
            &store_path,
            DictionaryStoreVerificationCheck::All,
        )?;
        if !report.ok {
            return Err(format!("full verification failed for {}", input.id).into());
        }
        let store = DictionaryStore::open(&store_path)?;
        let bytes = store.canonical_bytes()?;
        if bytes != release {
            return Err(
                format!("canonical bytes differ from pinned input for {}", input.id).into(),
            );
        }
        let stored = Dictionary::from_json_bytes(&bytes)?;
        if stored != source {
            return Err(format!("typed model differs from pinned input for {}", input.id).into());
        }
        if store.recorded_state_sha256() != input.expected_sha256 {
            return Err(format!("recorded state differs for {}", input.id).into());
        }
        eprintln!(
            "stage=verify-existing id={} elapsed_ms={} all_checks=passed bytes={} sha256={}",
            input.id,
            started.elapsed().as_millis(),
            bytes.len(),
            digest(&bytes)
        );
    }
    Ok(())
}
fn topology(value: &str) -> Result<DictionaryReadTopology, Box<dyn std::error::Error>> {
    match value {
        "unified" => Ok(DictionaryReadTopology::Unified),
        "attached" => Ok(DictionaryReadTopology::Attached),
        "fanout" => Ok(DictionaryReadTopology::Fanout),
        _ => Err(format!("unknown topology {value}").into()),
    }
}
fn run(
    manifest: &Path,
    mode: &str,
    request_path: Option<&Path>,
    request_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let run_started = Instant::now();
    let cfg: RunConfig = serde_json::from_slice(&load(manifest)?)?;
    let requests: Vec<Request> = if let Some(path) = request_path {
        serde_json::from_slice(&load(path)?)?
    } else {
        Vec::new()
    };
    let request = requests
        .into_iter()
        .find(|r| r.id == request_id)
        .ok_or("request id not found")?;
    if request.cursor_from.is_some() || request.prime {
        return Err("run accepts one direct request without batch references".into());
    }
    let (engine, open_profile) = open_engine(manifest, &cfg, mode)?;
    eprintln!("stage=open topology={mode} profile={open_profile:?}");
    let output = execute_request(&engine, &request, None)?;
    if let Some(profile) = output.exact_profile {
        eprintln!("stage=exact request={} profile={profile:?}", request.id);
    } else {
        eprintln!(
            "stage=reference request={} elapsed_us={}",
            request.id,
            output.lookup_elapsed.as_micros()
        );
    }
    eprintln!(
        "stage=value request={} elapsed_us={}",
        request.id,
        output.value_elapsed.as_micros()
    );
    let started = Instant::now();
    let bytes = render(&output.value)?;
    eprintln!(
        "stage=serialize request={} elapsed_us={} bytes={} sha256={}",
        request.id,
        started.elapsed().as_micros(),
        bytes.len(),
        digest(&bytes)
    );
    emit_bytes(&bytes)?;
    eprintln!(
        "stage=run-total elapsed_us={}",
        run_started.elapsed().as_micros()
    );
    Ok(())
}

fn open_engine(
    manifest: &Path,
    cfg: &RunConfig,
    mode: &str,
) -> Result<
    (
        DictionarySearchEngine,
        vlajmaji_search::DictionarySnapshotOpenProfile,
    ),
    Box<dyn std::error::Error>,
> {
    let mut members = cfg.members.clone();
    if mode == "unified" {
        members = vec![DictionarySnapshotMember {
            id: "unified".into(),
            path: manifest
                .parent()
                .ok_or("manifest parent missing")?
                .join("unified"),
            expected_state_sha256: cfg.logical_state_sha256.clone(),
        }];
    }
    let expected = cfg.logical_state_sha256.clone();
    let snapshot_manifest = DictionarySnapshotManifest {
        version: 1,
        generation: cfg.generation.clone(),
        logical_state_sha256: expected,
        members,
        selected_release_ids: cfg.selected_release_ids.clone(),
    };
    eprintln!("stage=open topology={mode} start");
    let started = Instant::now();
    let (engine, open_profile) =
        DictionarySearchEngine::open_snapshot_profiled(topology(mode)?, snapshot_manifest)?;
    eprintln!(
        "stage=open topology={mode} elapsed_ms={} profile={open_profile:?}",
        started.elapsed().as_millis()
    );
    Ok((engine, open_profile))
}

fn query(
    engine: &DictionarySearchEngine,
    request: &Request,
    cursor: Option<String>,
) -> Result<
    (
        vlajmaji_search::DictionaryExactResponse,
        vlajmaji_search::DictionaryExactProfile,
    ),
    Box<dyn std::error::Error>,
> {
    let opts = DictionaryExactOptions {
        sources: request.sources.clone(),
        page_size: request.page_size,
        cursor,
        ..Default::default()
    };
    Ok(engine.exact_profiled(&request.spelling, opts)?)
}

struct ExecutedRequest {
    value: Value,
    next_cursor: Option<String>,
    exact_profile: Option<vlajmaji_search::DictionaryExactProfile>,
    lookup_elapsed: std::time::Duration,
    value_elapsed: std::time::Duration,
}

fn execute_request(
    engine: &DictionarySearchEngine,
    request: &Request,
    cursor: Option<String>,
) -> Result<ExecutedRequest, Box<dyn std::error::Error>> {
    let (value, next_cursor, exact_profile, lookup_elapsed, value_elapsed) = match request
        .operation
        .as_str()
    {
        "exact" => {
            if request.expected_error.is_some() {
                return Err("expected_error is supported only for reference requests".into());
            }
            let (response, profile) = query(engine, request, cursor)?;
            let elapsed = profile.total;
            let next_cursor = response.next_cursor.clone();
            let value_started = Instant::now();
            let value = serde_json::to_value(response)?;
            (
                value,
                next_cursor,
                Some(profile),
                elapsed,
                value_started.elapsed(),
            )
        }
        "reference" => {
            if cursor.is_some() {
                return Err("reference requests cannot consume an exact-query cursor".into());
            }
            let target = request
                .target
                .as_ref()
                .ok_or("reference request requires target")?;
            let started = Instant::now();
            let result = engine.resolve(target);
            let elapsed = started.elapsed();
            let value_started = Instant::now();
            let value = match result {
                Ok(response) if request.expected_error.is_none() => serde_json::to_value(response)?,
                Ok(_) => {
                    return Err(format!(
                        "request {} expected error {} but resolved successfully",
                        request.id,
                        request.expected_error.as_deref().unwrap_or("<missing>")
                    )
                    .into());
                }
                Err(error) => {
                    let report = error.report();
                    if request.expected_error.as_deref() != Some(report.code.as_str()) {
                        return Err(error.into());
                    }
                    serde_json::json!({"expected_error": true, "error": report})
                }
            };
            (value, None, None, elapsed, value_started.elapsed())
        }
        other => return Err(format!("unsupported request operation {other}").into()),
    };
    Ok(ExecutedRequest {
        value,
        next_cursor,
        exact_profile,
        lookup_elapsed,
        value_elapsed,
    })
}

fn batch(
    manifest: &Path,
    mode: &str,
    request_path: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let cfg: RunConfig = serde_json::from_slice(&load(manifest)?)?;
    let requests: Vec<Request> = serde_json::from_slice(&load(request_path)?)?;
    let primes: Vec<_> = requests.iter().filter(|request| request.prime).collect();
    let measured: Vec<_> = requests.iter().filter(|request| !request.prime).collect();
    if primes.is_empty() || measured.is_empty() {
        return Err("batch requires at least one priming and one measured request".into());
    }
    if output_dir.exists() {
        return Err(format!("refusing occupied batch output {}", output_dir.display()).into());
    }
    fs::create_dir(output_dir)?;
    let (engine, open_profile) = open_engine(manifest, &cfg, mode)?;
    eprintln!("stage=batch-open profile={open_profile:?}");
    let mut cursors = BTreeMap::<String, Option<String>>::new();
    for request in &primes {
        if cursors.contains_key(&request.id) {
            return Err(format!("duplicate batch request id {}", request.id).into());
        }
        let cursor = resolve_cursor(request, &cursors)?;
        let executed = execute_request(&engine, request, cursor)?;
        cursors.insert(request.id.clone(), executed.next_cursor);
        if let Some(profile) = executed.exact_profile {
            eprintln!("stage=prime request={} profile={profile:?}", request.id);
        } else {
            eprintln!(
                "stage=prime-reference request={} elapsed_us={}",
                request.id,
                executed.lookup_elapsed.as_micros()
            );
        }
    }
    let measured_pass_started = Instant::now();
    let mut summary = Vec::with_capacity(measured.len());
    for (index, request) in measured.into_iter().enumerate() {
        if cursors.contains_key(&request.id) {
            return Err(format!("duplicate batch request id {}", request.id).into());
        }
        let cursor = resolve_cursor(request, &cursors)?;
        let executed = execute_request(&engine, request, cursor)?;
        let next_cursor = executed.next_cursor;
        let value = executed.value;
        let value_elapsed = executed.value_elapsed;
        let serialize_started = Instant::now();
        let bytes = render(&value)?;
        let serialize_elapsed = serialize_started.elapsed();
        let output_path = output_dir.join(format!("response-{index:03}.json"));
        let write_started = Instant::now();
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output_path)?;
        output.write_all(&bytes)?;
        output.flush()?;
        let write_elapsed = write_started.elapsed();
        let state = digest(&bytes);
        cursors.insert(request.id.clone(), next_cursor);
        if let Some(profile) = executed.exact_profile {
            let warm_lookup_us =
                profile.candidate_sql.as_micros() + profile.lookup_order_paging.as_micros();
            let hydration_and_serialization_us = profile.hydration.as_micros()
                + value_elapsed.as_micros()
                + serialize_elapsed.as_micros();
            eprintln!(
                "stage=measure request={} profile={profile:?} warm_lookup_us={} hydration_value_serialization_us={} value_us={} serialize_us={} write_us={} bytes={} sha256={state}",
                request.id,
                warm_lookup_us,
                hydration_and_serialization_us,
                value_elapsed.as_micros(),
                serialize_elapsed.as_micros(),
                write_elapsed.as_micros(),
                bytes.len()
            );
        } else {
            eprintln!(
                "stage=measure-reference request={} lookup_resolve_us={} value_us={} serialize_us={} write_us={} bytes={} sha256={state}",
                request.id,
                executed.lookup_elapsed.as_micros(),
                value_elapsed.as_micros(),
                serialize_elapsed.as_micros(),
                write_elapsed.as_micros(),
                bytes.len()
            );
        }
        summary.push(serde_json::json!({
            "id": request.id,
            "path": output_path,
            "bytes": bytes.len(),
            "sha256": state,
        }));
    }
    eprintln!(
        "stage=measured-pass requests={} elapsed_us={}",
        summary.len(),
        measured_pass_started.elapsed().as_micros()
    );
    emit_bytes(&render(&serde_json::json!({"responses": summary}))?)
}

fn resolve_cursor(
    request: &Request,
    cursors: &BTreeMap<String, Option<String>>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    match request.cursor_from.as_deref() {
        None => Ok(None),
        Some(id) => {
            cursors.get(id).cloned().flatten().map(Some).ok_or_else(|| {
                format!("request {} has no next cursor from {id}", request.id).into()
            })
        }
    }
}
fn main() {
    let result: Result<(), Box<dyn std::error::Error>> = {
        let args: Vec<String> = env::args().skip(1).collect();
        match args.first().map(String::as_str) {
            Some("prepare") if args.len() == 3 && args[1] == "--manifest" => prepare(Path::new(&args[2])),
            Some("verify-existing") if args.len() == 3 && args[1] == "--manifest" => verify_existing(Path::new(&args[2])),
            Some("run") if args.len() == 9 && args[1] == "--manifest" && args[3] == "--topology" && args[5] == "--requests" && args[7] == "--request" => run(Path::new(&args[2]), &args[4], Some(Path::new(&args[6])), &args[8]),
            Some("batch") if args.len() == 9 && args[1] == "--manifest" && args[3] == "--topology" && args[5] == "--requests" && args[7] == "--output-dir" => batch(Path::new(&args[2]), &args[4], Path::new(&args[6]), Path::new(&args[8])),
            _ => Err("usage: topology_probe prepare --manifest FILE | run --manifest FILE --topology unified|attached|fanout --requests FILE --request ID | batch --manifest FILE --topology unified|attached|fanout --requests FILE --output-dir DIR".into()),
        }
    };
    if let Err(error) = result {
        eprintln!("topology_probe: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BTreeMap, Request, RunConfig, Value, batch, compare_population, digest,
        open_verified_existing, require_fresh, resolve_cursor,
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };
    use vlajmaji_lexicon::{
        Dictionary, JbotciImportOptions, LocalRef, RelationTarget, compose_dictionaries,
        import_jbotci_export,
    };
    use vlajmaji_store::{DictionarySnapshotMember, DictionaryStore};
    fn counts(rows: &[(&str, u64)]) -> BTreeMap<String, u64> {
        rows.iter().map(|(k, v)| (String::from(*k), *v)).collect()
    }
    #[test]
    fn test_population_comparator_accepts_empty_and_reordered_exact_categories() {
        assert!(compare_population(&counts(&[]), &counts(&[])).is_ok());
        assert!(
            compare_population(
                &counts(&[("assertions", 2), ("forms", 0), ("native_fields", 3)]),
                &counts(&[("native_fields", 3), ("forms", 0), ("assertions", 2)])
            )
            .is_ok()
        );
    }
    #[test]
    fn test_population_comparator_rejects_changed_multiplicity_and_missing_empty_category() {
        assert!(
            compare_population(&counts(&[("assertions", 2)]), &counts(&[("assertions", 3)]))
                .is_err()
        );
        assert!(
            compare_population(
                &counts(&[("assertions", 2), ("forms", 0)]),
                &counts(&[("assertions", 2)])
            )
            .is_err()
        );
        assert!(
            compare_population(
                &counts(&[("assertions", 2)]),
                &counts(&[("assertions", 2), ("forms", 0)])
            )
            .is_err()
        );
        assert!(compare_population(&counts(&[]), &counts(&[("forms", 0)])).is_err());
    }
    #[test]
    fn test_candidate_path_controls_refuse_occupied_or_missing_without_mutation() {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-probe-resume-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let path: PathBuf = root.join("tiny");
        let dictionary = Dictionary::new();
        let bytes = dictionary.to_json_bytes().unwrap();
        let expected = format!("{:x}", Sha256::digest(&bytes));
        let store = DictionaryStore::create(&path, &dictionary).unwrap();
        drop(store);
        assert!(require_fresh(&path).is_err());
        let db = path.join("dictionary.sqlite3");
        let before = Sha256::digest(fs::read(&db).unwrap());
        assert!(
            open_verified_existing(
                &path,
                "0000000000000000000000000000000000000000000000000000000000000000"
            )
            .is_err()
        );
        assert!(open_verified_existing(&root.join("unknown"), &expected).is_err());
        assert!(open_verified_existing(&path, &expected).is_ok());
        let after = Sha256::digest(fs::read(&db).unwrap());
        assert_eq!(before, after);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_batch_cursor_reference_requires_a_prior_measured_page() {
        let prior = BTreeMap::from([(String::from("page-one"), Some(String::from("cursor")))]);
        let page_two = Request {
            id: "page-two".into(),
            operation: "exact".into(),
            spelling: "farvi".into(),
            sources: Vec::new(),
            page_size: Some(1),
            cursor_from: Some("page-one".into()),
            prime: false,
            target: None,
            expected_error: None,
        };
        assert_eq!(
            resolve_cursor(&page_two, &prior).unwrap(),
            Some(String::from("cursor"))
        );
        let missing = Request {
            cursor_from: Some("not-run".into()),
            ..page_two.clone()
        };
        assert!(resolve_cursor(&missing, &prior).is_err());
        let exhausted = BTreeMap::from([(String::from("page-one"), None)]);
        assert!(resolve_cursor(&page_two, &exhausted).is_err());
    }

    #[test]
    fn test_batch_primes_exact_queries_then_measures_a_reference_page_chain() {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-probe-batch-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let raw = br#"[{"word":"farvi","word_type":"gismu","definition":"to grow"},{"word":"farvi","word_type":"gismu","definition":"to develop"}]"#;
        let metadata = format!(
            "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 2\nentry_count = 2\n",
            digest(raw)
        )
        .into_bytes();
        let dictionary = import_jbotci_export(
            raw,
            &metadata,
            &JbotciImportOptions {
                revision: "0000000000000000000000000000000000000000".into(),
                expected_raw_sha256: digest(raw),
                expected_metadata_sha256: digest(&metadata),
            },
        )
        .unwrap()
        .into_dictionary();
        let logical = compose_dictionaries(vec![dictionary.clone(), dictionary.clone()]).unwrap();
        let expected_state = digest(&logical.to_json_bytes().unwrap());
        let reference_target =
            RelationTarget::Local(LocalRef::Entry(logical.metadata().entries[0].id.clone()));
        let wrong_role_target =
            RelationTarget::Local(LocalRef::Entry(logical.metadata().assertions[0].id.clone()));
        let mut members = Vec::new();
        for index in 0..2 {
            let path = root.join(format!("member-{index}"));
            DictionaryStore::create(&path, &dictionary).unwrap();
            members.push(DictionarySnapshotMember {
                id: format!("member-{index}"),
                path,
                expected_state_sha256: digest(&dictionary.to_json_bytes().unwrap()),
            });
        }
        let unified = root.join("unified");
        DictionaryStore::create(&unified, &logical).unwrap();
        let manifest = root.join("run.json");
        fs::write(
            &manifest,
            serde_json::to_vec(&RunConfig {
                generation: "tiny-batch".into(),
                logical_state_sha256: expected_state,
                members,
                selected_release_ids: None,
            })
            .unwrap(),
        )
        .unwrap();
        let requests = root.join("requests.json");
        fs::write(
            &requests,
            serde_json::to_vec(&serde_json::json!([
                {"id":"prime-first","spelling":"farvi","page_size":1,"prime":true},
                {"id":"prime-next","spelling":"farvi","page_size":1,"prime":true,"cursor_from":"prime-first"},
                {"id":"prime-reference","operation":"reference","prime":true,"target":reference_target.clone()},
                {"id":"measure-first","spelling":"farvi","page_size":1},
                {"id":"measure-next","spelling":"farvi","page_size":1,"cursor_from":"measure-first"},
                {"id":"measure-wrong-role","operation":"reference","target":wrong_role_target,"expected_error":"invalid_reference"},
                {"id":"measure-reference","operation":"reference","target":reference_target}
            ]))
            .unwrap(),
        )
        .unwrap();
        let output = root.join("responses");
        batch(&manifest, "fanout", &requests, &output).unwrap();
        let first: Value =
            serde_json::from_slice(&fs::read(output.join("response-000.json")).unwrap()).unwrap();
        let second: Value =
            serde_json::from_slice(&fs::read(output.join("response-001.json")).unwrap()).unwrap();
        let wrong_role: Value =
            serde_json::from_slice(&fs::read(output.join("response-002.json")).unwrap()).unwrap();
        let reference: Value =
            serde_json::from_slice(&fs::read(output.join("response-003.json")).unwrap()).unwrap();
        assert_eq!(first["groups"].as_array().unwrap().len(), 1);
        assert_eq!(second["groups"].as_array().unwrap().len(), 1);
        assert_eq!(reference["status"], "Resolved");
        assert_eq!(wrong_role["expected_error"], true);
        assert_eq!(wrong_role["error"]["code"], "invalid_reference");
        assert_ne!(
            first["groups"][0]["hits"][0]["occurrence_id"],
            second["groups"][0]["hits"][0]["occurrence_id"]
        );
        fs::remove_dir_all(root).unwrap();
    }
}
