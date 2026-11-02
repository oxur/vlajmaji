//! Explicit remaining S07 calls after preserved session50292; never repeats setup.
use serde::Serialize;
use serde_json::{Value, json};
use std::{env, error::Error, fs, path::Path};
use vlajmaji::{lexicon::PackId, search::*, store::*};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn save(out: &Path, name: &str, value: &impl Serialize) -> Result<()> {
    fs::write(out.join(name), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn exact() -> DictionaryRequest {
    DictionaryRequest::Exact {
        spelling: "farvi".into(),
        options: Default::default(),
    }
}
fn check_generated(
    response: &ConfiguredWorkbenchLexicalFullResponse,
    observed: &Value,
) -> Result<()> {
    let WorkbenchLexicalFullResult::Generate(body) = &response.result else {
        panic!("Generate result")
    };
    assert_eq!(serde_json::to_value(&body.value)?, *observed);
    assert!(body.code_notices.is_empty());
    Ok(())
}
fn main() -> Result<()> {
    let out = std::path::PathBuf::from(
        env::args_os()
            .nth(1)
            .ok_or("usage: remaining-consumer SAME_OUTPUT_ROOT")?,
    );
    assert!(out.is_absolute());
    let config = out.join("profiles.json");
    assert!(!config.exists() && !out.join("view-r2").exists());
    assert!(out.join("catalogue").is_dir() && out.join("imported-owned-journal").is_dir());
    // This is a retained, already input-checked actual result, used for parity only.
    let baseline: WorkbenchFullResponse =
        serde_json::from_slice(&fs::read(out.join("rust-exact-r2.json"))?)?;
    let WorkbenchFullResult::Exact(body) = &baseline.result else {
        panic!("retained r2 Exact")
    };
    assert_eq!((body.totals.occurrences, body.totals.source_groups), (8, 4));
    assert_eq!(
        baseline.selection.owned.as_ref().unwrap().revision,
        "owned-r2"
    );
    // Seven observed VALUE fields from actual session50292 stderr (SHA10fb8389).
    // This is not a fabricated direct Generate envelope or code-notices observation.
    let observed_direct_value = json!({"components":["mlatu","kerlo"],"setting":"force_y","spelling":"latkerlo","engine_score":7937,"component_indices":[[0,3],[3,8]],"score_function":"latkerlo_jvotci::get_lujvo_with_analytics","engine_id":"latkerlo-jvotci-2.10.2601/ForceY"});
    let request = WorkbenchLexicalRequest {
        version: 1,
        query: WorkbenchLexicalQuery::Generate {
            components: vec!["mlatu".into(), "kerlo".into()],
            setting: MorphologySetting::ForceY,
        },
        options: Default::default(),
    };
    let configure = WorkbenchProfiles::configure(
        &config,
        WorkbenchProfileConfigureRequest {
            name: "four-pack".into(),
            catalogue_root: Some(out.join("catalogue")),
            owned_root: Some(out.join("imported-owned-journal")),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![
                    PackId::LensiskuJbotci,
                    PackId::Llg,
                    PackId::LaLojban,
                    PackId::Sutysisku,
                ],
                disabled: vec![],
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: vec![],
            },
            make_default: true,
        },
        None,
    )?;
    save(&out, "profile-configure.json", &configure)?;
    let refresh = WorkbenchProfiles::refresh(&config, Some("four-pack"), out.join("view-r2"), 1)?;
    save(&out, "profile-refresh.json", &refresh)?;
    println!(
        "Remaining-stage profile configure+refresh completed; no setup/store/direct-view replay"
    );
    let mut named = ConfiguredWorkbenchSearchEngine::open(&config, Some("four-pack"))?;
    let named_exact = named.execute(exact())?;
    assert_eq!(named_exact.result, baseline.result);
    let named_gen = named.execute_lexical(&request)?;
    check_generated(&named_gen, &observed_direct_value)?;
    save(&out, "rust-configured-exact.json", &named_exact)?;
    save(&out, "rust-configured-generate.json", &named_gen)?;
    drop(named);
    let mut default = ConfiguredWorkbenchSearchEngine::open(&config, None)?;
    let default_exact = default.execute(exact())?;
    assert_eq!(default_exact.result, baseline.result);
    assert_eq!(default_exact.result, named_exact.result);
    let default_gen = default.execute_lexical(&request)?;
    check_generated(&default_gen, &observed_direct_value)?;
    assert_eq!(default_gen.result, named_gen.result);
    save(&out, "rust-configured-default-exact.json", &default_exact)?;
    save(&out, "rust-configured-default-generate.json", &default_gen)?;
    save(
        &out,
        "profile-report.json",
        &WorkbenchProfiles::read(&config)?.report(),
    )?;
    // Only the three originally approved native CLI callers consume these requests.
    save(&out, "exact-farvi.json", &exact())?;
    save(&out, "generate-force-y.json", &request)?;
    println!(
        "PASS remaining-stage named/default Exact8 and Generate values; original3 nativeCLI still pending"
    );
    Ok(())
}
