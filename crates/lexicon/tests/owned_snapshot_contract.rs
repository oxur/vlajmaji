//! Fixed complete-history codec and sealing contract tests.

use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    Dictionary, EntityId, OwnedCurationError, OwnedHead, OwnedRevisionRecord, OwnedSectionPrefix,
    OwnedSnapshot, OwnedSnapshotCommit, OwnedSnapshotData, OwnedSnapshotError, OwnedSnapshotSeal,
    Presence, SourceId, SourceOrigin, TextValue, TimePrecision, load_owned_snapshot,
    seal_owned_snapshot,
};

#[path = "../examples/support/dictionary_fixture.rs"]
mod fixture;

const SCHEMA1_SNAPSHOT: &str = r#####"{"version":1,"journal_schema":1,"source":"owned-example","head":"owned-r1","dictionary":{"version":1,"legacy_archive":null,"registry":{"version":1,"sources":[{"id":"owned-example","display_name":"Personal dictionary","abbreviation":"OWN","abbreviation_kind":"ProjectAssigned","abbreviation_evidence":"Example-assigned selector","origin":"OwnedDraft"}],"contributors":[{"id":"editor-example","display_name":"Dictionary editor"}],"distributors":[],"aliases":[],"lineage":[],"snapshots":[{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"source":"owned-example","distributor":null}],"occurrences":[{"occurrence":{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/a"},"contributor":"editor-example","native_entry_id":"farvi","native_assertion_id":"example-a"},{"occurrence":{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/b"},"contributor":"editor-example","native_entry_id":"farvi","native_assertion_id":"example-b"}]},"captures":[{"identity":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"metadata":{"source_name":"Example input","format_label":"application/json","locator":"seed/edit example","revision":"Missing","captured_at":"Missing","time_precision":"Unknown","notice_locator":"Missing"},"bytes":[123,34,119,111,114,100,34,58,34,102,97,114,118,105,34,44,34,100,101,102,105,110,105,116,105,111,110,115,34,58,91,34,103,114,111,119,34,44,34,100,101,118,101,108,111,112,34,93,125],"occurrences":[{"locator":"/a","fields":[{"name":"definition","encoding":"Text","raw_value":"grow"}],"raw_line":"farvi — grow"},{"locator":"/b","fields":[{"name":"definition","encoding":"Text","raw_value":"develop"}],"raw_line":"farvi — develop"}]}],"metadata":{"catalogue_revision":1,"code_notices":[],"collections":[{"id":"owned-collection","source":"owned-example","display_name":"My dictionary","description":{"Value":{"text":"An explicitly owned personal dictionary","language":"Missing","markup":"Missing"}}}],"components":[{"id":"owned-component","collection":"owned-collection","native_name":"owned lexical entries","languages":["jbo"]}],"releases":[{"id":"owned-release-r0","component":"owned-component","distributor":null,"upstream_revision":"Missing","distributor_revision":"Missing","captured_at":"Missing","time_precision":"Unknown","schema_version":1,"normalizer":"owned-example","normalizer_version":"1","captures":[],"assertions":[],"accounting":{"input_occurrences":0,"normalized_occurrences":0,"query_excluded_occurrences":0,"unit":"occurrences"},"diagnostics":[],"selected":true,"availability":"Available","content_notices":[{"text":{"Value":"example data only; no licence grant"},"locator":"Missing","license":"Missing"}],"redistribution":"Held","redistribution_evidence":[]},{"id":"owned-release-r1","component":"owned-component","distributor":null,"upstream_revision":"Missing","distributor_revision":"Missing","captured_at":"Missing","time_precision":"Unknown","schema_version":1,"normalizer":"owned-example","normalizer_version":"1","captures":[{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"}],"assertions":["owned-assertion-a","owned-assertion-b"],"accounting":{"input_occurrences":2,"normalized_occurrences":2,"query_excluded_occurrences":0,"unit":"occurrences"},"diagnostics":[],"selected":true,"availability":"Available","content_notices":[{"text":{"Value":"example data only; no licence grant"},"locator":"Missing","license":"Missing"}],"redistribution":"Held","redistribution_evidence":[]}],"entries":[{"id":"owned-entry-farvi","source":"owned-example","spelling":{"Value":"farvi"},"language":{"Value":"jbo"},"script":"Missing","native_word_class":"Missing","selmaho":"Null","mapped_pos":"Missing"}],"assertions":[{"id":"owned-assertion-a","entry":"owned-entry-farvi","occurrence":{"capture":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/a"},"native_sense_id":"Missing","language":{"Value":"en"},"definition":{"Value":{"text":"grow","language":"Missing","markup":"Missing"}},"source_score":"Null","timestamps":[],"eligibility":"Lexical"},{"id":"owned-assertion-b","entry":"owned-entry-farvi","occurrence":{"capture":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/b"},"native_sense_id":"Missing","language":{"Value":"en"},"definition":{"Value":{"text":"develop","language":"Missing","markup":"Missing"}},"source_score":"Null","timestamps":[],"eligibility":"Lexical"}],"contexts":[],"frames":[],"forms":[],"etymologies":[],"relations":[],"owned_revisions":[{"id":"owned-r0","source":"owned-example","parent":null,"editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":[],"tombstones":[],"rationale":{"Value":{"text":"Start with an empty active set","language":"Missing","markup":"Missing"}}},{"id":"owned-r1","source":"owned-example","parent":"owned-r0","editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":["owned-assertion-a","owned-assertion-b"],"tombstones":[],"rationale":{"Value":{"text":"Add two source-distinct definitions","language":"Missing","markup":"Missing"}}}],"owned_heads":[{"source":"owned-example","revision":"owned-r1"}],"editorial_senses":[],"computations":[],"annotations":[],"extensions":[]}},"commits":[{"revision":"owned-r0","state_sha256":"178cb4a48245db47a568bb740f5ec123575aae38eb961c9ce05f40c7f0526c9a","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":0},{"section":"registry.occurrences","length":0},{"section":"captures","length":0},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":1},{"section":"metadata.entries","length":0},{"section":"metadata.assertions","length":0},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":1},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]},{"revision":"owned-r1","state_sha256":"e96d7fdee27a1cf84b152799b51c7b6e541aeae714396665fa98985f5c81d05f","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":1},{"section":"registry.occurrences","length":2},{"section":"captures","length":1},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":2},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":2},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":2},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]}],"curation":null}"#####;

