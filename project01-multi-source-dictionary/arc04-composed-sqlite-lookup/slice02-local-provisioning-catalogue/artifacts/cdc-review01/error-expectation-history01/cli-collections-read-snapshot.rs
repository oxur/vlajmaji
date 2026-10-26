//! Public collection commands exercised through the actual CLI process.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;
use vlajmaji_lexicon::{Dictionary, PackId, PackPayloadKind, seal_release};

fn fresh_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("vlajmaji-s02-cli-{label}-{}-{stamp}", std::process::id()));
    std::fs::create_dir(&path).expect("create fresh CLI test root");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    path
}

fn run(args: &[&Path]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vlajmaji"));
    command.args(args).env_clear().stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    command.output().expect("run collection CLI")
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("CLI JSON stdout")
}

#[test]
fn test_collections_cli_register_install_list_inspect_expect_structured_reports() {
    let root = fresh_root("lifecycle");
    let files = root.join("sealed inputs");
    std::fs::create_dir(&files).unwrap();
    let data = Dictionary::from_document(Default::default()).unwrap();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &data).unwrap();
    let release = files.join("release.json");
    let seal = files.join("seal.json");
    std::fs::write(&release, &sealed.release).unwrap();
    std::fs::write(&seal, &sealed.seal).unwrap();
    let catalogue = root.join("catalogue with spaces");

    let register = run(&[
        Path::new("collections"), Path::new("register"), &catalogue,
        Path::new("--pack"), Path::new("llg"), Path::new("--release"), &release,
        Path::new("--seal"), &seal, Path::new("--allow-synthetic"),
    ]);
    assert!(register.status.success(), "{}", String::from_utf8_lossy(&register.stderr));
    let registered = json(&register);
    assert_eq!(registered["operation"], "register");
    assert_eq!(registered["outcome"], "registered");
    assert_eq!(registered["local_revision"], 1);
    assert!(registered["generation"].is_null());
    assert!(!catalogue.join("generations").read_dir().unwrap().next().is_some());

    let install = run(&[
        Path::new("collections"), Path::new("install"), &catalogue,
        Path::new("--pack"), Path::new("llg"), Path::new("--release"), &release,
        Path::new("--seal"), &seal, Path::new("--allow-synthetic"),
    ]);
    assert!(install.status.success(), "{}", String::from_utf8_lossy(&install.stderr));
    let installed = json(&install);
    assert_eq!(installed["operation"], "install");
    assert_eq!(installed["outcome"], "installed");
    assert_eq!(installed["local_revision"], 2);
    let store = PathBuf::from(installed["generation"]["path"]["utf8"].as_str().unwrap());
    assert!(store.join("dictionary.sqlite3").is_file());

    let list = run(&[Path::new("collections"), Path::new("list"), &catalogue]);
    assert!(list.status.success(), "{}", String::from_utf8_lossy(&list.stderr));
    let listed = json(&list);
    assert_eq!(listed["local_revision"], 2);
    assert_eq!(listed["packs"][0]["pack_id"], "llg");
    assert_eq!(listed["generation"]["generation_id"], installed["generation"]["generation_id"]);

    let inspect = run(&[Path::new("collections"), Path::new("inspect"), &catalogue, Path::new("llg")]);
    assert!(inspect.status.success(), "{}", String::from_utf8_lossy(&inspect.stderr));
    let inspected = json(&inspect);
    assert_eq!(inspected["pack_id"], "llg");
    assert_eq!(inspected["last_verification_scope"], "all");
    assert_eq!(inspected["policy_state"], "unconfigured");

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn test_collections_cli_list_absent_catalogue_expect_typed_error() {
    let parent = fresh_root("absent");
    let absent = parent.join("no catalogue");
    let output = run(&[Path::new("collections"), Path::new("list"), &absent]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).expect("structured stderr");
    assert_eq!(error["error"]["category"], "missing_catalogue");
    assert_eq!(error["error"]["operation"], "list");
    assert_eq!(error["error"]["activation"], "not_applicable");
    assert!(error["error"]["observed_generation"].is_null());
    assert!(!absent.exists());
    let _ = std::fs::remove_dir_all(parent);
}
