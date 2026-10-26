// Independent source, prepared before candidate execution. This file is not a
// passing receipt; the final runner freezes its bytes before its first run.
use std::path::Path;
use vlajmaji_lexicon::PackId;
use vlajmaji_store::{CollectionProfileWriteRequest, CollectionProfileMembershipWriteRequest, CollectionProfiles};

pub fn run(base: &Path, root: &Path, packs: &[PackId]) {
    let file = base.join("profiles.toml");
    let request = || CollectionProfileWriteRequest {
        name: "local".into(), root: root.to_owned(), packs: packs.to_vec(),
        disabled: Vec::new(), make_default: true,
    };
    let written = CollectionProfiles::configure(&file, request()).unwrap();
    assert_eq!(written.revision, 1);
    assert_eq!(written.outcome, "committed");
    let captured = CollectionProfiles::read(&file).unwrap();
    let selected = captured.profile(None).unwrap();
    assert_eq!(selected.name, "local");
    assert_eq!(selected.root, root);
    assert_eq!(selected.packs, packs);
    assert!(selected.disabled.is_empty());
    captured.verify_unchanged().unwrap();
    println!("PASS initial explicit profile roundtrip and default selection");

    let first = std::fs::read(&file).unwrap();
    let mut with_comment = first.clone();
    with_comment.extend_from_slice(b"\n# retained independent comment\n");
    std::fs::write(&file, &with_comment).unwrap();
    assert!(captured.verify_unchanged().is_err());
    let repeated = CollectionProfiles::configure(&file, request()).unwrap();
    assert_eq!(repeated.revision, 1);
    assert_eq!(repeated.outcome, "unchanged");
    assert_eq!(std::fs::read(&file).unwrap(), with_comment);
    println!("PASS semantic no-op preserves changed formatting and exact bytes");

    // The request has an absolute root and explicit disabled=[], while this
    // hand-authored file uses a relative root and omits disabled entirely.
    // A wire-DTO comparison would incorrectly rewrite this equivalent policy.
    let root_relative = root.strip_prefix(base).unwrap().to_str().unwrap();
    let pack_json = serde_json::to_string(&packs.iter().map(|pack| pack.as_str()).collect::<Vec<_>>()).unwrap();
    let equivalent = format!("version = 1\nrevision = 7\ndefault_profile = \"local\"\n\n[profiles.local]\nroot = {}\npacks = {}\n# no disabled field\n", serde_json::to_string(root_relative).unwrap(), pack_json);
    std::fs::write(&file, equivalent.as_bytes()).unwrap();
    let repeated = CollectionProfiles::configure(&file, request()).unwrap();
    assert_eq!(repeated.revision, 7);
    assert_eq!(repeated.outcome, "unchanged");
    assert_eq!(std::fs::read(&file).unwrap(), equivalent.as_bytes());
    println!("PASS resolved-relative and omitted-default semantic no-op");

    let valid = equivalent.clone();
    let variants = [
        ("unknown field", valid.replacen("version = 1", "unknown = true\nversion = 1", 1)),
        ("duplicate field", valid.replacen("version = 1", "version = 1\nversion = 1", 1)),
        ("unsupported version", valid.replacen("version = 1", "version = 2", 1)),
        ("unknown default", valid.replacen("default_profile = \"local\"", "default_profile = \"absent\"", 1)),
        ("negative revision", valid.replacen("revision = 7", "revision = -1", 1)),
        ("invalid profile name", valid.replacen("profiles.local", "profiles.\"bad name\"", 1)),
        ("duplicate pack", valid.replacen(&format!("packs = {pack_json}"), &format!("packs = [{0:?}, {0:?}]", packs[0].as_str()), 1)),
    ];
    for (label, bytes) in variants {
        std::fs::write(&file, bytes.as_bytes()).unwrap();
        assert!(CollectionProfiles::read(&file).is_err(), "{label}");
        assert_eq!(std::fs::read(&file).unwrap(), bytes.as_bytes(), "reader mutated {label}");
        std::fs::write(&file, valid.as_bytes()).unwrap();
        assert_eq!(CollectionProfiles::read(&file).unwrap().profile(None).unwrap().packs, packs, "valid counterpart {label}");
        println!("PASS strict config {label} rejection and exact valid counterpart");
    }
    #[cfg(unix)] {
        use std::os::unix::{ffi::{OsStrExt,OsStringExt}};
        let encoded:String=root.as_os_str().as_bytes().iter().map(|byte|format!("{byte:02x}")).collect();
        let native=|encoding:&str,hex:&str|format!("version=1\nrevision=7\ndefault_profile=\"local\"\n[profiles.local]\nroot_native={{encoding={encoding:?},hex={hex:?}}}\npacks={pack_json}\n");
        let valid_native=native("unix-bytes",&encoded);
        std::fs::write(&file,&valid_native).unwrap();
        assert_eq!(CollectionProfiles::read(&file).unwrap().profile(None).unwrap().root,root);
        let invalids=[("encoding",native("unsupported",&encoded)),("empty",native("unix-bytes","")),
            ("odd",native("unix-bytes","2")),("nonhex",native("unix-bytes","zz")),
            ("uppercase",native("unix-bytes",&encoded.to_uppercase())),
            ("relative",native("unix-bytes","72656c6174697665")),("NUL",native("unix-bytes","2f006f")),
            ("both roots",format!("{valid_native}root=\"/tmp\"\n"))];
        assert_ne!(encoded,encoded.to_uppercase());
        for (label,bytes) in invalids {
            std::fs::write(&file,&bytes).unwrap();assert!(CollectionProfiles::read(&file).is_err(),"native {label}");
            assert_eq!(std::fs::read(&file).unwrap(),bytes.as_bytes());
            std::fs::write(&file,&valid_native).unwrap();
            assert_eq!(CollectionProfiles::read(&file).unwrap().profile(None).unwrap().root,root);
        }
        let mut unsupported_bytes=base.as_os_str().as_bytes().to_vec();unsupported_bytes.extend_from_slice(b"/unselected-native-\xff");
        let unsupported=std::path::PathBuf::from(std::ffi::OsString::from_vec(unsupported_bytes.clone()));
        let unsupported_hex:String=unsupported_bytes.iter().map(|byte|format!("{byte:02x}")).collect();
        let with_unused=format!("{valid_native}\n[profiles.unused]\nroot_native={{encoding=\"unix-bytes\",hex={unsupported_hex:?}}}\npacks=[]\n");
        std::fs::write(&file,with_unused).unwrap();
        let before=std::fs::read_dir(base).unwrap().map(|entry|entry.unwrap().file_name()).collect::<std::collections::BTreeSet<_>>();
        let snapshot=CollectionProfiles::read(&file).unwrap();
        assert_eq!(snapshot.profile(Some("unused")).unwrap().root,unsupported);
        let report=serde_json::to_value(snapshot.report()).unwrap();
        let summary=report["profiles"].as_array().unwrap().iter().find(|row|row["name"]=="unused").unwrap();
        assert_eq!(summary["root"]["encoding"],"unix-bytes");assert_eq!(summary["root"]["hex"],unsupported_hex);
        let after=std::fs::read_dir(base).unwrap().map(|entry|entry.unwrap().file_name()).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(before,after);
        println!("PASS native ROOT strict wire cases, exact byte roundtrip and unchanged directory entries; unselected-root read boundary also needs source inspection");
        std::fs::write(&file,valid.as_bytes()).unwrap();
    }

    let no_default = valid.replace("default_profile = \"local\"\n", "");
    std::fs::write(&file, no_default.as_bytes()).unwrap();
    let no_default = CollectionProfiles::read(&file).unwrap();
    assert!(no_default.profile(None).is_err());
    assert_eq!(no_default.profile(Some("local")).unwrap().root, root);
    println!("PASS explicit profile works with no implicit default");

    // Parsing an unselected profile is not permission to inspect its ROOT.
    let untouched = base.join("unselected-root-must-remain-absent");
    let with_unused = format!("{valid}\n[profiles.unused]\nroot = \"unselected-root-must-remain-absent\"\npacks = []\ndisabled = []\n");
    std::fs::write(&file, with_unused.as_bytes()).unwrap();
    let read = CollectionProfiles::read(&file).unwrap();
    assert_eq!(read.profile(Some("unused")).unwrap().root, untouched);
    assert!(read.profile(Some("unused")).unwrap().packs.is_empty());
    assert!(!untouched.exists());
    let _ = read.report();
    assert!(!untouched.exists());
    println!("PASS unselected root receives no creation and empty membership stays empty");

    std::fs::write(&file, valid.as_bytes()).unwrap();
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        // Existing destination mode must survive a deliberate changed policy.
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o640)).unwrap();
        let mut changed = request();
        changed.disabled = vec![packs[0]];
        let report = CollectionProfiles::configure(&file, changed).unwrap();
        assert_eq!(report.revision, 8);
        assert_eq!(std::fs::metadata(&file).unwrap().permissions().mode() & 0o777, 0o640);
        assert_eq!(CollectionProfiles::read(&file).unwrap().profile(None).unwrap().disabled, vec![packs[0]]);
        println!("PASS changed policy increments revision and preserves existing mode");
    }
    std::fs::write(&file, valid.as_bytes()).unwrap();
    // This exact suffix is reviewed against the frozen implementation before
    // execution; the sentinel is ours, and the candidate must not remove it.
    let lock = base.join("profiles.toml.lock");
    let sentinel = b"independent occupied lock; preserve exact bytes";
    std::fs::OpenOptions::new().write(true).create_new(true).open(&lock).unwrap();
    std::fs::write(&lock, sentinel).unwrap();
    assert!(CollectionProfiles::configure(&file, request()).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), valid.as_bytes());
    assert_eq!(std::fs::read(&lock).unwrap(), sentinel);
    std::fs::remove_file(&lock).unwrap();
    assert_eq!(CollectionProfiles::configure(&file, request()).unwrap().outcome, "unchanged");
    println!("PASS occupied lock preserves sentinel and exact same request succeeds after release");

    let overflow = valid.replace("revision = 7", "revision = 9223372036854775807");
    std::fs::write(&file, overflow.as_bytes()).unwrap();
    let mut changed = request(); changed.disabled = vec![packs[0]];
    assert!(CollectionProfiles::configure(&file, changed).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), overflow.as_bytes());
    assert!(!lock.exists());
    std::fs::write(&file, valid.as_bytes()).unwrap();
    let mut changed = request(); changed.disabled = vec![packs[0]];
    assert_eq!(CollectionProfiles::configure(&file, changed).unwrap().revision, 8);
    println!("PASS revision overflow preserves bytes and ordinary changed revision succeeds");

    #[cfg(unix)] {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let target = base.join("symlink-target.toml");
        std::fs::write(&target, valid.as_bytes()).unwrap();
        let linked = base.join("symlink-config.toml");
        symlink(&target, &linked).unwrap();
        assert!(CollectionProfiles::configure(&linked, request()).is_err());
        assert!(std::fs::symlink_metadata(&linked).unwrap().file_type().is_symlink());
        assert_eq!(std::fs::read(&target).unwrap(), valid.as_bytes());
        std::fs::remove_file(&linked).unwrap();
        assert_eq!(CollectionProfiles::configure(&linked, request()).unwrap().revision, 1);
        println!("PASS config-leaf symlink rejected without target mutation and absent-leaf counterpart succeeds");

        let alias = base.join("benign-parent-alias");
        symlink(base, &alias).unwrap();
        let through_alias = alias.join("new-private/inner/config.toml");
        CollectionProfiles::configure(&through_alias, request()).unwrap();
        assert_eq!(std::fs::metadata(base.join("new-private")).unwrap().permissions().mode() & 0o777, 0o700);
        assert_eq!(std::fs::metadata(base.join("new-private/inner")).unwrap().permissions().mode() & 0o777, 0o700);
        assert_eq!(std::fs::metadata(&through_alias).unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(CollectionProfiles::read(&through_alias).unwrap().profile(None).unwrap().root, root);
        println!("PASS benign ancestor alias and private newly-created parent/file modes");
    }
    std::fs::write(&file,valid.as_bytes()).unwrap();
    let mut explicit=request();explicit.disabled=vec![PackId::LensiskuJbotci];
    CollectionProfiles::configure(&file,explicit).unwrap();
    let membership=||CollectionProfileMembershipWriteRequest{name:"local".into(),root:root.to_owned(),packs:vec![PackId::LaLojban],make_default:true};
    CollectionProfiles::configure_membership(&file,membership()).unwrap();
    let selected=CollectionProfiles::read(&file).unwrap().profile(None).unwrap();
    assert_eq!(selected.packs,vec![PackId::LaLojban]);assert_eq!(selected.disabled,vec![PackId::LensiskuJbotci]);
    let preserved=std::fs::read(&file).unwrap();let revision=CollectionProfiles::read(&file).unwrap().revision();
    assert_eq!(CollectionProfiles::configure_membership(&file,membership()).unwrap().outcome,"unchanged");
    assert_eq!(std::fs::read(&file).unwrap(),preserved);assert_eq!(CollectionProfiles::read(&file).unwrap().revision(),revision);
    let mut clear=request();clear.packs=vec![PackId::LaLojban];clear.disabled=Vec::new();
    CollectionProfiles::configure(&file,clear).unwrap();
    assert!(CollectionProfiles::read(&file).unwrap().profile(None).unwrap().disabled.is_empty());
    println!("PASS membership-only preserves independent disable policy/no-op; explicit writer deliberately clears it");

    CollectionProfiles::configure_membership(&file,CollectionProfileMembershipWriteRequest{name:"other".into(),root:root.to_owned(),packs:vec![PackId::Llg],make_default:false}).unwrap();
    let snapshot=CollectionProfiles::read(&file).unwrap();assert!(snapshot.profile(Some("other")).unwrap().disabled.is_empty());
    assert_eq!(snapshot.profile(Some("local")).unwrap().packs,vec![PackId::LaLojban]);
    println!("PASS new membership profile starts enabled and preserves unrelated target semantics");

    fn register(root:&Path,pack:PackId,dictionary:&vlajmaji_lexicon::Dictionary) {
        let sealed=vlajmaji_lexicon::seal_release(pack,vlajmaji_lexicon::PackPayloadKind::Synthetic,dictionary).unwrap();
        let input=vlajmaji_store::CollectionInput::SealedBytes{pack_id:pack,release:std::borrow::Cow::Borrowed(&sealed.release),seal:std::borrow::Cow::Borrowed(&sealed.seal)};
        let request=vlajmaji_store::CollectionRequest::new(vec![input],vlajmaji_store::CollectionSelection::Packs(vec![pack]),true).unwrap();
        vlajmaji_store::CollectionCatalogue::register(root,request).unwrap();
    }
    let models=crate::models();let narrow=base.join("membership-narrow-root");
    register(&narrow,PackId::Llg,&models[1]);
    let mut explicit=request();explicit.disabled=vec![PackId::LensiskuJbotci];CollectionProfiles::configure(&file,explicit).unwrap();
    let old=std::fs::read(&file).unwrap();
    let moved=||CollectionProfileMembershipWriteRequest{name:"local".into(),root:narrow.clone(),packs:vec![PackId::Llg],make_default:true};
    assert!(CollectionProfiles::configure_membership(&file,moved()).is_err());assert_eq!(std::fs::read(&file).unwrap(),old);
    register(&narrow,PackId::LensiskuJbotci,&models[0]);
    CollectionProfiles::configure_membership(&file,moved()).unwrap();
    let moved=CollectionProfiles::read(&file).unwrap().profile(None).unwrap();
    assert_eq!(moved.root,narrow);assert_eq!(moved.disabled,vec![PackId::LensiskuJbotci]);
    println!("PASS incompatible root preserves exact old file; identical request succeeds after required registration exists");
    std::fs::write(&file, valid.as_bytes()).unwrap();
}