const SCHEMA1_SEAL: &str = r#####"{"version":1,"kind":"vlajmaji-owned-snapshot","snapshot_version":1,"journal_schema":1,"dictionary_version":1,"registry_version":1,"source":"owned-example","head":"owned-r1","revision_ids":["owned-r0","owned-r1"],"snapshot_sha256":"a16d6f1c3112bb62fa3dd091d6a972e17c0f084dc433f4f3110584faef090a05","snapshot_bytes":8040}"#####;

const SCHEMA2_SNAPSHOT: &str = r#####"{"version":1,"journal_schema":2,"source":"owned-example","head":"owned-r4","dictionary":{"version":1,"legacy_archive":null,"registry":{"version":1,"sources":[{"id":"owned-example","display_name":"Personal dictionary","abbreviation":"OWN","abbreviation_kind":"ProjectAssigned","abbreviation_evidence":"Example-assigned selector","origin":"OwnedDraft"}],"contributors":[{"id":"editor-example","display_name":"Dictionary editor"}],"distributors":[],"aliases":[],"lineage":[],"snapshots":[{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"source":"owned-example","distributor":null}],"occurrences":[{"occurrence":{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/a"},"contributor":"editor-example","native_entry_id":"farvi","native_assertion_id":"example-a"},{"occurrence":{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/b"},"contributor":"editor-example","native_entry_id":"farvi","native_assertion_id":"example-b"}]},"captures":[{"identity":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"metadata":{"source_name":"Example input","format_label":"application/json","locator":"seed/edit example","revision":"Missing","captured_at":"Missing","time_precision":"Unknown","notice_locator":"Missing"},"bytes":[123,34,119,111,114,100,34,58,34,102,97,114,118,105,34,44,34,100,101,102,105,110,105,116,105,111,110,115,34,58,91,34,103,114,111,119,34,44,34,100,101,118,101,108,111,112,34,93,125],"occurrences":[{"locator":"/a","fields":[{"name":"definition","encoding":"Text","raw_value":"grow"}],"raw_line":"farvi — grow"},{"locator":"/b","fields":[{"name":"definition","encoding":"Text","raw_value":"develop"}],"raw_line":"farvi — develop"}]}],"metadata":{"catalogue_revision":4,"code_notices":[],"collections":[{"id":"owned-collection","source":"owned-example","display_name":"My dictionary","description":{"Value":{"text":"An explicitly owned personal dictionary","language":"Missing","markup":"Missing"}}}],"components":[{"id":"owned-component","collection":"owned-collection","native_name":"owned lexical entries","languages":["jbo"]}],"releases":[{"id":"owned-release-r0","component":"owned-component","distributor":null,"upstream_revision":"Missing","distributor_revision":"Missing","captured_at":"Missing","time_precision":"Unknown","schema_version":1,"normalizer":"owned-example","normalizer_version":"1","captures":[],"assertions":[],"accounting":{"input_occurrences":0,"normalized_occurrences":0,"query_excluded_occurrences":0,"unit":"occurrences"},"diagnostics":[],"selected":true,"availability":"Available","content_notices":[{"text":{"Value":"example data only; no licence grant"},"locator":"Missing","license":"Missing"}],"redistribution":"Held","redistribution_evidence":[]},{"id":"owned-release-r1","component":"owned-component","distributor":null,"upstream_revision":"Missing","distributor_revision":"Missing","captured_at":"Missing","time_precision":"Unknown","schema_version":1,"normalizer":"owned-example","normalizer_version":"1","captures":[{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"}],"assertions":["owned-assertion-a","owned-assertion-b"],"accounting":{"input_occurrences":2,"normalized_occurrences":2,"query_excluded_occurrences":0,"unit":"occurrences"},"diagnostics":[],"selected":true,"availability":"Available","content_notices":[{"text":{"Value":"example data only; no licence grant"},"locator":"Missing","license":"Missing"}],"redistribution":"Held","redistribution_evidence":[]}],"entries":[{"id":"owned-entry-farvi","source":"owned-example","spelling":{"Value":"farvi"},"language":{"Value":"jbo"},"script":"Missing","native_word_class":"Missing","selmaho":"Null","mapped_pos":"Missing"}],"assertions":[{"id":"owned-assertion-a","entry":"owned-entry-farvi","occurrence":{"capture":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/a"},"native_sense_id":"Missing","language":{"Value":"en"},"definition":{"Value":{"text":"grow","language":"Missing","markup":"Missing"}},"source_score":"Null","timestamps":[],"eligibility":"Lexical"},{"id":"owned-assertion-b","entry":"owned-entry-farvi","occurrence":{"capture":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/b"},"native_sense_id":"Missing","language":{"Value":"en"},"definition":{"Value":{"text":"develop","language":"Missing","markup":"Missing"}},"source_score":"Null","timestamps":[],"eligibility":"Lexical"}],"contexts":[],"frames":[],"forms":[],"etymologies":[],"relations":[],"owned_revisions":[{"id":"owned-r0","source":"owned-example","parent":null,"editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":[],"tombstones":[],"rationale":{"Value":{"text":"Start with an empty active set","language":"Missing","markup":"Missing"}}},{"id":"owned-r1","source":"owned-example","parent":"owned-r0","editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":["owned-assertion-a","owned-assertion-b"],"tombstones":[],"rationale":{"Value":{"text":"Add two source-distinct definitions","language":"Missing","markup":"Missing"}}},{"id":"owned-r2","source":"owned-example","parent":"owned-r1","editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":["owned-assertion-a","owned-assertion-b"],"tombstones":[],"rationale":{"Value":{"text":"Review explicit own and external occurrences","language":"Missing","markup":"Missing"}}},{"id":"owned-r3","source":"owned-example","parent":"owned-r2","editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":["owned-assertion-a","owned-assertion-b"],"tombstones":[],"rationale":{"Value":{"text":"Explicit curation selection owned-r3","language":"Missing","markup":"Missing"}}},{"id":"owned-r4","source":"owned-example","parent":"owned-r3","editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":["owned-assertion-a","owned-assertion-b"],"tombstones":[],"rationale":{"Value":{"text":"Explicit curation selection owned-r4","language":"Missing","markup":"Missing"}}}],"owned_heads":[{"source":"owned-example","revision":"owned-r4"}],"editorial_senses":[],"computations":[],"annotations":[],"extensions":[]}},"commits":[{"revision":"owned-r0","state_sha256":"178cb4a48245db47a568bb740f5ec123575aae38eb961c9ce05f40c7f0526c9a","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":0},{"section":"registry.occurrences","length":0},{"section":"captures","length":0},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":1},{"section":"metadata.entries","length":0},{"section":"metadata.assertions","length":0},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":1},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]},{"revision":"owned-r1","state_sha256":"e96d7fdee27a1cf84b152799b51c7b6e541aeae714396665fa98985f5c81d05f","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":1},{"section":"registry.occurrences","length":2},{"section":"captures","length":1},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":2},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":2},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":2},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]},{"revision":"owned-r2","state_sha256":"1cdf04f2a5dfc078ebc7c55416821a2a62a5835865c21846a78d289b142cef3a","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":1},{"section":"registry.occurrences","length":2},{"section":"captures","length":1},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":2},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":2},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":3},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]},{"revision":"owned-r3","state_sha256":"d5279857727404503d065e4760960e2665d0e3b617cd5d88249eba53fb4e3917","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":1},{"section":"registry.occurrences","length":2},{"section":"captures","length":1},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":2},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":2},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":4},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]},{"revision":"owned-r4","state_sha256":"5d912d6f36fe0df64045a862ed4dd228fde1caf2d2371979e9e86f758b0ab1a5","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":1},{"section":"registry.occurrences","length":2},{"section":"captures","length":1},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":2},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":2},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":5},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]}],"curation":{"version":1,"contexts":[{"id":"cdc-context","model_sha256":"db8483868e595b42c412632c0b5b90e101d5a25a4f74bc2f5d1546de94ce0f13","source_ids":["lensisku"],"release_ids":["la-lojban-release-e9aeeda43d145fac9dd2430a88c958be771edfae8e31b7b8e0b9a82f0b7c756b"]}],"records":[{"id":"cdc-reviewed-sense","revision":"owned-r2","editor":"editor-example","rationale":{"text":"Review two retained owned assertions and one supplied occurrence","language":"Missing","markup":"Missing"},"kind":{"sense":{"members":[{"owned_assertion":"owned-assertion-a"},{"owned_assertion":"owned-assertion-b"},{"external":{"context":"cdc-context","target":{"source":"lensisku","release":"la-lojban-release-e9aeeda43d145fac9dd2430a88c958be771edfae8e31b7b8e0b9a82f0b7c756b","native_entry_id":null,"native_assertion_id":"7","occurrence":{"capture":{"namespace":"captured:lensisku:la-lojban:en","sha256":"4667ccabc6b00fd874b95a4e12d0639fa7dfb85aa0fc1046e8108ef198332a15"},"locator":"/0"}},"entry":"la-lojban-4667ccabc6b00fd874b95a4e12d0639fa7dfb85aa0fc1046e8108ef198332a15-entry-0","assertion":"la-lojban-4667ccabc6b00fd874b95a4e12d0639fa7dfb85aa0fc1046e8108ef198332a15-assertion-0"}}]}}},{"id":"cdc-reviewed-link","revision":"owned-r2","editor":"editor-example","rationale":{"text":"Link the reviewed sense to the pinned external occurrence","language":"Missing","markup":"Missing"},"kind":{"link":{"origin":{"EditorialSense":"cdc-reviewed-sense"},"direction":"Forward","target":{"context":"cdc-context","target":{"source":"lensisku","release":"la-lojban-release-e9aeeda43d145fac9dd2430a88c958be771edfae8e31b7b8e0b9a82f0b7c756b","native_entry_id":null,"native_assertion_id":"7","occurrence":{"capture":{"namespace":"captured:lensisku:la-lojban:en","sha256":"4667ccabc6b00fd874b95a4e12d0639fa7dfb85aa0fc1046e8108ef198332a15"},"locator":"/0"}},"entry":"la-lojban-4667ccabc6b00fd874b95a4e12d0639fa7dfb85aa0fc1046e8108ef198332a15-entry-0","assertion":"la-lojban-4667ccabc6b00fd874b95a4e12d0639fa7dfb85aa0fc1046e8108ef198332a15-assertion-0"}}}}],"revisions":[{"revision":"owned-r0","active":[],"tombstones":[]},{"revision":"owned-r1","active":[],"tombstones":[]},{"revision":"owned-r2","active":["cdc-reviewed-sense","cdc-reviewed-link"],"tombstones":[]},{"revision":"owned-r3","active":["cdc-reviewed-sense"],"tombstones":["cdc-reviewed-link"]},{"revision":"owned-r4","active":["cdc-reviewed-sense","cdc-reviewed-link"],"tombstones":[]}],"prefixes":[{"revision":"owned-r0","contexts_length":0,"records_length":0,"curation_sha256":"7a9ff76782384a8d55e2ccf5505759d2c6b709ec3a56a86294f70e78d3b7a2e0"},{"revision":"owned-r1","contexts_length":0,"records_length":0,"curation_sha256":"8003d426f599021ac75733efd9cc9462e2f99b3bd421c423a256e0d71596bf2b"},{"revision":"owned-r2","contexts_length":1,"records_length":2,"curation_sha256":"b6ec1a55d9602412849c910464996899f0b015899bd68a7002031e7cc2a445a9"},{"revision":"owned-r3","contexts_length":1,"records_length":2,"curation_sha256":"36ee700b310f64f599327f22d7017eacc32705bc051aa514a55675b6f4520e18"},{"revision":"owned-r4","contexts_length":1,"records_length":2,"curation_sha256":"fdf5d1d7ec48458d4815b2230ee5086f2709e028d7fa1416111337674062ef4a"}]}}"#####;

