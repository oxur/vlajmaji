// Prospective independent S04 phase1 caller. Not compiled/executed before source freeze.
include!("helpers.rs");
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    process::Command,
};
use vlajmaji_search::{
    CollectionQueryError, CollectionQueryResult, CollectionQuerySelection, CollectionSearchEngine,
    DictionaryExactOptions, DictionaryRequest, ReferenceStatus,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionGeneration, CollectionInput, CollectionRequest,
    CollectionSelection, DictionaryStore,
};
const IDS: [PackId; 4] = [
    PackId::LensiskuJbotci,
    PackId::Llg,
    PackId::LaLojban,
    PackId::Sutysisku,
];
type Seals = Vec<(PackId, Vec<u8>, Vec<u8>)>;
fn sealed(ds: &[Dictionary]) -> Seals {
    IDS.iter()
        .zip(ds)
        .map(|(id, d)| {
            let x = seal_release(*id, PackPayloadKind::Synthetic, d).unwrap();
            (*id, x.release, x.seal)
        })
        .collect()
}
fn req(xs: &[(PackId, Vec<u8>, Vec<u8>)]) -> CollectionRequest<'_> {
    CollectionRequest::new(
        xs.iter()
            .map(|(id, release, seal)| CollectionInput::SealedBytes {
                pack_id: *id,
                release: Cow::Borrowed(release),
                seal: Cow::Borrowed(seal),
            })
            .collect(),
        CollectionSelection::Packs(xs.iter().map(|x| x.0).collect()),
        true,
    )
    .unwrap()
}
fn state(root: &Path) -> (u32, u64, Option<CollectionGeneration>) {
    let c = CollectionCatalogue::open(root).unwrap();
    (
        c.schema_version().unwrap(),
        c.local_revision().unwrap(),
        c.generation().unwrap(),
    )
}
fn db(root: &Path) -> String {
    sha(&std::fs::read(root.join("catalogue.sqlite3")).unwrap())
}
fn expected(ds: &[Dictionary]) -> BTreeMap<&'static str, BTreeMap<String, usize>> {
    let mut all: BTreeMap<_, BTreeMap<_, usize>> = BTreeMap::new();
    for d in ds {
        for (k, rows) in inventory(&draft(d.clone())) {
            let out = all.entry(k).or_default();
            for (k, n) in rows {
                let old = out.entry(k).or_default();
                *old = (*old).max(n);
            }
        }
    }
    all
}
fn engine(root: &Path) -> CollectionSearchEngine {
    CollectionSearchEngine::open(
        root,
        CollectionQuerySelection {
            packs: IDS.to_vec(),
            disabled: vec![],
            profile: None,
        },
    )
    .unwrap()
}
fn page(size: Option<usize>, cursor: Option<String>) -> DictionaryRequest {
    DictionaryRequest::Exact {
        spelling: "farvi".into(),
        options: DictionaryExactOptions {
            page_size: size,
            cursor,
            ..Default::default()
        },
    }
}
fn store_inventory(g: &CollectionGeneration) -> BTreeMap<&'static str, BTreeMap<String, usize>> {
    let s = DictionaryStore::open(&g.store_path).unwrap();
    inventory(&s.unverified_document().unwrap())
}
fn target(d: &Dictionary) -> RelationTarget {
    let a = d
        .metadata()
        .assertions
        .iter()
        .find(|a| a.eligibility == Eligibility::Lexical)
        .unwrap();
    let e = d
        .metadata()
        .entries
        .iter()
        .find(|e| e.id == a.entry)
        .unwrap();
    let r = d
        .metadata()
        .releases
        .iter()
        .find(|r| r.assertions.contains(&a.id))
        .unwrap();
    let o = d
        .registry()
        .as_document()
        .occurrences
        .iter()
        .find(|o| o.occurrence == a.occurrence.to_record_identity().unwrap())
        .unwrap();
    RelationTarget::Catalog(CatalogTarget {
        source: e.source.clone(),
        release: r.id.clone(),
        native_entry_id: o.native_entry_id.clone(),
        native_assertion_id: o.native_assertion_id.clone(),
        occurrence: Some(a.occurrence.clone()),
    })
}
fn old_cli(binary: &Path, args: &[&std::ffi::OsStr], stdout: &Path) {
    let out = Command::new(binary)
        .args(args)
        .env_clear()
        .output()
        .unwrap();
    std::fs::write(stdout, &out.stdout).unwrap();
    std::fs::write(stdout.with_extension("stderr"), &out.stderr).unwrap();
    assert!(
        out.status.success(),
        "old CLI {:?}: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}
fn old_root(binary: &Path, manifest: &Path, root: &Path, out: &Path, register: bool) {
    use std::ffi::OsStr;
    old_cli(
        binary,
        &[
            OsStr::new("collections"),
            OsStr::new(if register { "register" } else { "install" }),
            root.as_os_str(),
            OsStr::new("--manifest"),
            manifest.as_os_str(),
            OsStr::new("--all"),
            OsStr::new("--allow-synthetic"),
        ],
        out,
    );
}
fn schema_audit(root: &Path) {
    use rusqlite::{Connection, OpenFlags};
    let c = Connection::open_with_flags(
        root.join("catalogue.sqlite3"),
        OpenFlags::SQLITE_OPEN_READ_WRITE,
    )
    .unwrap();
    let before = db(root);
    let physical = [
        "catalogue_meta_v2",
        "registrations_v2",
        "registration_locations_v2",
        "generations_v2",
        "installed_heads_v2",
        "attempts_v2",
    ];
    let mut q=c.prepare("SELECT name,type FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%' AND type IN ('table','view') ORDER BY name").unwrap();
    let rows = q
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect::<BTreeMap<_, _>>();
    let mut exp = physical
        .into_iter()
        .map(|x| (x.to_owned(), "table".into()))
        .collect::<BTreeMap<_, _>>();
    exp.insert("catalogue_meta".into(), "view".into());
    assert_eq!(rows, exp);
    drop(q);
    let columns = c
        .prepare("PRAGMA table_info(catalogue_meta)")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    assert_eq!(columns, vec!["key", "value"]);
    for (t, target, cols) in [
        (
            "registration_locations_v2",
            "registrations_v2",
            vec!["pack_id", "release_sha256"],
        ),
        (
            "installed_heads_v2",
            "generations_v2",
            vec!["generation_id"],
        ),
    ] {
        let mut q = c.prepare(&format!("PRAGMA foreign_key_list({t})")).unwrap();
        let fks = q
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        assert_eq!(fks.len(), cols.len());
        assert_eq!(
            fks.iter()
                .map(|r| r.0)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            1
        );
        for (seq, col) in cols.into_iter().enumerate() {
            assert!(
                fks.iter()
                    .any(|r| r.1 == seq as i64 && r.2 == target && r.3 == col && r.4 == col)
            );
        }
    }
    assert!(
        !c.prepare("PRAGMA foreign_key_check")
            .unwrap()
            .exists([])
            .unwrap()
    );
    for (t, col) in [
        ("catalogue_meta", "value"),
        ("registrations", "pack_id"),
        ("registration_locations", "pack_id"),
        ("generations", "generation_id"),
        ("installed_heads", "pack_id"),
        ("attempts", "attempt_id"),
    ] {
        for sql in [
            format!("INSERT INTO {t} DEFAULT VALUES"),
            format!("UPDATE {t} SET {col}={col} WHERE 0"),
            format!("DELETE FROM {t} WHERE 0"),
        ] {
            let e = c.execute_batch(&sql).unwrap_err().to_string();
            assert!(
                e.contains("no such table") || e.contains("view"),
                "not a namespace refusal: {sql}: {e}"
            );
        }
    }
    assert!(
        c.prepare("SELECT members_json FROM generations ORDER BY local_revision DESC LIMIT 1")
            .is_err()
    );
    assert!(c.prepare("SELECT payload_kind,seal_json,descriptor_json FROM registrations WHERE pack_id=?1 AND release_sha256=?2").is_err());
    drop(c);
    assert_eq!(before, db(root));
    println!(
        "PASS exact six physical tables/single view/FKs and eighteen old-name write refusals plus two no-op reads; unchanged catalogue bytes"
    );
}
// Prospective independent controls over a caller-owned v1 fixture only.
// Restoring its exact baseline bytes is test setup; no accepted/root data is touched.
fn migration_metadata_negatives(root: &Path, revision: u64) {
    let file = root.join("catalogue.sqlite3");
    let original = std::fs::read(&file).unwrap();
    for label in [
        "seal-pack",
        "seal-digest",
        "unregistered-member",
        "duplicate-member",
    ] {
        let c = rusqlite::Connection::open(&file).unwrap();
        match label {
            "seal-pack" | "seal-digest" => {
                let bytes: Vec<u8> = c
                    .query_row(
                        "SELECT seal_json FROM registrations WHERE pack_id='llg'",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap();
                let mut seal: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                if label == "seal-pack" {
                    seal["pack_id"] = serde_json::json!("sutysisku");
                } else {
                    seal["release_sha256"] = serde_json::json!("0".repeat(64));
                }
                c.execute(
                    "UPDATE registrations SET seal_json=?1 WHERE pack_id='llg'",
                    [serde_json::to_vec(&seal).unwrap()],
                )
                .unwrap();
            }
            "unregistered-member" => {
                c.execute("DELETE FROM registration_locations WHERE pack_id='llg'", [])
                    .unwrap();
                c.execute("DELETE FROM registrations WHERE pack_id='llg'", [])
                    .unwrap();
                assert!(
                    !c.prepare("PRAGMA foreign_key_check")
                        .unwrap()
                        .exists([])
                        .unwrap()
                );
            }
            "duplicate-member" => {
                let bytes: Vec<u8> = c
                    .query_row(
                        "SELECT members_json FROM generations ORDER BY local_revision DESC LIMIT 1",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap();
                let mut members: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                let members = members.as_array_mut().unwrap();
                let llg = members
                    .iter()
                    .find(|m| m["pack_id"] == "llg")
                    .unwrap()
                    .clone();
                members.push(llg);
                c.execute("UPDATE generations SET members_json=?1 WHERE generation_id=(SELECT generation_id FROM generations ORDER BY local_revision DESC LIMIT 1)",[serde_json::to_vec(members).unwrap()]).unwrap();
            }
            _ => unreachable!(),
        }
        drop(c);
        let corrupted = std::fs::read(&file).unwrap();
        assert_ne!(corrupted, original);
        let error = CollectionCatalogue::migrate(root, revision).unwrap_err();
        assert_eq!(
            error.report("migrate").category,
            "unsupported_version",
            "{label}: {error:?}"
        );
        assert_eq!(
            std::fs::read(&file).unwrap(),
            corrupted,
            "migration changed rejected {label}"
        );
        std::fs::write(&file, &original).unwrap();
        let c = CollectionCatalogue::open(root).unwrap();
        assert_eq!(c.schema_version().unwrap(), 1);
        assert_eq!(c.local_revision().unwrap(), revision);
        assert!(c.generation().unwrap().is_some());
        drop(c);
        assert_eq!(std::fs::read(&file).unwrap(), original);
        println!(
            "PASS independent migration metadata negative {label}; unchanged rejected bytes and exact restored valid counterpart"
        );
    }
}
fn io_signature(
    error: &(dyn std::error::Error + 'static),
) -> Option<(std::io::ErrorKind, Option<i32>)> {
    let mut at = Some(error);
    while let Some(e) = at {
        if let Some(io) = e.downcast_ref::<std::io::Error>() {
            return Some((io.kind(), io.raw_os_error()));
        }
        at = e.source();
    }
    None
}
#[cfg(unix)]
fn native_controls(base: &Path, root: &Path, seals: &Seals) -> PathBuf {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let byte_root = base.join(std::ffi::OsString::from_vec(b"native-root-\xff".to_vec()));
    let chosen;
    match std::fs::create_dir(&byte_root) {
        Ok(()) => {
            std::fs::remove_dir(&byte_root).unwrap();
            chosen = byte_root.clone();
            println!(
                "METHOD host permits invalid-UTF8 root; captured-v1 migration/update/rebuild fixture will use that exact native path"
            );
        }
        Err(direct) => {
            let metadata = std::fs::symlink_metadata(&byte_root).unwrap_err();
            println!(
                "METHOD invalid-byte ROOT: direct mkdir={direct:?}; equivalent direct symlink_metadata={metadata:?}"
            );
            let want = Some((metadata.kind(), metadata.raw_os_error()));
            for (op, error) in [
                (
                    "migrate",
                    CollectionCatalogue::migrate(&byte_root, 0).unwrap_err(),
                ),
                (
                    "update",
                    CollectionCatalogue::update(&byte_root, req(&seals[3..4]), 0).unwrap_err(),
                ),
                (
                    "rebuild",
                    CollectionCatalogue::rebuild(&byte_root, 0).unwrap_err(),
                ),
            ] {
                let report = error.report(op);
                if metadata.kind() == std::io::ErrorKind::NotFound {
                    assert_eq!(report.category, "missing_catalogue", "{op}: {error:?}");
                } else {
                    assert_eq!(io_signature(&error), want, "{op}: {error:?}");
                }
                let path = report.path.unwrap();
                assert_eq!(
                    path.hex,
                    byte_root
                        .as_os_str()
                        .as_bytes()
                        .iter()
                        .map(|x| format!("{x:02x}"))
                        .collect::<String>()
                );
                assert!(path.utf8.is_none());
            }
            assert!(!PathBuf::from(byte_root.to_string_lossy().into_owned()).exists());
            chosen = base.join("old v1 λ ;$ catalogue");
            println!(
                "PASS platform-qualified invalid-native ROOT errors: exact native report and equivalent metadata outcome; no invalid-byte physical-success claim on this host"
            );
        }
    }
    let input = base.join(std::ffi::OsString::from_vec(
        b"native-release-\xfe.json".to_vec(),
    ));
    let seal = base.join("native-input-seal.json");
    std::fs::write(&seal, &seals[3].2).unwrap();
    let (_, revision, _) = state(root);
    let before = db(root);
    let write = std::fs::write(&input, &seals[3].1);
    let direct = std::fs::read(&input);
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedFiles {
            pack_id: PackId::Sutysisku,
            release: input.clone(),
            seal: seal.clone(),
        }],
        CollectionSelection::Packs(vec![PackId::Sutysisku]),
        true,
    )
    .unwrap();
    let actual = CollectionCatalogue::update(root, request, revision);
    match (write, direct, actual) {
        (Ok(()), Ok(bytes), Ok(report)) => {
            assert_eq!(bytes, seals[3].1);
            assert_eq!(report.outcome, "idempotent");
            println!(
                "PASS actual existing invalid-UTF8 sealed input through update; unchanged catalogue"
            );
        }
        (Err(_), Err(direct), Err(error)) => {
            assert_eq!(
                io_signature(&error),
                Some((direct.kind(), direct.raw_os_error()))
            );
            let reported = error.report("update").path.unwrap();
            assert_eq!(
                reported.hex,
                input
                    .as_os_str()
                    .as_bytes()
                    .iter()
                    .map(|x| format!("{x:02x}"))
                    .collect::<String>()
            );
            assert!(reported.utf8.is_none());
            println!(
                "PASS platform-qualified native sealed-input read: exact bytes/report/direct IO cause; no physical-success claim"
            );
        }
        other => panic!("native input outcome differs from direct filesystem control: {other:?}"),
    }
    assert_eq!(db(root), before);
    assert!(!PathBuf::from(input.to_string_lossy().into_owned()).exists());
    let valid = CollectionRequest::new(
        vec![CollectionInput::SealedFiles {
            pack_id: PackId::Sutysisku,
            release: base.join("inputs/sutysisku/release.json"),
            seal: base.join("inputs/sutysisku/seal.json"),
        }],
        CollectionSelection::Packs(vec![PackId::Sutysisku]),
        true,
    )
    .unwrap();
    assert_eq!(
        CollectionCatalogue::update(root, valid, revision)
            .unwrap()
            .outcome,
        "idempotent"
    );
    assert_eq!(db(root), before);
    println!("PASS actual valid sealed-files update counterpart; no byte change");
    chosen
}
fn deleted_cwd_child(root: &Path, cwd: &Path) {
    assert!(root.is_absolute() && cwd.is_absolute());
    let (_, revision, g) = state(root);
    let old = g.unwrap();
    let old_bytes = std::fs::read(old.store_path.join("dictionary.sqlite3")).unwrap();
    let before = db(root);
    std::fs::create_dir(cwd).unwrap();
    std::env::set_current_dir(cwd).unwrap();
    std::fs::remove_dir(cwd).unwrap();
    assert!(std::env::current_dir().is_err());
    let migration = CollectionCatalogue::migrate(root, revision).unwrap();
    assert_eq!(migration.outcome, "idempotent");
    assert_eq!(before, db(root));
    let seals = sealed(&models());
    let update = CollectionCatalogue::update(root, req(&seals[3..4]), revision).unwrap();
    assert_eq!(update.outcome, "idempotent");
    assert_eq!(before, db(root));
    let rebuild = CollectionCatalogue::rebuild(root, revision).unwrap();
    assert_eq!(rebuild.local_revision, revision + 1);
    let rebuilt = rebuild.generation.unwrap();
    assert_ne!(rebuilt.generation_id, old.generation_id);
    assert_eq!(rebuilt.content_id, old.content_id);
    assert_eq!(rebuilt.canonical_sha256, old.canonical_sha256);
    assert_eq!(
        std::fs::read(old.store_path.join("dictionary.sqlite3")).unwrap(),
        old_bytes
    );
    println!(
        "{}",
        serde_json::json!({"ok":true,"scope":"actual all-absolute migrate/update/rebuild with deleted cwd and no HOME"})
    );
}
fn main() {
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("__deleted-cwd")) {
        let root = PathBuf::from(std::env::args_os().nth(2).unwrap());
        let cwd = PathBuf::from(std::env::args_os().nth(3).unwrap());
        deleted_cwd_child(&root, &cwd);
        return;
    }
    let base = PathBuf::from(std::env::args_os().nth(1).expect("exclusive new scratch"));
    let old = PathBuf::from(std::env::args_os().nth(2).expect("preserved old v1 CLI"));
    std::fs::create_dir(&base).unwrap();
    let ds = models();
    assert_eq!(
        ds.iter()
            .map(|d| d.metadata().assertions.len())
            .collect::<Vec<_>>(),
        vec![1, 1, 1, 3]
    );
    assert_eq!(
        ds.iter()
            .map(|d| d
                .metadata()
                .assertions
                .iter()
                .filter(|a| a.eligibility == Eligibility::Lexical)
                .count())
            .collect::<Vec<_>>(),
        vec![1, 1, 1, 2]
    );
    let exp = expected(&ds);
    assert_eq!(exp.len(), 25);
    assert!(exp["frames"].is_empty());
    let mut wrong = exp.clone();
    wrong.remove("frames");
    assert_ne!(exp, wrong);
    let mut wrong = exp.clone();
    wrong.insert("unknown_empty", BTreeMap::new());
    assert_ne!(exp, wrong);
    let mut wrong = exp.clone();
    *wrong
        .get_mut("assertions")
        .unwrap()
        .values_mut()
        .next()
        .unwrap() += 1;
    assert_ne!(exp, wrong);
    let mut changed = draft(ds[0].clone());
    changed.metadata.entries[0].spelling = Presence::Missing;
    assert_ne!(inventory(&draft(ds[0].clone())), inventory(&changed));
    let mut reordered = draft(ds[3].clone());
    let original_order = reordered
        .metadata
        .assertions
        .iter()
        .map(|a| a.id.clone())
        .collect::<Vec<_>>();
    reordered.metadata.assertions.reverse();
    assert_ne!(
        original_order,
        reordered
            .metadata
            .assertions
            .iter()
            .map(|a| a.id.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(inventory(&draft(ds[3].clone())), inventory(&reordered));
    let seals = sealed(&ds);
    let input = base.join("inputs");
    std::fs::create_dir(&input).unwrap();
    let mut manifest = vec![];
    for (id, r, s) in &seals {
        let dir = input.join(id.as_str());
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("release.json"), r).unwrap();
        std::fs::write(dir.join("seal.json"), s).unwrap();
        manifest.push(serde_json::json!({"kind":"sealed","pack_id":id.as_str(),"release":format!("inputs/{}/release.json",id.as_str()),"seal":format!("inputs/{}/seal.json",id.as_str())}));
    }
    let mf = base.join("manifest.json");
    std::fs::write(
        &mf,
        serde_json::to_vec(&serde_json::json!({"version":1,"inputs":manifest})).unwrap(),
    )
    .unwrap();
    std::fs::write(
        base.join("pre-install-expected.json"),
        serde_json::to_vec_pretty(&exp).unwrap(),
    )
    .unwrap();
    println!(
        "PASS source-derived 25-family inventory with reorder positive and four distinct comparator negatives BEFORE installs"
    );
    let fresh = base.join("fresh-v2");
    CollectionCatalogue::install(&fresh, req(&seals)).unwrap();
    let (version, _, generation) = state(&fresh);
    assert_eq!(version, 2);
    assert_eq!(store_inventory(&generation.unwrap()), exp);
    let c = CollectionCatalogue::open(&fresh).unwrap();
    let listed = serde_json::to_value(c.list().unwrap()).unwrap();
    let keys = listed
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        keys,
        std::collections::BTreeSet::from([
            "version",
            "catalogue_schema_version",
            "local_revision",
            "generation",
            "packs",
            "staged_attempts"
        ])
    );
    assert_eq!(listed["version"], 1);
    assert_eq!(listed["catalogue_schema_version"], 2);
    drop(c);
    schema_audit(&fresh);
    println!(
        "PASS fresh catalogue2 full inventory and exact intentionally additive list field shape"
    );
    let child = Command::new(std::env::current_exe().unwrap())
        .arg("__deleted-cwd")
        .arg(&fresh)
        .arg(base.join("deleted-cwd-child"))
        .env_clear()
        .output()
        .unwrap();
    std::fs::write(base.join("deleted-cwd.stdout"), &child.stdout).unwrap();
    std::fs::write(base.join("deleted-cwd.stderr"), &child.stderr).unwrap();
    assert!(
        child.status.success(),
        "deleted-cwd child: {}",
        String::from_utf8_lossy(&child.stderr)
    );
    let child_result: serde_json::Value = serde_json::from_slice(&child.stdout).unwrap();
    assert_eq!(child_result["ok"], true);
    println!(
        "PASS actual isolated child: all-absolute migrate/update/rebuild succeed with deleted cwd and no HOME"
    );
    let v1 = native_controls(&base, &fresh, &seals);
    old_root(&old, &mf, &v1, &base.join("old-install.json"), false);
    let (version, revision, g) = state(&v1);
    assert_eq!(version, 1);
    let original = g.unwrap();
    assert_eq!(store_inventory(&original), exp);
    let original_store_sha =
        sha(&std::fs::read(original.store_path.join("dictionary.sqlite3")).unwrap());
    let before = db(&v1);
    let err = CollectionCatalogue::install(&v1, req(&seals)).unwrap_err();
    assert_eq!(err.report("install").category, "migration_required");
    assert_eq!(before, db(&v1));
    println!(
        "PASS captured old binary produces v1 readable by new API; ordinary v1 write rejects without byte changes"
    );
    migration_metadata_negatives(&v1, revision);
    // This is a modeled old SQL connection, not an actual paused old CLI process.
    let preopened = rusqlite::Connection::open(v1.join("catalogue.sqlite3")).unwrap();
    let version_before: String = preopened
        .query_row(
            "SELECT value FROM catalogue_meta WHERE key='schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(version_before, "1");
    let m = CollectionCatalogue::migrate(&v1, revision).unwrap();
    assert_eq!(m.schema_version, 2);
    assert_eq!(m.local_revision, revision + 1);
    let (_, mut revision, g) = state(&v1);
    assert_eq!(g.as_ref().unwrap().generation_id, original.generation_id);
    assert_eq!(
        sha(&std::fs::read(original.store_path.join("dictionary.sqlite3")).unwrap()),
        original_store_sha
    );
    assert!(
        preopened
            .prepare("SELECT members_json FROM generations ORDER BY local_revision DESC LIMIT 1")
            .is_err()
    );
    assert!(preopened.prepare("SELECT payload_kind,seal_json,descriptor_json FROM registrations WHERE pack_id=?1 AND release_sha256=?2").is_err());
    drop(preopened);
    schema_audit(&v1);
    let out = Command::new(&old)
        .args(["collections", "list"])
        .arg(&v1)
        .env_clear()
        .output()
        .unwrap();
    assert!(!out.status.success());
    let report: serde_json::Value = serde_json::from_slice(&out.stderr).unwrap();
    assert_eq!(report["error"]["category"], "unsupported_version");
    std::fs::write(base.join("old-after-v2.stderr"), out.stderr).unwrap();
    println!(
        "PASS migrated v1 keeps exact generation/store; captured old binary rejects v2; modeled preopened old no-op reads reject"
    );
    let before = db(&v1);
    let idem = CollectionCatalogue::migrate(&v1, revision).unwrap();
    assert_eq!(idem.outcome, "idempotent");
    assert_eq!(db(&v1), before);
    let (mut eng, cookie) = {
        let mut e = engine(&v1);
        let response = e.execute(page(Some(1), None)).unwrap();
        let CollectionQueryResult::Exact(x) = response.result else {
            panic!()
        };
        assert_eq!(x.totals.occurrences, 5);
        let cookie = x.next_cursor.unwrap();
        (e, cookie)
    };
    let fresh = eng.execute(page(Some(1), Some(cookie.clone()))).unwrap();
    assert!(matches!(fresh.result, CollectionQueryResult::Exact(_)));
    drop(eng);
    let same = CollectionCatalogue::update(&v1, req(&seals[3..4]), revision).unwrap();
    assert_eq!(same.outcome, "idempotent");
    assert_eq!(same.local_revision, revision);
    assert_eq!(
        same.generation.unwrap().generation_id,
        original.generation_id
    );
    assert_eq!(db(&v1), before);
    engine(&v1)
        .execute(page(Some(1), Some(cookie.clone())))
        .unwrap();
    println!(
        "PASS same-seal verified update and repeat migration retain revision/bytes/generation and fresh-engine cursor"
    );
    let sut = String::from_utf8(SUT.to_vec())
        .unwrap()
        .replace("synthetic two", "independently revised two");
    let d = import_sutysisku_export(
        sut.as_bytes(),
        &SutysiskuImportOptions {
            revision: "1".repeat(40),
            expected_raw_sha256: sha(sut.as_bytes()),
            raw_locator: Some("https://example.invalid/sut".into()),
            notice_locator: Some("https://example.invalid/code-notice".into()),
        },
    )
    .unwrap()
    .into_dictionary();
    let old_target = target(&ds[3]);
    let new_target = target(&d);
    let mut after = ds.clone();
    after[3] = d.clone();
    let after_expected = expected(&after);
    let seal = seal_release(PackId::Sutysisku, PackPayloadKind::Synthetic, &d).unwrap();
    let new_one = vec![(PackId::Sutysisku, seal.release, seal.seal)];
    std::fs::write(
        base.join("pre-update-expected.json"),
        serde_json::to_vec_pretty(&after_expected).unwrap(),
    )
    .unwrap();
    let before = db(&v1);
    let e = CollectionCatalogue::update(&v1, req(&new_one), revision - 1).unwrap_err();
    assert_eq!(e.report("update").category, "revision_conflict");
    assert_eq!(before, db(&v1));
    let updated = CollectionCatalogue::update(&v1, req(&new_one), revision).unwrap();
    revision = updated.local_revision;
    let updated = updated.generation.unwrap();
    assert_ne!(updated.generation_id, original.generation_id);
    assert_eq!(store_inventory(&updated), after_expected);
    assert_eq!(
        sha(&std::fs::read(original.store_path.join("dictionary.sqlite3")).unwrap()),
        original_store_sha
    );
    assert!(matches!(
        engine(&v1).execute(page(Some(1), Some(cookie))),
        Err(CollectionQueryError::StaleCursor)
    ));
    let mut e = engine(&v1);
    for (t, want) in [
        (old_target, ReferenceStatus::CatalogueOnly),
        (new_target, ReferenceStatus::Resolved),
    ] {
        let r = e
            .execute(DictionaryRequest::Resolve { target: t.clone() })
            .unwrap();
        let CollectionQueryResult::Resolve(x) = r.result else {
            panic!()
        };
        assert_eq!(x.target, t);
        assert_eq!(x.status, want);
    }
    drop(e);
    println!(
        "PASS stale revision rejects, strict new release updates full independent inventory, retains old bytes/ref identity, and invalidates prior cursor"
    );
    let current_sha = sha(&std::fs::read(updated.store_path.join("dictionary.sqlite3")).unwrap());
    let rebuilt = CollectionCatalogue::rebuild(&v1, revision).unwrap();
    assert_eq!(rebuilt.outcome, "rebuilt");
    assert_eq!(rebuilt.local_revision, revision + 1);
    let rebuilt = rebuilt.generation.unwrap();
    assert_ne!(rebuilt.generation_id, updated.generation_id);
    assert_eq!(rebuilt.content_id, updated.content_id);
    assert_eq!(rebuilt.canonical_sha256, updated.canonical_sha256);
    assert_eq!(store_inventory(&rebuilt), after_expected);
    assert_eq!(
        sha(&std::fs::read(updated.store_path.join("dictionary.sqlite3")).unwrap()),
        current_sha
    );
    println!(
        "PASS forced rebuild creates new physical generation with exact logical body/content and immutable prior bytes"
    );
    let registered = base.join("registered-v1");
    old_root(
        &old,
        &mf,
        &registered,
        &base.join("old-register.json"),
        true,
    );
    let c = CollectionCatalogue::open(&registered).unwrap();
    let rev = c.local_revision().unwrap();
    let before = db(&registered);
    let error = CollectionCatalogue::migrate(&registered, rev).unwrap_err();
    assert_eq!(error.report("migrate").category, "busy");
    assert_eq!(db(&registered), before);
    assert_eq!(c.schema_version().unwrap(), 1);
    drop(c);
    let m = CollectionCatalogue::migrate(&registered, rev).unwrap();
    assert_eq!(m.schema_version, 2);
    assert!(state(&registered).2.is_none());
    println!(
        "PASS actual pinned-reader migration Busy/rollback/prior1 and released-reader registered-only successful counterpart"
    );
    let (_, revision, g) = state(&registered);
    assert!(g.is_none());
    let before = db(&registered);
    let e = CollectionCatalogue::rebuild(&registered, revision).unwrap_err();
    assert_eq!(e.report("rebuild").category, "not_installed");
    assert_eq!(db(&registered), before);
    println!("PASS registered-only rebuild fails explicitly without activation/byte change");
    println!(
        "{}",
        serde_json::json!({"ok":true,"scope":"independent S04 initial tiny fixture/version/update/rebuild/ref controls; not Full or complete S04","base":base})
    );
}
