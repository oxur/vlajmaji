use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe { enabled: bool, revision: u64, label: String, packs: Vec<String> }
fn parse(s: &str) -> Result<Probe, confyg::ConfigError> {
    confyg::Confygery::new()?.add_str(s)?.build()
}
fn main() {
    let cases = [
        ("enabled = false\nrevision = 0\nlabel = ''\npacks = []\n", Probe { enabled:false,revision:0,label:String::new(),packs:vec![] }),
        ("enabled = true\nrevision = 1\nlabel = 'all'\npacks = ['jb','llg']\n", Probe { enabled:true,revision:1,label:"all".into(),packs:vec!["jb".into(),"llg".into()] }),
        ("enabled = false\nrevision = 2\nlabel = 'one'\npacks = ['llg']\n", Probe { enabled:false,revision:2,label:"one".into(),packs:vec!["llg".into()] }),
    ];
    for (s, expected) in cases { let actual=parse(s).unwrap(); assert_eq!(actual,expected); let wire=toml::to_string_pretty(&actual).unwrap(); assert_eq!(parse(&wire).unwrap(),expected); }
    for s in ["enabled=false\nrevision=0\nlabel=''\npacks=[]\nextra=true", "enabled=false\nenabled=true\nrevision=0\nlabel=''\npacks=[]", "enabled=false\nrevision=0\npacks=[]"] { assert!(parse(s).is_err()); }
    println!("PASS: three literal single-source cases, three serializer roundtrips, unknown/duplicate/missing-field negatives; no env sources/file search or stores.");
}