const SCHEMA2_SEAL: &str = r#####"{"version":1,"kind":"vlajmaji-owned-snapshot","snapshot_version":1,"journal_schema":2,"dictionary_version":1,"registry_version":1,"source":"owned-example","head":"owned-r4","revision_ids":["owned-r0","owned-r1","owned-r2","owned-r3","owned-r4"],"snapshot_sha256":"2d4925e6baf72044b71cae7a78f466412276b3d837765a6f0b1ed59fe58c3d8a","snapshot_bytes":15705}"#####;

const ALTERNATE_SNAPSHOT: &str = r#####"{"version":1,"journal_schema":1,"source":"owned-example","head":"owned-r1","dictionary":{"version":1,"legacy_archive":null,"registry":{"version":1,"sources":[{"id":"owned-example","display_name":"Personal dictionary","abbreviation":"OWN","abbreviation_kind":"ProjectAssigned","abbreviation_evidence":"Example-assigned selector","origin":"OwnedDraft"}],"contributors":[{"id":"editor-example","display_name":"Dictionary editor"}],"distributors":[],"aliases":[],"lineage":[],"snapshots":[{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"source":"owned-example","distributor":null}],"occurrences":[{"occurrence":{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/a"},"contributor":"editor-example","native_entry_id":"farvi","native_assertion_id":"example-a"},{"occurrence":{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/b"},"contributor":"editor-example","native_entry_id":"farvi","native_assertion_id":"example-b"}]},"captures":[{"identity":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"metadata":{"source_name":"Example input","format_label":"application/json","locator":"seed/edit example","revision":"Missing","captured_at":"Missing","time_precision":"Unknown","notice_locator":"Missing"},"bytes":[123,34,119,111,114,100,34,58,34,102,97,114,118,105,34,44,34,100,101,102,105,110,105,116,105,111,110,115,34,58,91,34,103,114,111,119,34,44,34,100,101,118,101,108,111,112,34,93,125],"occurrences":[{"locator":"/a","fields":[{"name":"definition","encoding":"Text","raw_value":"grow"}],"raw_line":"farvi — grow"},{"locator":"/b","fields":[{"name":"definition","encoding":"Text","raw_value":"develop"}],"raw_line":"farvi — develop"}]}],"metadata":{"catalogue_revision":1,"code_notices":[],"collections":[{"id":"owned-collection","source":"owned-example","display_name":"My dictionary","description":{"Value":{"text":"An explicitly owned personal dictionary","language":"Missing","markup":"Missing"}}}],"components":[{"id":"owned-component","collection":"owned-collection","native_name":"owned lexical entries","languages":["jbo"]}],"releases":[{"id":"owned-release-r0","component":"owned-component","distributor":null,"upstream_revision":"Missing","distributor_revision":"Missing","captured_at":"Missing","time_precision":"Unknown","schema_version":1,"normalizer":"owned-example","normalizer_version":"1","captures":[],"assertions":[],"accounting":{"input_occurrences":0,"normalized_occurrences":0,"query_excluded_occurrences":0,"unit":"occurrences"},"diagnostics":[],"selected":true,"availability":"Available","content_notices":[{"text":{"Value":"example data only; no licence grant"},"locator":"Missing","license":"Missing"}],"redistribution":"Held","redistribution_evidence":[]},{"id":"owned-release-r1","component":"owned-component","distributor":null,"upstream_revision":"Missing","distributor_revision":"Missing","captured_at":"Missing","time_precision":"Unknown","schema_version":1,"normalizer":"owned-example","normalizer_version":"1","captures":[{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"}],"assertions":["owned-assertion-a","owned-assertion-b"],"accounting":{"input_occurrences":2,"normalized_occurrences":2,"query_excluded_occurrences":0,"unit":"occurrences"},"diagnostics":[],"selected":true,"availability":"Available","content_notices":[{"text":{"Value":"example data only; no licence grant"},"locator":"Missing","license":"Missing"}],"redistribution":"Held","redistribution_evidence":[]}],"entries":[{"id":"owned-entry-farvi","source":"owned-example","spelling":{"Value":"farvi"},"language":{"Value":"jbo"},"script":"Missing","native_word_class":"Missing","selmaho":"Null","mapped_pos":"Missing"}],"assertions":[{"id":"owned-assertion-a","entry":"owned-entry-farvi","occurrence":{"capture":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/a"},"native_sense_id":"Missing","language":{"Value":"en"},"definition":{"Value":{"text":"grow","language":"Missing","markup":"Missing"}},"source_score":"Null","timestamps":[],"eligibility":"Lexical"},{"id":"owned-assertion-b","entry":"owned-entry-farvi","occurrence":{"capture":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/b"},"native_sense_id":"Missing","language":{"Value":"en"},"definition":{"Value":{"text":"develop","language":"Missing","markup":"Missing"}},"source_score":"Null","timestamps":[],"eligibility":"Lexical"}],"contexts":[],"frames":[],"forms":[],"etymologies":[],"relations":[],"owned_revisions":[{"id":"owned-r0","source":"owned-example","parent":null,"editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":[],"tombstones":[],"rationale":{"Value":{"text":"Start with an empty active set","language":"Missing","markup":"Missing"}}},{"id":"owned-r1","source":"owned-example","parent":"owned-r0","editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":["owned-assertion-a","owned-assertion-b"],"tombstones":[],"rationale":{"Value":{"text":"Add two source-distinct definitions","language":"Missing","markup":"Missing"}}}],"owned_heads":[{"source":"owned-example","revision":"owned-r1"}],"editorial_senses":[],"computations":[],"annotations":[],"extensions":[]}},"commits":[{"revision":"owned-r0","state_sha256":"e674cb41a0b83b2db2165fb5d966e78e8ad460e6e821304c6e9e2ab8841a1a47","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":0},{"section":"registry.occurrences","length":0},{"section":"captures","length":0},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":1},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":0},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":1},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]},{"revision":"owned-r1","state_sha256":"e96d7fdee27a1cf84b152799b51c7b6e541aeae714396665fa98985f5c81d05f","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":1},{"section":"registry.occurrences","length":2},{"section":"captures","length":1},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":2},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":2},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":2},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]}],"curation":null}"#####;

const ALTERNATE_SEAL: &str = r#####"{"version":1,"kind":"vlajmaji-owned-snapshot","snapshot_version":1,"journal_schema":1,"dictionary_version":1,"registry_version":1,"source":"owned-example","head":"owned-r1","revision_ids":["owned-r0","owned-r1"],"snapshot_sha256":"a85fd0340209ebd49161aded45cceaa06f4773e9de7358cf14c1d3e707f236f7","snapshot_bytes":8040}"#####;

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn refresh_curation_hashes(data: &mut OwnedSnapshotData, valid: &OwnedSnapshot) {
    #[derive(serde::Serialize)]
    struct CurationHash<'a> {
        version: u32,
        source: &'a SourceId,
        revision: &'a EntityId,
        base_state_sha256: String,
        contexts: &'a [vlajmaji_lexicon::OwnedContextPin],
        records: &'a [vlajmaji_lexicon::OwnedCurationRecord],
        revisions: &'a [vlajmaji_lexicon::OwnedCurationRevision],
    }

    let snapshot = data;
    let curation = snapshot.curation.as_mut().unwrap();
    for index in 0..curation.prefixes.len() {
        let contexts_length = usize::try_from(curation.prefixes[index].contexts_length).unwrap();
        let records_length = usize::try_from(curation.prefixes[index].records_length).unwrap();
        let revision = &snapshot.commits[index].revision;
        let base = valid.dictionary_at(revision).unwrap();
        let base_state_sha256 = sha256(&base.to_json_bytes().unwrap());
        let bytes = serde_json::to_vec(&CurationHash {
            version: curation.version,
            source: &snapshot.source,
            revision,
            base_state_sha256,
            contexts: &curation.contexts[..contexts_length],
            records: &curation.records[..records_length],
            revisions: &curation.revisions[..=index],
        })
        .unwrap();
        curation.prefixes[index].curation_sha256 = sha256(&bytes);
    }
}

fn check_history(payload: &str, seal_bytes: &str, schema: u32) -> OwnedSnapshot {
    let bytes = payload.as_bytes();
    let seal = seal_bytes.as_bytes();
    let snapshot = OwnedSnapshot::from_json_bytes(bytes).unwrap();
    assert_eq!(snapshot.to_json_bytes().unwrap(), bytes);
    let sealed = seal_owned_snapshot(&snapshot).unwrap();
    assert_eq!(sealed.snapshot, bytes);
    assert_eq!(sealed.seal, seal);
    assert_eq!(load_owned_snapshot(bytes, seal).unwrap(), snapshot);

    let data = snapshot.as_data();
    assert_eq!(data.journal_schema, schema);
    assert_eq!(data.commits.len(), if schema == 1 { 2 } else { 5 });
    assert!(
        data.commits
            .iter()
            .all(|commit| commit.prefixes.len() == 24)
    );
    for (index, commit) in data.commits.iter().enumerate() {
        let state = snapshot.dictionary_at(&commit.revision).unwrap();
        assert_eq!(sha256(&state.to_json_bytes().unwrap()), commit.state_sha256);
        match (
            &data.curation,
            snapshot.curation_at(&commit.revision).unwrap(),
        ) {
            (None, None) => {}
            (Some(curation), Some(state)) => {
                let prefix = &curation.prefixes[index];
                assert_eq!(state.state_sha256().unwrap(), prefix.curation_sha256);
                assert_eq!(
                    state.as_data().contexts.len() as u64,
                    prefix.contexts_length
                );
                assert_eq!(state.as_data().records.len() as u64, prefix.records_length);
                assert_eq!(state.as_data().revisions.len(), index + 1);
            }
            other => panic!("curation presence differs: {other:?}"),
        }
    }
    snapshot
}

#[test]
fn schema_one_and_two_roundtrip_all_historical_states_and_seals() {
    let schema1 = check_history(SCHEMA1_SNAPSHOT, SCHEMA1_SEAL, 1);
    let schema2 = check_history(SCHEMA2_SNAPSHOT, SCHEMA2_SEAL, 2);

    let schema2_lengths: Vec<Vec<u64>> = schema2
        .as_data()
        .commits
        .iter()
        .map(|commit| commit.prefixes.iter().map(|prefix| prefix.length).collect())
        .collect();
    assert_eq!(
        schema2_lengths,
        vec![
            vec![
                1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0
            ],
            vec![
                1, 1, 0, 0, 0, 1, 2, 1, 0, 1, 1, 2, 1, 2, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0
            ],
            vec![
                1, 1, 0, 0, 0, 1, 2, 1, 0, 1, 1, 2, 1, 2, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0
            ],
            vec![
                1, 1, 0, 0, 0, 1, 2, 1, 0, 1, 1, 2, 1, 2, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0
            ],
            vec![
                1, 1, 0, 0, 0, 1, 2, 1, 0, 1, 1, 2, 1, 2, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0
            ],
        ]
    );
    let cutoffs: Vec<(u64, u64)> = schema2
        .as_data()
        .curation
        .as_ref()
        .unwrap()
        .prefixes
        .iter()
        .map(|prefix| (prefix.contexts_length, prefix.records_length))
        .collect();
    assert_eq!(cutoffs, [(0, 0), (0, 0), (1, 2), (1, 2), (1, 2)]);
    assert!(
        schema1
            .curation_at(&schema1.as_data().head)
            .unwrap()
            .is_none()
    );
}

#[test]
fn same_final_head_does_not_make_a_different_old_cutoff_an_identical_history() {
    let original = check_history(SCHEMA1_SNAPSHOT, SCHEMA1_SEAL, 1);
    let alternate = check_history(ALTERNATE_SNAPSHOT, ALTERNATE_SEAL, 1);
    let original_data = original.as_data();
    let alternate_data = alternate.as_data();
    assert_eq!(original_data.dictionary, alternate_data.dictionary);
    assert_eq!(original_data.head, alternate_data.head);
    assert_eq!(original_data.commits.last(), alternate_data.commits.last());
    assert_ne!(
        original_data.commits[0].prefixes,
        alternate_data.commits[0].prefixes
    );
    let entries = |snapshot: &OwnedSnapshot| snapshot.as_data().commits[0].prefixes[12].length;
    assert_eq!(entries(&original), 0);
    assert_eq!(entries(&alternate), 1);
}

#[test]
fn seal_byte_methods_reject_invalid_header_versions_and_digest_syntax() {
    let valid: OwnedSnapshotSeal = serde_json::from_str(SCHEMA1_SEAL).unwrap();
    assert_eq!(valid.to_json_bytes().unwrap(), SCHEMA1_SEAL.as_bytes());
    for invalid in [
        SCHEMA1_SEAL.replace("\"version\":1", "\"version\":2"),
        SCHEMA1_SEAL.replace("vlajmaji-owned-snapshot", "other-kind"),
        SCHEMA1_SEAL.replace("a16d6f1c", "A16d6f1c"),
    ] {
        assert!(OwnedSnapshotSeal::from_json_bytes(invalid.as_bytes()).is_err());
    }
    let mut invalid = valid;
    invalid.snapshot_sha256 = "A".repeat(64);
    assert!(invalid.to_json_bytes().is_err());
}

#[test]
fn strict_snapshot_parser_rejects_duplicate_and_unknown_envelope_fields() {
    let duplicate = SCHEMA1_SNAPSHOT.replacen("\"version\":1", "\"version\":1,\"version\":1", 1);
    assert!(OwnedSnapshot::from_json_bytes(duplicate.as_bytes()).is_err());
    let unknown = SCHEMA1_SNAPSHOT.replacen("{\"version\":1", "{\"extra\":true,\"version\":1", 1);
    assert!(OwnedSnapshot::from_json_bytes(unknown.as_bytes()).is_err());
    let nested_duplicate = SCHEMA1_SNAPSHOT.replacen(
        "\"dictionary\":{\"version\":1",
        "\"dictionary\":{\"version\":1,\"version\":1",
        1,
    );
    assert!(OwnedSnapshot::from_json_bytes(nested_duplicate.as_bytes()).is_err());
    let unsupported = SCHEMA1_SNAPSHOT.replacen("\"version\":1", "\"version\":2", 1);
    assert!(matches!(
        OwnedSnapshot::from_json_bytes(unsupported.as_bytes()),
        Err(OwnedSnapshotError::UnsupportedVersion { found: 2, .. })
    ));
}

#[test]
fn bare_pretty_json_is_valid_but_sealed_input_must_be_canonical_and_all_routes_validate() {
    let value: serde_json::Value = serde_json::from_str(SCHEMA1_SNAPSHOT).unwrap();
    let pretty = serde_json::to_vec_pretty(&value).unwrap();
    let parsed = OwnedSnapshot::from_json_bytes(&pretty).unwrap();
    assert_eq!(parsed.to_json_bytes().unwrap(), SCHEMA1_SNAPSHOT.as_bytes());

    let mut pretty_seal: OwnedSnapshotSeal = serde_json::from_str(SCHEMA1_SEAL).unwrap();
    pretty_seal.snapshot_sha256 = sha256(&pretty);
    pretty_seal.snapshot_bytes = u64::try_from(pretty.len()).unwrap();
    let pretty_seal = pretty_seal.to_json_bytes().unwrap();
    assert!(matches!(
        load_owned_snapshot(&pretty, &pretty_seal),
        Err(OwnedSnapshotError::NonCanonical)
    ));

    let mut invalid_data = parsed.as_data().clone();
    invalid_data.commits[0].state_sha256 = "0".repeat(64);
    assert!(OwnedSnapshot::from_data(invalid_data.clone()).is_err());
    let invalid_data_json = serde_json::to_vec(&invalid_data).unwrap();
    assert!(serde_json::from_slice::<OwnedSnapshot>(&invalid_data_json).is_err());

    let length_mismatch = [SCHEMA1_SNAPSHOT.as_bytes(), b" "].concat();
    assert!(matches!(
        load_owned_snapshot(&length_mismatch, SCHEMA1_SEAL.as_bytes()),
        Err(OwnedSnapshotError::LengthMismatch { .. })
    ));
    let mut changed_byte = SCHEMA1_SNAPSHOT.as_bytes().to_vec();
    let last = changed_byte.len() - 1;
    changed_byte[last] = b' ';
    assert!(matches!(
        load_owned_snapshot(&changed_byte, SCHEMA1_SEAL.as_bytes()),
        Err(OwnedSnapshotError::HashMismatch { .. })
    ));

    let mut wrong_inventory: OwnedSnapshotSeal = serde_json::from_str(SCHEMA1_SEAL).unwrap();
    let alternate_head = EntityId::new("owned-r9").unwrap();
    wrong_inventory.head = alternate_head.clone();
    *wrong_inventory.revision_ids.last_mut().unwrap() = alternate_head;
    assert!(matches!(
        load_owned_snapshot(
            SCHEMA1_SNAPSHOT.as_bytes(),
            &wrong_inventory.to_json_bytes().unwrap()
        ),
        Err(OwnedSnapshotError::InventoryMismatch { field: "head" })
    ));
}

#[test]
fn from_data_rejects_invalid_history_inventory_and_prefixes_without_panicking() {
    let schema1 = check_history(SCHEMA1_SNAPSHOT, SCHEMA1_SEAL, 1);
    assert_eq!(
        OwnedSnapshot::from_data(schema1.as_data().clone()).unwrap(),
        schema1
    );
    let mut invalid = schema1.as_data().clone();
    invalid.commits[0].prefixes.pop();
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let mut invalid = schema1.as_data().clone();
    invalid.commits[0].prefixes[1].section = invalid.commits[0].prefixes[0].section.clone();
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let mut invalid = schema1.as_data().clone();
    invalid.commits[0].prefixes[0].section = "unknown.section".into();
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let mut invalid = schema1.as_data().clone();
    invalid.commits[0].prefixes.swap(0, 1);
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let mut invalid = schema1.as_data().clone();
    invalid.commits[0].prefixes[0].length = u64::MAX;
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let mut invalid = schema1.as_data().clone();
    invalid.commits[1].prefixes[0].length = 0;
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let mut invalid = schema1.as_data().clone();
    invalid.commits[1].prefixes[12].length = 0;
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let mut invalid = schema1.as_data().clone();
    invalid.commits[0].revision = EntityId::new("owned-r9").unwrap();
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let mut invalid = schema1.as_data().clone();
    invalid.source = SourceId::new("other-owned-source").unwrap();
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let mut invalid = schema1.as_data().clone();
    invalid.head = EntityId::new("owned-r0").unwrap();
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let schema2 = check_history(SCHEMA2_SNAPSHOT, SCHEMA2_SEAL, 2);
    let mut invalid = schema2.as_data().clone();
    invalid.curation = None;
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let mut invalid = schema2.as_data().clone();
    invalid.curation.as_mut().unwrap().prefixes[2].records_length = 1;
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let mut invalid = schema2.as_data().clone();
    invalid.curation.as_mut().unwrap().prefixes[0].contexts_length = u64::MAX;
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let mut invalid = schema2.as_data().clone();
    invalid.curation.as_mut().unwrap().revisions[2].revision = EntityId::new("owned-r9").unwrap();
    assert!(OwnedSnapshot::from_data(invalid).is_err());

    let mut wrong_creation_revision = schema2.as_data().clone();
    wrong_creation_revision.curation.as_mut().unwrap().records[0].revision =
        EntityId::new("owned-r1").unwrap();
    refresh_curation_hashes(&mut wrong_creation_revision, &schema2);
    let sealed_wrong_creation_revision = seal_unvalidated_data(&wrong_creation_revision);
    assert!(matches!(
        OwnedSnapshot::from_data(wrong_creation_revision.clone()),
        Err(OwnedSnapshotError::Curation(
            OwnedCurationError::InvalidDocument(message)
        )) if message == "new definitions must be active at creation"
    ));
    assert!(matches!(
        load_owned_snapshot(
            &sealed_wrong_creation_revision.0,
            &sealed_wrong_creation_revision.1
        ),
        Err(OwnedSnapshotError::Curation(
            OwnedCurationError::InvalidDocument(message)
        )) if message == "new definitions must be active at creation"
    ));

    let mut invalid = schema2.as_data().clone();
    let curation = invalid.curation.as_mut().unwrap();
    let link = curation.records[1].id.clone();
    curation.revisions[2].active = vec![link.clone()];
    refresh_curation_hashes(&mut invalid, &schema2);
    let sealed_invalid = seal_unvalidated_data(&invalid);
    assert!(matches!(
        OwnedSnapshot::from_data(invalid.clone()),
        Err(OwnedSnapshotError::Curation(
            OwnedCurationError::InvalidDocument(message)
        )) if message == "new definitions must be active at creation"
    ));
    assert!(matches!(
        load_owned_snapshot(&sealed_invalid.0, &sealed_invalid.1),
        Err(OwnedSnapshotError::Curation(
            OwnedCurationError::InvalidDocument(message)
        )) if message == "new definitions must be active at creation"
    ));

    let mut invalid = schema2.as_data().clone();
    let curation = invalid.curation.as_mut().unwrap();
    let link = curation.records[1].id.clone();
    curation.revisions[2].tombstones = vec![link];
    refresh_curation_hashes(&mut invalid, &schema2);
    let sealed_invalid = seal_unvalidated_data(&invalid);
    assert!(matches!(
        OwnedSnapshot::from_data(invalid.clone()),
        Err(OwnedSnapshotError::Curation(
            OwnedCurationError::InvalidDocument(message)
        )) if message == "curation tombstones differ from prior active removals"
    ));
    assert!(matches!(
        load_owned_snapshot(&sealed_invalid.0, &sealed_invalid.1),
        Err(OwnedSnapshotError::Curation(
            OwnedCurationError::InvalidDocument(message)
        )) if message == "curation tombstones differ from prior active removals"
    ));

    let mut missing_new_active = schema1.as_data().clone();
    let mut document = missing_new_active.dictionary.document();
    document.metadata.owned_revisions[1].assertions.pop();
    let dictionary = Dictionary::from_document(document).unwrap();
    missing_new_active.commits[1].state_sha256 = sha256(&dictionary.to_json_bytes().unwrap());
    missing_new_active.dictionary = dictionary;
    let sealed_invalid = seal_unvalidated_data(&missing_new_active);
    assert!(matches!(
        OwnedSnapshot::from_data(missing_new_active.clone()),
        Err(OwnedSnapshotError::History(message))
            if message == "new assertions must be active at their creation revision"
    ));
    assert!(matches!(
        load_owned_snapshot(&sealed_invalid.0, &sealed_invalid.1),
        Err(OwnedSnapshotError::History(message))
            if message == "new assertions must be active at their creation revision"
    ));

    let valid_removal = schema1_with_r2_removing_b(&schema1);
    let validated_removal = OwnedSnapshot::from_data(valid_removal.clone()).unwrap();
    let sealed_removal = seal_unvalidated_data(&valid_removal);
    assert_eq!(
        load_owned_snapshot(&sealed_removal.0, &sealed_removal.1).unwrap(),
        validated_removal
    );
    let mut missing_tombstone = valid_removal;
    let mut document = missing_tombstone.dictionary.document();
    document.metadata.owned_revisions[2].tombstones.clear();
    let dictionary = Dictionary::from_document(document).unwrap();
    missing_tombstone.commits[2].state_sha256 = sha256(&dictionary.to_json_bytes().unwrap());
    missing_tombstone.dictionary = dictionary;
    let sealed_invalid = seal_unvalidated_data(&missing_tombstone);
    assert!(matches!(
        OwnedSnapshot::from_data(missing_tombstone.clone()),
        Err(OwnedSnapshotError::History(message))
            if message == "new tombstones must exactly list removed prior assertions in order"
    ));
    assert!(matches!(
        load_owned_snapshot(&sealed_invalid.0, &sealed_invalid.1),
        Err(OwnedSnapshotError::History(message))
            if message == "new tombstones must exactly list removed prior assertions in order"
    ));
    assert_eq!(
        load_owned_snapshot(SCHEMA1_SNAPSHOT.as_bytes(), SCHEMA1_SEAL.as_bytes()).unwrap(),
        schema1
    );
    assert_eq!(
        load_owned_snapshot(SCHEMA2_SNAPSHOT.as_bytes(), SCHEMA2_SEAL.as_bytes()).unwrap(),
        schema2
    );
}

fn seal_unvalidated_data(data: &OwnedSnapshotData) -> (Vec<u8>, Vec<u8>) {
    let snapshot = serde_json::to_vec(data).unwrap();
    let seal = OwnedSnapshotSeal {
        version: 1,
        kind: "vlajmaji-owned-snapshot".into(),
        snapshot_version: data.version,
        journal_schema: data.journal_schema,
        dictionary_version: 1,
        registry_version: 1,
        source: data.source.clone(),
        head: data.head.clone(),
        revision_ids: data
            .commits
            .iter()
            .map(|commit| commit.revision.clone())
            .collect(),
        snapshot_sha256: sha256(&snapshot),
        snapshot_bytes: snapshot.len() as u64,
    }
    .to_json_bytes()
    .unwrap();
    (snapshot, seal)
}

fn schema1_with_r2_removing_b(schema1: &OwnedSnapshot) -> OwnedSnapshotData {
    let mut data = schema1.as_data().clone();
    let mut document = data.dictionary.document();
    let prior = document.metadata.owned_revisions[1].clone();
    let removed = prior.assertions[1].clone();
    let id = EntityId::new("owned-r2").unwrap();
    document.metadata.catalogue_revision = 2;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: id.clone(),
        source: prior.source,
        parent: Some(prior.id),
        editor: prior.editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![document.metadata.owned_revisions[1].assertions[0].clone()],
        tombstones: vec![removed],
        rationale: Presence::Value(TextValue {
            text: "Remove one reviewed assertion".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    document.metadata.owned_heads[0].revision = id.clone();
    let dictionary = Dictionary::from_document(document).unwrap();
    let mut prefixes = data.commits[1].prefixes.clone();
    prefixes
        .iter_mut()
        .find(|prefix| prefix.section == "metadata.owned_revisions")
        .unwrap()
        .length = 3;
    data.commits.push(OwnedSnapshotCommit {
        revision: id.clone(),
        state_sha256: sha256(&dictionary.to_json_bytes().unwrap()),
        prefixes,
    });
    data.dictionary = dictionary;
    data.head = id;
    data
}

#[test]
fn rich_complete_fixture_survives_one_commit_snapshot_codec_byte_for_byte() {
    let dictionary = rich_owned_genesis();
    let source = SourceId::new("source-o").unwrap();
    let revision = dictionary.metadata().owned_revisions[0].id.clone();
    let registry = dictionary.registry().as_document();
    let metadata = dictionary.metadata();
    let sections = [
        ("registry.sources", registry.sources.len()),
        ("registry.contributors", registry.contributors.len()),
        ("registry.distributors", registry.distributors.len()),
        ("registry.aliases", registry.aliases.len()),
        ("registry.lineage", registry.lineage.len()),
        ("registry.snapshots", registry.snapshots.len()),
        ("registry.occurrences", registry.occurrences.len()),
        ("captures", dictionary.captures().len()),
        ("metadata.code_notices", metadata.code_notices.len()),
        ("metadata.collections", metadata.collections.len()),
        ("metadata.components", metadata.components.len()),
        ("metadata.releases", metadata.releases.len()),
        ("metadata.entries", metadata.entries.len()),
        ("metadata.assertions", metadata.assertions.len()),
        ("metadata.contexts", metadata.contexts.len()),
        ("metadata.frames", metadata.frames.len()),
        ("metadata.forms", metadata.forms.len()),
        ("metadata.etymologies", metadata.etymologies.len()),
        ("metadata.relations", metadata.relations.len()),
        ("metadata.owned_revisions", metadata.owned_revisions.len()),
        ("metadata.editorial_senses", metadata.editorial_senses.len()),
        ("metadata.computations", metadata.computations.len()),
        ("metadata.annotations", metadata.annotations.len()),
        ("metadata.extensions", metadata.extensions.len()),
    ];
    assert_eq!(sections.len(), 24);
    assert!(sections.iter().all(|(_, length)| *length > 0));
    let prefixes = sections
        .into_iter()
        .map(|(section, length)| OwnedSectionPrefix {
            section: section.into(),
            length: length as u64,
        })
        .collect();
    let state_sha256 = sha256(&dictionary.to_json_bytes().unwrap());
    let snapshot = OwnedSnapshot::from_data(OwnedSnapshotData {
        version: 1,
        journal_schema: 1,
        source,
        head: revision.clone(),
        dictionary,
        commits: vec![OwnedSnapshotCommit {
            revision,
            state_sha256,
            prefixes,
        }],
        curation: None,
    })
    .unwrap();
    let sealed = seal_owned_snapshot(&snapshot).unwrap();
    assert_eq!(
        load_owned_snapshot(&sealed.snapshot, &sealed.seal).unwrap(),
        snapshot
    );
    assert_eq!(snapshot.to_json_bytes().unwrap(), sealed.snapshot);
}

fn rich_owned_genesis() -> Dictionary {
    let mut document = fixture::complete_fixture()
        .expect("complete fixture")
        .document();
    let owner = SourceId::new("source-o").unwrap();
    let mut registry = document.registry.document();
    registry.sources.retain(|source| source.id == owner);
    registry.sources[0].origin = SourceOrigin::OwnedDraft;
    registry.aliases.retain(|alias| alias.source == owner);
    registry.aliases.push(vlajmaji_lexicon::SourceAlias {
        source: owner.clone(),
        text: "owned-fixture-alias".into(),
        status: vlajmaji_lexicon::AliasStatus::Active,
        evidence: Some("synthetic contract fixture".into()),
    });
    registry.aliases.push(vlajmaji_lexicon::SourceAlias {
        source: owner.clone(),
        text: "owned-fixture-alias-second".into(),
        status: vlajmaji_lexicon::AliasStatus::Active,
        evidence: Some("second synthetic contract fixture alias".into()),
    });
    registry.lineage.retain(|edge| edge.source == owner);
    registry.lineage.push(vlajmaji_lexicon::SourceLineage {
        source: owner.clone(),
        upstream: vlajmaji_lexicon::LineageTarget::Unresolved("synthetic fixture target".into()),
        evidence: Some("synthetic contract fixture".into()),
    });
    registry.lineage.push(vlajmaji_lexicon::SourceLineage {
        source: owner.clone(),
        upstream: vlajmaji_lexicon::LineageTarget::Unresolved(
            "second synthetic fixture target".into(),
        ),
        evidence: Some("second synthetic contract fixture lineage".into()),
    });
    for edge in &mut registry.lineage {
        if let vlajmaji_lexicon::LineageTarget::Known(_) = edge.upstream {
            edge.upstream = vlajmaji_lexicon::LineageTarget::Unresolved(
                "synthetic retained lineage target".into(),
            );
        }
    }
    for snapshot in &mut registry.snapshots {
        snapshot.source = owner.clone();
    }
    document.registry = vlajmaji_lexicon::SourceRegistry::from_document(registry).unwrap();

    document.legacy_archive = None;
    document.metadata.catalogue_revision = 0;
    for collection in &mut document.metadata.collections {
        collection.source = owner.clone();
    }
    for entry in &mut document.metadata.entries {
        entry.source = owner.clone();
    }
    for relation in &mut document.metadata.relations {
        if let vlajmaji_lexicon::RelationTarget::Catalog(target) = &mut relation.target {
            target.source = owner.clone();
        }
    }
    for release in &mut document.metadata.releases {
        if release.content_notices.is_empty() {
            release
                .content_notices
                .push(vlajmaji_lexicon::ContentNotice {
                    text: Presence::Value("synthetic notice".into()),
                    locator: Presence::Missing,
                    license: Presence::Missing,
                });
        }
    }
    let mut ordered_extension = document.metadata.extensions[0].clone();
    ordered_extension.name = "synthetic-second-extension".into();
    ordered_extension.value = vlajmaji_lexicon::RawJson::new(r#"{"ordered":2}"#).unwrap();
    document.metadata.extensions.push(ordered_extension);
    let mut genesis_revision = document
        .metadata
        .owned_revisions
        .last()
        .expect("fixture owned head")
        .clone();
    genesis_revision.parent = None;
    genesis_revision.tombstones.clear();
    document.metadata.owned_revisions = vec![genesis_revision.clone()];
    document.metadata.owned_heads = vec![OwnedHead {
        source: owner,
        revision: genesis_revision.id,
    }];
    Dictionary::from_document(document).unwrap()
}
