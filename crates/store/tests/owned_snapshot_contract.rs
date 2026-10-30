//! Persistent export/import and exact-replay checks for complete owned history.

const SCHEMA1_SNAPSHOT: &str = r#####"{"version":1,"journal_schema":1,"source":"owned-example","head":"owned-r1","dictionary":{"version":1,"legacy_archive":null,"registry":{"version":1,"sources":[{"id":"owned-example","display_name":"Personal dictionary","abbreviation":"OWN","abbreviation_kind":"ProjectAssigned","abbreviation_evidence":"Example-assigned selector","origin":"OwnedDraft"}],"contributors":[{"id":"editor-example","display_name":"Dictionary editor"}],"distributors":[],"aliases":[],"lineage":[],"snapshots":[{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"source":"owned-example","distributor":null}],"occurrences":[{"occurrence":{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/a"},"contributor":"editor-example","native_entry_id":"farvi","native_assertion_id":"example-a"},{"occurrence":{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/b"},"contributor":"editor-example","native_entry_id":"farvi","native_assertion_id":"example-b"}]},"captures":[{"identity":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"metadata":{"source_name":"Example input","format_label":"application/json","locator":"seed/edit example","revision":"Missing","captured_at":"Missing","time_precision":"Unknown","notice_locator":"Missing"},"bytes":[123,34,119,111,114,100,34,58,34,102,97,114,118,105,34,44,34,100,101,102,105,110,105,116,105,111,110,115,34,58,91,34,103,114,111,119,34,44,34,100,101,118,101,108,111,112,34,93,125],"occurrences":[{"locator":"/a","fields":[{"name":"definition","encoding":"Text","raw_value":"grow"}],"raw_line":"farvi — grow"},{"locator":"/b","fields":[{"name":"definition","encoding":"Text","raw_value":"develop"}],"raw_line":"farvi — develop"}]}],"metadata":{"catalogue_revision":1,"code_notices":[],"collections":[{"id":"owned-collection","source":"owned-example","display_name":"My dictionary","description":{"Value":{"text":"An explicitly owned personal dictionary","language":"Missing","markup":"Missing"}}}],"components":[{"id":"owned-component","collection":"owned-collection","native_name":"owned lexical entries","languages":["jbo"]}],"releases":[{"id":"owned-release-r0","component":"owned-component","distributor":null,"upstream_revision":"Missing","distributor_revision":"Missing","captured_at":"Missing","time_precision":"Unknown","schema_version":1,"normalizer":"owned-example","normalizer_version":"1","captures":[],"assertions":[],"accounting":{"input_occurrences":0,"normalized_occurrences":0,"query_excluded_occurrences":0,"unit":"occurrences"},"diagnostics":[],"selected":true,"availability":"Available","content_notices":[{"text":{"Value":"example data only; no licence grant"},"locator":"Missing","license":"Missing"}],"redistribution":"Held","redistribution_evidence":[]},{"id":"owned-release-r1","component":"owned-component","distributor":null,"upstream_revision":"Missing","distributor_revision":"Missing","captured_at":"Missing","time_precision":"Unknown","schema_version":1,"normalizer":"owned-example","normalizer_version":"1","captures":[{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"}],"assertions":["owned-assertion-a","owned-assertion-b"],"accounting":{"input_occurrences":2,"normalized_occurrences":2,"query_excluded_occurrences":0,"unit":"occurrences"},"diagnostics":[],"selected":true,"availability":"Available","content_notices":[{"text":{"Value":"example data only; no licence grant"},"locator":"Missing","license":"Missing"}],"redistribution":"Held","redistribution_evidence":[]}],"entries":[{"id":"owned-entry-farvi","source":"owned-example","spelling":{"Value":"farvi"},"language":{"Value":"jbo"},"script":"Missing","native_word_class":"Missing","selmaho":"Null","mapped_pos":"Missing"}],"assertions":[{"id":"owned-assertion-a","entry":"owned-entry-farvi","occurrence":{"capture":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/a"},"native_sense_id":"Missing","language":{"Value":"en"},"definition":{"Value":{"text":"grow","language":"Missing","markup":"Missing"}},"source_score":"Null","timestamps":[],"eligibility":"Lexical"},{"id":"owned-assertion-b","entry":"owned-entry-farvi","occurrence":{"capture":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/b"},"native_sense_id":"Missing","language":{"Value":"en"},"definition":{"Value":{"text":"develop","language":"Missing","markup":"Missing"}},"source_score":"Null","timestamps":[],"eligibility":"Lexical"}],"contexts":[],"frames":[],"forms":[],"etymologies":[],"relations":[],"owned_revisions":[{"id":"owned-r0","source":"owned-example","parent":null,"editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":[],"tombstones":[],"rationale":{"Value":{"text":"Start with an empty active set","language":"Missing","markup":"Missing"}}},{"id":"owned-r1","source":"owned-example","parent":"owned-r0","editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":["owned-assertion-a","owned-assertion-b"],"tombstones":[],"rationale":{"Value":{"text":"Add two source-distinct definitions","language":"Missing","markup":"Missing"}}}],"owned_heads":[{"source":"owned-example","revision":"owned-r1"}],"editorial_senses":[],"computations":[],"annotations":[],"extensions":[]}},"commits":[{"revision":"owned-r0","state_sha256":"178cb4a48245db47a568bb740f5ec123575aae38eb961c9ce05f40c7f0526c9a","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":0},{"section":"registry.occurrences","length":0},{"section":"captures","length":0},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":1},{"section":"metadata.entries","length":0},{"section":"metadata.assertions","length":0},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":1},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]},{"revision":"owned-r1","state_sha256":"e96d7fdee27a1cf84b152799b51c7b6e541aeae714396665fa98985f5c81d05f","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":1},{"section":"registry.occurrences","length":2},{"section":"captures","length":1},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":2},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":2},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":2},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]}],"curation":null}"#####;
const SCHEMA1_SEAL: &str = r#####"{"version":1,"kind":"vlajmaji-owned-snapshot","snapshot_version":1,"journal_schema":1,"dictionary_version":1,"registry_version":1,"source":"owned-example","head":"owned-r1","revision_ids":["owned-r0","owned-r1"],"snapshot_sha256":"a16d6f1c3112bb62fa3dd091d6a972e17c0f084dc433f4f3110584faef090a05","snapshot_bytes":8040}"#####;
const SCHEMA2_SNAPSHOT: &str = r#####"{"version":1,"journal_schema":2,"source":"owned-example","head":"owned-r4","dictionary":{"version":1,"legacy_archive":null,"registry":{"version":1,"sources":[{"id":"owned-example","display_name":"Personal dictionary","abbreviation":"OWN","abbreviation_kind":"ProjectAssigned","abbreviation_evidence":"Example-assigned selector","origin":"OwnedDraft"}],"contributors":[{"id":"editor-example","display_name":"Dictionary editor"}],"distributors":[],"aliases":[],"lineage":[],"snapshots":[{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"source":"owned-example","distributor":null}],"occurrences":[{"occurrence":{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/a"},"contributor":"editor-example","native_entry_id":"farvi","native_assertion_id":"example-a"},{"occurrence":{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/b"},"contributor":"editor-example","native_entry_id":"farvi","native_assertion_id":"example-b"}]},"captures":[{"identity":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"metadata":{"source_name":"Example input","format_label":"application/json","locator":"seed/edit example","revision":"Missing","captured_at":"Missing","time_precision":"Unknown","notice_locator":"Missing"},"bytes":[123,34,119,111,114,100,34,58,34,102,97,114,118,105,34,44,34,100,101,102,105,110,105,116,105,111,110,115,34,58,91,34,103,114,111,119,34,44,34,100,101,118,101,108,111,112,34,93,125],"occurrences":[{"locator":"/a","fields":[{"name":"definition","encoding":"Text","raw_value":"grow"}],"raw_line":"farvi — grow"},{"locator":"/b","fields":[{"name":"definition","encoding":"Text","raw_value":"develop"}],"raw_line":"farvi — develop"}]}],"metadata":{"catalogue_revision":4,"code_notices":[],"collections":[{"id":"owned-collection","source":"owned-example","display_name":"My dictionary","description":{"Value":{"text":"An explicitly owned personal dictionary","language":"Missing","markup":"Missing"}}}],"components":[{"id":"owned-component","collection":"owned-collection","native_name":"owned lexical entries","languages":["jbo"]}],"releases":[{"id":"owned-release-r0","component":"owned-component","distributor":null,"upstream_revision":"Missing","distributor_revision":"Missing","captured_at":"Missing","time_precision":"Unknown","schema_version":1,"normalizer":"owned-example","normalizer_version":"1","captures":[],"assertions":[],"accounting":{"input_occurrences":0,"normalized_occurrences":0,"query_excluded_occurrences":0,"unit":"occurrences"},"diagnostics":[],"selected":true,"availability":"Available","content_notices":[{"text":{"Value":"example data only; no licence grant"},"locator":"Missing","license":"Missing"}],"redistribution":"Held","redistribution_evidence":[]},{"id":"owned-release-r1","component":"owned-component","distributor":null,"upstream_revision":"Missing","distributor_revision":"Missing","captured_at":"Missing","time_precision":"Unknown","schema_version":1,"normalizer":"owned-example","normalizer_version":"1","captures":[{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"}],"assertions":["owned-assertion-a","owned-assertion-b"],"accounting":{"input_occurrences":2,"normalized_occurrences":2,"query_excluded_occurrences":0,"unit":"occurrences"},"diagnostics":[],"selected":true,"availability":"Available","content_notices":[{"text":{"Value":"example data only; no licence grant"},"locator":"Missing","license":"Missing"}],"redistribution":"Held","redistribution_evidence":[]}],"entries":[{"id":"owned-entry-farvi","source":"owned-example","spelling":{"Value":"farvi"},"language":{"Value":"jbo"},"script":"Missing","native_word_class":"Missing","selmaho":"Null","mapped_pos":"Missing"}],"assertions":[{"id":"owned-assertion-a","entry":"owned-entry-farvi","occurrence":{"capture":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/a"},"native_sense_id":"Missing","language":{"Value":"en"},"definition":{"Value":{"text":"grow","language":"Missing","markup":"Missing"}},"source_score":"Null","timestamps":[],"eligibility":"Lexical"},{"id":"owned-assertion-b","entry":"owned-entry-farvi","occurrence":{"capture":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/b"},"native_sense_id":"Missing","language":{"Value":"en"},"definition":{"Value":{"text":"develop","language":"Missing","markup":"Missing"}},"source_score":"Null","timestamps":[],"eligibility":"Lexical"}],"contexts":[],"frames":[],"forms":[],"etymologies":[],"relations":[],"owned_revisions":[{"id":"owned-r0","source":"owned-example","parent":null,"editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":[],"tombstones":[],"rationale":{"Value":{"text":"Start with an empty active set","language":"Missing","markup":"Missing"}}},{"id":"owned-r1","source":"owned-example","parent":"owned-r0","editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":["owned-assertion-a","owned-assertion-b"],"tombstones":[],"rationale":{"Value":{"text":"Add two source-distinct definitions","language":"Missing","markup":"Missing"}}},{"id":"owned-r2","source":"owned-example","parent":"owned-r1","editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":["owned-assertion-a","owned-assertion-b"],"tombstones":[],"rationale":{"Value":{"text":"Review explicit own and external occurrences","language":"Missing","markup":"Missing"}}},{"id":"owned-r3","source":"owned-example","parent":"owned-r2","editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":["owned-assertion-a","owned-assertion-b"],"tombstones":[],"rationale":{"Value":{"text":"Explicit curation selection owned-r3","language":"Missing","markup":"Missing"}}},{"id":"owned-r4","source":"owned-example","parent":"owned-r3","editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":["owned-assertion-a","owned-assertion-b"],"tombstones":[],"rationale":{"Value":{"text":"Explicit curation selection owned-r4","language":"Missing","markup":"Missing"}}}],"owned_heads":[{"source":"owned-example","revision":"owned-r4"}],"editorial_senses":[],"computations":[],"annotations":[],"extensions":[]}},"commits":[{"revision":"owned-r0","state_sha256":"178cb4a48245db47a568bb740f5ec123575aae38eb961c9ce05f40c7f0526c9a","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":0},{"section":"registry.occurrences","length":0},{"section":"captures","length":0},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":1},{"section":"metadata.entries","length":0},{"section":"metadata.assertions","length":0},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":1},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]},{"revision":"owned-r1","state_sha256":"e96d7fdee27a1cf84b152799b51c7b6e541aeae714396665fa98985f5c81d05f","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":1},{"section":"registry.occurrences","length":2},{"section":"captures","length":1},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":2},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":2},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":2},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]},{"revision":"owned-r2","state_sha256":"1cdf04f2a5dfc078ebc7c55416821a2a62a5835865c21846a78d289b142cef3a","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":1},{"section":"registry.occurrences","length":2},{"section":"captures","length":1},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":2},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":2},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":3},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]},{"revision":"owned-r3","state_sha256":"d5279857727404503d065e4760960e2665d0e3b617cd5d88249eba53fb4e3917","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":1},{"section":"registry.occurrences","length":2},{"section":"captures","length":1},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":2},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":2},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":4},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]},{"revision":"owned-r4","state_sha256":"5d912d6f36fe0df64045a862ed4dd228fde1caf2d2371979e9e86f758b0ab1a5","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":1},{"section":"registry.occurrences","length":2},{"section":"captures","length":1},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":2},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":2},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":5},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]}],"curation":{"version":1,"contexts":[{"id":"cdc-context","model_sha256":"db8483868e595b42c412632c0b5b90e101d5a25a4f74bc2f5d1546de94ce0f13","source_ids":["lensisku"],"release_ids":["la-lojban-release-e9aeeda43d145fac9dd2430a88c958be771edfae8e31b7b8e0b9a82f0b7c756b"]}],"records":[{"id":"cdc-reviewed-sense","revision":"owned-r2","editor":"editor-example","rationale":{"text":"Review two retained owned assertions and one supplied occurrence","language":"Missing","markup":"Missing"},"kind":{"sense":{"members":[{"owned_assertion":"owned-assertion-a"},{"owned_assertion":"owned-assertion-b"},{"external":{"context":"cdc-context","target":{"source":"lensisku","release":"la-lojban-release-e9aeeda43d145fac9dd2430a88c958be771edfae8e31b7b8e0b9a82f0b7c756b","native_entry_id":null,"native_assertion_id":"7","occurrence":{"capture":{"namespace":"captured:lensisku:la-lojban:en","sha256":"4667ccabc6b00fd874b95a4e12d0639fa7dfb85aa0fc1046e8108ef198332a15"},"locator":"/0"}},"entry":"la-lojban-4667ccabc6b00fd874b95a4e12d0639fa7dfb85aa0fc1046e8108ef198332a15-entry-0","assertion":"la-lojban-4667ccabc6b00fd874b95a4e12d0639fa7dfb85aa0fc1046e8108ef198332a15-assertion-0"}}]}}},{"id":"cdc-reviewed-link","revision":"owned-r2","editor":"editor-example","rationale":{"text":"Link the reviewed sense to the pinned external occurrence","language":"Missing","markup":"Missing"},"kind":{"link":{"origin":{"EditorialSense":"cdc-reviewed-sense"},"direction":"Forward","target":{"context":"cdc-context","target":{"source":"lensisku","release":"la-lojban-release-e9aeeda43d145fac9dd2430a88c958be771edfae8e31b7b8e0b9a82f0b7c756b","native_entry_id":null,"native_assertion_id":"7","occurrence":{"capture":{"namespace":"captured:lensisku:la-lojban:en","sha256":"4667ccabc6b00fd874b95a4e12d0639fa7dfb85aa0fc1046e8108ef198332a15"},"locator":"/0"}},"entry":"la-lojban-4667ccabc6b00fd874b95a4e12d0639fa7dfb85aa0fc1046e8108ef198332a15-entry-0","assertion":"la-lojban-4667ccabc6b00fd874b95a4e12d0639fa7dfb85aa0fc1046e8108ef198332a15-assertion-0"}}}}],"revisions":[{"revision":"owned-r0","active":[],"tombstones":[]},{"revision":"owned-r1","active":[],"tombstones":[]},{"revision":"owned-r2","active":["cdc-reviewed-sense","cdc-reviewed-link"],"tombstones":[]},{"revision":"owned-r3","active":["cdc-reviewed-sense"],"tombstones":["cdc-reviewed-link"]},{"revision":"owned-r4","active":["cdc-reviewed-sense","cdc-reviewed-link"],"tombstones":[]}],"prefixes":[{"revision":"owned-r0","contexts_length":0,"records_length":0,"curation_sha256":"7a9ff76782384a8d55e2ccf5505759d2c6b709ec3a56a86294f70e78d3b7a2e0"},{"revision":"owned-r1","contexts_length":0,"records_length":0,"curation_sha256":"8003d426f599021ac75733efd9cc9462e2f99b3bd421c423a256e0d71596bf2b"},{"revision":"owned-r2","contexts_length":1,"records_length":2,"curation_sha256":"b6ec1a55d9602412849c910464996899f0b015899bd68a7002031e7cc2a445a9"},{"revision":"owned-r3","contexts_length":1,"records_length":2,"curation_sha256":"36ee700b310f64f599327f22d7017eacc32705bc051aa514a55675b6f4520e18"},{"revision":"owned-r4","contexts_length":1,"records_length":2,"curation_sha256":"fdf5d1d7ec48458d4815b2230ee5086f2709e028d7fa1416111337674062ef4a"}]}}"#####;
const SCHEMA2_SEAL: &str = r#####"{"version":1,"kind":"vlajmaji-owned-snapshot","snapshot_version":1,"journal_schema":2,"dictionary_version":1,"registry_version":1,"source":"owned-example","head":"owned-r4","revision_ids":["owned-r0","owned-r1","owned-r2","owned-r3","owned-r4"],"snapshot_sha256":"2d4925e6baf72044b71cae7a78f466412276b3d837765a6f0b1ed59fe58c3d8a","snapshot_bytes":15705}"#####;
const ALTERNATE_SNAPSHOT: &str = r#####"{"version":1,"journal_schema":1,"source":"owned-example","head":"owned-r1","dictionary":{"version":1,"legacy_archive":null,"registry":{"version":1,"sources":[{"id":"owned-example","display_name":"Personal dictionary","abbreviation":"OWN","abbreviation_kind":"ProjectAssigned","abbreviation_evidence":"Example-assigned selector","origin":"OwnedDraft"}],"contributors":[{"id":"editor-example","display_name":"Dictionary editor"}],"distributors":[],"aliases":[],"lineage":[],"snapshots":[{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"source":"owned-example","distributor":null}],"occurrences":[{"occurrence":{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/a"},"contributor":"editor-example","native_entry_id":"farvi","native_assertion_id":"example-a"},{"occurrence":{"snapshot":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/b"},"contributor":"editor-example","native_entry_id":"farvi","native_assertion_id":"example-b"}]},"captures":[{"identity":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"metadata":{"source_name":"Example input","format_label":"application/json","locator":"seed/edit example","revision":"Missing","captured_at":"Missing","time_precision":"Unknown","notice_locator":"Missing"},"bytes":[123,34,119,111,114,100,34,58,34,102,97,114,118,105,34,44,34,100,101,102,105,110,105,116,105,111,110,115,34,58,91,34,103,114,111,119,34,44,34,100,101,118,101,108,111,112,34,93,125],"occurrences":[{"locator":"/a","fields":[{"name":"definition","encoding":"Text","raw_value":"grow"}],"raw_line":"farvi — grow"},{"locator":"/b","fields":[{"name":"definition","encoding":"Text","raw_value":"develop"}],"raw_line":"farvi — develop"}]}],"metadata":{"catalogue_revision":1,"code_notices":[],"collections":[{"id":"owned-collection","source":"owned-example","display_name":"My dictionary","description":{"Value":{"text":"An explicitly owned personal dictionary","language":"Missing","markup":"Missing"}}}],"components":[{"id":"owned-component","collection":"owned-collection","native_name":"owned lexical entries","languages":["jbo"]}],"releases":[{"id":"owned-release-r0","component":"owned-component","distributor":null,"upstream_revision":"Missing","distributor_revision":"Missing","captured_at":"Missing","time_precision":"Unknown","schema_version":1,"normalizer":"owned-example","normalizer_version":"1","captures":[],"assertions":[],"accounting":{"input_occurrences":0,"normalized_occurrences":0,"query_excluded_occurrences":0,"unit":"occurrences"},"diagnostics":[],"selected":true,"availability":"Available","content_notices":[{"text":{"Value":"example data only; no licence grant"},"locator":"Missing","license":"Missing"}],"redistribution":"Held","redistribution_evidence":[]},{"id":"owned-release-r1","component":"owned-component","distributor":null,"upstream_revision":"Missing","distributor_revision":"Missing","captured_at":"Missing","time_precision":"Unknown","schema_version":1,"normalizer":"owned-example","normalizer_version":"1","captures":[{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"}],"assertions":["owned-assertion-a","owned-assertion-b"],"accounting":{"input_occurrences":2,"normalized_occurrences":2,"query_excluded_occurrences":0,"unit":"occurrences"},"diagnostics":[],"selected":true,"availability":"Available","content_notices":[{"text":{"Value":"example data only; no licence grant"},"locator":"Missing","license":"Missing"}],"redistribution":"Held","redistribution_evidence":[]}],"entries":[{"id":"owned-entry-farvi","source":"owned-example","spelling":{"Value":"farvi"},"language":{"Value":"jbo"},"script":"Missing","native_word_class":"Missing","selmaho":"Null","mapped_pos":"Missing"}],"assertions":[{"id":"owned-assertion-a","entry":"owned-entry-farvi","occurrence":{"capture":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/a"},"native_sense_id":"Missing","language":{"Value":"en"},"definition":{"Value":{"text":"grow","language":"Missing","markup":"Missing"}},"source_score":"Null","timestamps":[],"eligibility":"Lexical"},{"id":"owned-assertion-b","entry":"owned-entry-farvi","occurrence":{"capture":{"namespace":"owned-example-capture","sha256":"2228dc977c90f4422c58d1120b6875d03219b89eabe252801fd013c7ebf5645c"},"locator":"/b"},"native_sense_id":"Missing","language":{"Value":"en"},"definition":{"Value":{"text":"develop","language":"Missing","markup":"Missing"}},"source_score":"Null","timestamps":[],"eligibility":"Lexical"}],"contexts":[],"frames":[],"forms":[],"etymologies":[],"relations":[],"owned_revisions":[{"id":"owned-r0","source":"owned-example","parent":null,"editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":[],"tombstones":[],"rationale":{"Value":{"text":"Start with an empty active set","language":"Missing","markup":"Missing"}}},{"id":"owned-r1","source":"owned-example","parent":"owned-r0","editor":"editor-example","timestamp":"Missing","time_precision":"Unknown","assertions":["owned-assertion-a","owned-assertion-b"],"tombstones":[],"rationale":{"Value":{"text":"Add two source-distinct definitions","language":"Missing","markup":"Missing"}}}],"owned_heads":[{"source":"owned-example","revision":"owned-r1"}],"editorial_senses":[],"computations":[],"annotations":[],"extensions":[]}},"commits":[{"revision":"owned-r0","state_sha256":"e674cb41a0b83b2db2165fb5d966e78e8ad460e6e821304c6e9e2ab8841a1a47","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":0},{"section":"registry.occurrences","length":0},{"section":"captures","length":0},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":1},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":0},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":1},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]},{"revision":"owned-r1","state_sha256":"e96d7fdee27a1cf84b152799b51c7b6e541aeae714396665fa98985f5c81d05f","prefixes":[{"section":"registry.sources","length":1},{"section":"registry.contributors","length":1},{"section":"registry.distributors","length":0},{"section":"registry.aliases","length":0},{"section":"registry.lineage","length":0},{"section":"registry.snapshots","length":1},{"section":"registry.occurrences","length":2},{"section":"captures","length":1},{"section":"metadata.code_notices","length":0},{"section":"metadata.collections","length":1},{"section":"metadata.components","length":1},{"section":"metadata.releases","length":2},{"section":"metadata.entries","length":1},{"section":"metadata.assertions","length":2},{"section":"metadata.contexts","length":0},{"section":"metadata.frames","length":0},{"section":"metadata.forms","length":0},{"section":"metadata.etymologies","length":0},{"section":"metadata.relations","length":0},{"section":"metadata.owned_revisions","length":2},{"section":"metadata.editorial_senses","length":0},{"section":"metadata.computations","length":0},{"section":"metadata.annotations","length":0},{"section":"metadata.extensions","length":0}]}],"curation":null}"#####;
const ALTERNATE_SEAL: &str = r#####"{"version":1,"kind":"vlajmaji-owned-snapshot","snapshot_version":1,"journal_schema":1,"dictionary_version":1,"registry_version":1,"source":"owned-example","head":"owned-r1","revision_ids":["owned-r0","owned-r1"],"snapshot_sha256":"a85fd0340209ebd49161aded45cceaa06f4773e9de7358cf14c1d3e707f236f7","snapshot_bytes":8040}"#####;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use rusqlite::types::ValueRef;
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    Dictionary, DictionaryDocument, EntityId, OwnedCurationData, OwnedCurationDocument, OwnedHead,
    OwnedRevisionRecord, OwnedSectionPrefix, OwnedSnapshot, OwnedSnapshotCommit, OwnedSnapshotData,
    Presence, SourceId, SourceOrigin, TextValue, TimePrecision, load_owned_snapshot,
    seal_owned_snapshot,
};
use vlajmaji_store::{
    OwnedImportOutcome, OwnedJournal, OwnedJournalErrorCategory, OwnedJournalReader,
};

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root(label: &str) -> PathBuf {
    for _ in 0..64 {
        let parent = std::env::temp_dir().join(format!(
            "vlajmaji-owned-snapshot-{label}-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&parent) {
            Ok(()) => return parent.join("journal"),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create isolated parent: {error}"),
        }
    }
    panic!("could not allocate an exclusive snapshot parent")
}

#[derive(Debug, PartialEq, Eq)]
enum SqlValue {
    Null,
    Integer(i64),
    Text(String),
    Blob(Vec<u8>),
}

fn logical_sql(root: &Path) -> Vec<(String, Vec<Vec<SqlValue>>)> {
    let connection = rusqlite::Connection::open(root.join("owned.sqlite")).unwrap();
    let tables = [
        (
            "owned_meta",
            "SELECT singleton,family,schema_version,model_version,registry_version,source_id,head_revision,catalogue_revision FROM owned_meta ORDER BY singleton",
        ),
        (
            "owned_records",
            "SELECT section,ordinal,payload FROM owned_records ORDER BY section,ordinal",
        ),
        (
            "owned_commits",
            "SELECT revision_id,parent_revision,sequence,state_sha256 FROM owned_commits ORDER BY sequence",
        ),
        (
            "owned_commit_sections",
            "SELECT revision_id,section,length FROM owned_commit_sections ORDER BY revision_id,section",
        ),
        (
            "owned_context_pins",
            "SELECT ordinal,id,payload FROM owned_context_pins ORDER BY ordinal",
        ),
        (
            "owned_curation_records",
            "SELECT ordinal,id,payload FROM owned_curation_records ORDER BY ordinal",
        ),
        (
            "owned_curation_revisions",
            "SELECT revision_id,payload FROM owned_curation_revisions ORDER BY revision_id",
        ),
        (
            "owned_curation_prefixes",
            "SELECT revision_id,contexts_length,records_length,curation_sha256 FROM owned_curation_prefixes ORDER BY revision_id",
        ),
    ];
    let mut result = Vec::new();
    for (table, sql) in tables {
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
                [table],
                |row| row.get(0),
            )
            .unwrap();
        if !exists {
            continue;
        }
        let mut statement = connection.prepare(sql).unwrap();
        let columns = statement.column_count();
        let mut rows = statement.query([]).unwrap();
        let mut table_rows = Vec::new();
        while let Some(row) = rows.next().unwrap() {
            let mut cells = Vec::new();
            for index in 0..columns {
                cells.push(match row.get_ref(index).unwrap() {
                    ValueRef::Null => SqlValue::Null,
                    ValueRef::Integer(value) => SqlValue::Integer(value),
                    ValueRef::Text(value) => {
                        SqlValue::Text(String::from_utf8(value.to_vec()).unwrap())
                    }
                    ValueRef::Blob(value) => SqlValue::Blob(value.to_vec()),
                    ValueRef::Real(_) => panic!("unexpected REAL in owned journal schema"),
                });
            }
            table_rows.push(cells);
        }
        result.push((table.to_owned(), table_rows));
    }
    result
}

fn embedded_snapshot(payload: &str, seal: &str) -> OwnedSnapshot {
    load_owned_snapshot(payload.as_bytes(), seal.as_bytes()).unwrap()
}

fn next_snapshot_revision(current: &Dictionary) -> Dictionary {
    let mut document = current.document();
    let prior = document.metadata.owned_revisions.last().unwrap().clone();
    let id = EntityId::new("owned-r2").unwrap();
    document.metadata.catalogue_revision += 1;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: id.clone(),
        source: prior.source,
        parent: Some(prior.id),
        editor: prior.editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: document
            .metadata
            .assertions
            .iter()
            .map(|row| row.id.clone())
            .collect(),
        tombstones: Vec::new(),
        rationale: Presence::Value(TextValue {
            text: "Snapshot API Busy retry control".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    document.metadata.owned_heads[0].revision = id;
    Dictionary::from_document(document).unwrap()
}

fn renamed_source_snapshot(input: &OwnedSnapshot) -> OwnedSnapshot {
    fn rename(document: &mut DictionaryDocument) {
        let mut registry = document.registry.document();
        registry.sources[0].display_name = "Alternate valid source display".into();
        document.registry = vlajmaji_lexicon::SourceRegistry::from_document(registry).unwrap();
    }

    let mut data = input.as_data().clone();
    let mut final_document = data.dictionary.document();
    rename(&mut final_document);
    data.dictionary = Dictionary::from_document(final_document).unwrap();
    for commit in &mut data.commits {
        let mut state = input.dictionary_at(&commit.revision).unwrap().document();
        rename(&mut state);
        commit.state_sha256 = sha256(
            &Dictionary::from_document(state)
                .unwrap()
                .to_json_bytes()
                .unwrap(),
        );
    }
    OwnedSnapshot::from_data(data).unwrap()
}

fn changed_curation_snapshot(input: &OwnedSnapshot) -> OwnedSnapshot {
    let mut data = input.as_data().clone();
    let curation = data.curation.as_mut().expect("schema-two curation");
    curation.records[0]
        .rationale
        .text
        .push_str("; alternate valid review");
    for (index, commit) in data.commits.iter().enumerate() {
        let contexts_length = usize::try_from(curation.prefixes[index].contexts_length).unwrap();
        let records_length = usize::try_from(curation.prefixes[index].records_length).unwrap();
        let state = OwnedCurationDocument::from_data(OwnedCurationData {
            version: curation.version,
            dictionary: input.dictionary_at(&commit.revision).unwrap(),
            contexts: curation.contexts[..contexts_length].to_vec(),
            records: curation.records[..records_length].to_vec(),
            revisions: curation.revisions[..=index].to_vec(),
        })
        .unwrap();
        curation.prefixes[index].curation_sha256 = state.state_sha256().unwrap();
    }
    OwnedSnapshot::from_data(data).unwrap()
}

#[test]
fn schema_one_two_import_export_and_exact_replay_preserve_whole_history() {
    for (label, payload, seal, schema, expected_count) in [
        ("schema1", SCHEMA1_SNAPSHOT, SCHEMA1_SEAL, 1, 2),
        ("schema2", SCHEMA2_SNAPSHOT, SCHEMA2_SEAL, 2, 5),
    ] {
        let expected = embedded_snapshot(payload, seal);
        let root = fresh_root(label);
        let (writer, report) = OwnedJournal::import_snapshot(&root, &expected).unwrap();
        assert_eq!(report.outcome, OwnedImportOutcome::Imported);
        assert_eq!(report.journal_schema, schema);
        assert_eq!(report.revision_count, expected_count);
        assert_eq!(report.head, expected.as_data().head);
        drop(writer);

        let header = rusqlite::Connection::open(root.join("owned.sqlite")).unwrap();
        let pragma: i32 = header
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        let metadata: i32 = header
            .query_row(
                "SELECT schema_version FROM owned_meta WHERE singleton=1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let expected_schema = i32::try_from(schema).expect("fixture schema fits i32");
        assert_eq!(pragma, expected_schema);
        assert_eq!(metadata, expected_schema);
        drop(header);

        let reader = OwnedJournalReader::open(&root).unwrap();
        let exported = reader.snapshot().unwrap();
        assert_eq!(exported, expected);
        assert_eq!(
            seal_owned_snapshot(&exported).unwrap().snapshot,
            payload.as_bytes()
        );
        assert_eq!(
            reader.history().unwrap().commits.len() as u64,
            expected_count
        );
        drop(reader);

        let before_replay = logical_sql(&root);
        let (writer, replay) = OwnedJournal::import_snapshot(&root, &expected).unwrap();
        assert_eq!(replay.outcome, OwnedImportOutcome::AlreadyPresent);
        assert_eq!(logical_sql(&root), before_replay);
        drop(writer);
    }

    let original = embedded_snapshot(SCHEMA1_SNAPSHOT, SCHEMA1_SEAL);
    let alternate = embedded_snapshot(ALTERNATE_SNAPSHOT, ALTERNATE_SEAL);
    assert_eq!(original.as_data().head, alternate.as_data().head);
    assert_eq!(
        original.as_data().dictionary,
        alternate.as_data().dictionary
    );
    let root = fresh_root("same-head-alternate");
    let (writer, _) = OwnedJournal::import_snapshot(&root, &original).unwrap();
    drop(writer);
    let before_conflict = logical_sql(&root);
    let error = match OwnedJournal::import_snapshot(&root, &alternate) {
        Err(error) => error,
        Ok(_) => panic!("a different old history must not replay as identical"),
    };
    assert_eq!(error.category(), OwnedJournalErrorCategory::ImportConflict);
    assert_eq!(logical_sql(&root), before_conflict);

    for (label, resident, incoming) in [
        (
            "schema-one-then-two",
            embedded_snapshot(SCHEMA1_SNAPSHOT, SCHEMA1_SEAL),
            embedded_snapshot(SCHEMA2_SNAPSHOT, SCHEMA2_SEAL),
        ),
        (
            "schema-two-then-one",
            embedded_snapshot(SCHEMA2_SNAPSHOT, SCHEMA2_SEAL),
            embedded_snapshot(SCHEMA1_SNAPSHOT, SCHEMA1_SEAL),
        ),
    ] {
        let root = fresh_root(label);
        let (writer, _) = OwnedJournal::import_snapshot(&root, &resident).unwrap();
        drop(writer);
        let before_conflict = logical_sql(&root);
        let error = match OwnedJournal::import_snapshot(&root, &incoming) {
            Err(error) => error,
            Ok(_) => panic!("different journal schemas must conflict"),
        };
        assert_eq!(error.category(), OwnedJournalErrorCategory::ImportConflict);
        assert_eq!(logical_sql(&root), before_conflict);
    }
}

#[test]
fn snapshot_import_conflicts_on_same_head_body_sidecar_and_history_extension() {
    let schema1 = embedded_snapshot(SCHEMA1_SNAPSHOT, SCHEMA1_SEAL);
    let renamed = renamed_source_snapshot(&schema1);
    assert_eq!(renamed.as_data().head, schema1.as_data().head);
    assert_ne!(renamed.as_data().dictionary, schema1.as_data().dictionary);
    let renamed_bytes = seal_owned_snapshot(&renamed).unwrap();
    assert_eq!(
        load_owned_snapshot(&renamed_bytes.snapshot, &renamed_bytes.seal).unwrap(),
        renamed
    );

    let root = fresh_root("same-head-body-conflict");
    let (writer, _) = OwnedJournal::import_snapshot(&root, &schema1).unwrap();
    drop(writer);
    let before = logical_sql(&root);
    let error = OwnedJournal::import_snapshot(&root, &renamed)
        .err()
        .expect("same-head body change must conflict");
    assert_eq!(error.category(), OwnedJournalErrorCategory::ImportConflict);
    assert_eq!(logical_sql(&root), before);

    let schema2 = embedded_snapshot(SCHEMA2_SNAPSHOT, SCHEMA2_SEAL);
    let altered_curation = changed_curation_snapshot(&schema2);
    assert_eq!(altered_curation.as_data().head, schema2.as_data().head);
    assert_eq!(
        altered_curation.as_data().dictionary,
        schema2.as_data().dictionary
    );
    assert_ne!(
        altered_curation.as_data().curation,
        schema2.as_data().curation
    );
    let altered_bytes = seal_owned_snapshot(&altered_curation).unwrap();
    assert_eq!(
        load_owned_snapshot(&altered_bytes.snapshot, &altered_bytes.seal).unwrap(),
        altered_curation
    );
    let root = fresh_root("same-head-curation-conflict");
    let (writer, _) = OwnedJournal::import_snapshot(&root, &schema2).unwrap();
    drop(writer);
    let before = logical_sql(&root);
    let error = OwnedJournal::import_snapshot(&root, &altered_curation)
        .err()
        .expect("same-head curation change must conflict");
    assert_eq!(error.category(), OwnedJournalErrorCategory::ImportConflict);
    assert_eq!(logical_sql(&root), before);

    let extension_source = fresh_root("snapshot-extension-source");
    let (writer, _) = OwnedJournal::import_snapshot(&extension_source, &schema1).unwrap();
    drop(writer);
    let mut writer = OwnedJournal::open(&extension_source).unwrap();
    let candidate = next_snapshot_revision(&schema1.as_data().dictionary);
    writer.commit(&schema1.as_data().head, &candidate).unwrap();
    drop(writer);
    let extended = OwnedJournalReader::open(&extension_source)
        .unwrap()
        .snapshot()
        .unwrap();
    assert_eq!(extended.as_data().head.as_str(), "owned-r2");
    assert_eq!(extended.as_data().dictionary, candidate);

    let older_resident = fresh_root("snapshot-extension-r1-resident");
    let (writer, _) = OwnedJournal::import_snapshot(&older_resident, &schema1).unwrap();
    drop(writer);
    let before = logical_sql(&older_resident);
    let error = OwnedJournal::import_snapshot(&older_resident, &extended)
        .err()
        .expect("older resident history must conflict with extension");
    assert_eq!(error.category(), OwnedJournalErrorCategory::ImportConflict);
    assert_eq!(logical_sql(&older_resident), before);

    let newer_resident = fresh_root("snapshot-extension-r2-resident");
    let (writer, _) = OwnedJournal::import_snapshot(&newer_resident, &extended).unwrap();
    drop(writer);
    let before = logical_sql(&newer_resident);
    let error = OwnedJournal::import_snapshot(&newer_resident, &schema1)
        .err()
        .expect("newer resident history must conflict with older input");
    assert_eq!(error.category(), OwnedJournalErrorCategory::ImportConflict);
    assert_eq!(logical_sql(&newer_resident), before);
}

#[test]
fn snapshot_rejects_orphaned_commit_section_rows_and_recovers_after_restore() {
    let expected = embedded_snapshot(SCHEMA1_SNAPSHOT, SCHEMA1_SEAL);
    let root = fresh_root("orphan-roster");
    let (writer, _) = OwnedJournal::import_snapshot(&root, &expected).unwrap();
    drop(writer);

    let connection = rusqlite::Connection::open(root.join("owned.sqlite")).unwrap();
    connection
        .pragma_update(None, "foreign_keys", "OFF")
        .unwrap();
    connection
        .execute(
            "INSERT INTO owned_commit_sections(revision_id,section,length) VALUES('orphan-revision','registry.sources',0)",
            [],
        )
        .unwrap();
    drop(connection);

    let reader = OwnedJournalReader::open(&root).unwrap();
    assert_eq!(
        reader.snapshot().unwrap_err().category(),
        OwnedJournalErrorCategory::Integrity
    );
    drop(reader);

    let connection = rusqlite::Connection::open(root.join("owned.sqlite")).unwrap();
    connection
        .execute(
            "DELETE FROM owned_commit_sections WHERE revision_id='orphan-revision' AND section='registry.sources'",
            [],
        )
        .unwrap();
    drop(connection);

    let reader = OwnedJournalReader::open(&root).unwrap();
    assert_eq!(reader.snapshot().unwrap(), expected);
}

#[test]
fn imported_rich_24_section_snapshot_matches_canonical_input_whole() {
    let expected = rich_owned_snapshot();
    let sealed = seal_owned_snapshot(&expected).unwrap();
    let input_bytes = expected.to_json_bytes().unwrap();
    assert_eq!(sealed.snapshot, input_bytes);
    let input_sha256 = sha256(&input_bytes);
    let root = fresh_root("rich24-import");

    let (writer, report) = OwnedJournal::import_snapshot(&root, &expected).unwrap();
    assert_eq!(report.outcome, OwnedImportOutcome::Imported);
    assert_eq!(report.head, expected.as_data().head);
    drop(writer);

    let reader = OwnedJournalReader::open(&root).unwrap();
    let exported = reader.snapshot().unwrap();
    assert_eq!(exported, expected);
    let exported_bytes = exported.to_json_bytes().unwrap();
    assert_eq!(exported_bytes, input_bytes);
    assert_eq!(sha256(&exported_bytes), input_sha256);
}

fn rich_owned_snapshot() -> OwnedSnapshot {
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
        source: owner.clone(),
        revision: genesis_revision.id.clone(),
    }];
    let dictionary = Dictionary::from_document(document).unwrap();
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
            length: u64::try_from(length).unwrap(),
        })
        .collect();
    let bytes = dictionary.to_json_bytes().unwrap();
    let revision = genesis_revision.id;
    OwnedSnapshot::from_data(OwnedSnapshotData {
        version: 1,
        journal_schema: 1,
        source: owner,
        head: revision.clone(),
        dictionary,
        commits: vec![OwnedSnapshotCommit {
            revision,
            state_sha256: sha256(&bytes),
            prefixes,
        }],
        curation: None,
    })
    .unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn imported_snapshot_reader_pins_busy_writer_then_refreshes_after_explicit_retry() {
    let expected = embedded_snapshot(SCHEMA1_SNAPSHOT, SCHEMA1_SEAL);
    let root = fresh_root("snapshot-busy-retry");
    let (mut writer, _) = OwnedJournal::import_snapshot(&root, &expected).unwrap();
    let reader = OwnedJournalReader::open(&root).unwrap();
    let pinned = reader.snapshot().unwrap();
    assert_eq!(pinned, expected);
    let before_busy = logical_sql(&root);

    let candidate = next_snapshot_revision(&expected.as_data().dictionary);
    let error = writer
        .commit(&expected.as_data().head, &candidate)
        .unwrap_err();
    assert_eq!(error.category(), OwnedJournalErrorCategory::Busy);
    assert_eq!(reader.snapshot().unwrap(), pinned);
    assert_eq!(logical_sql(&root), before_busy);
    drop(reader);

    let committed = writer.commit(&expected.as_data().head, &candidate).unwrap();
    assert_eq!(committed.head.revision.id.as_str(), "owned-r2");
    let refreshed = OwnedJournalReader::open(&root).unwrap().snapshot().unwrap();
    assert_eq!(refreshed.as_data().head.as_str(), "owned-r2");
    assert_eq!(refreshed.as_data().dictionary, candidate);
    assert_eq!(
        &refreshed.as_data().commits[..expected.as_data().commits.len()],
        expected.as_data().commits.as_slice()
    );
    for revision in ["owned-r0", "owned-r1"] {
        let revision = vlajmaji_lexicon::EntityId::new(revision).unwrap();
        assert_eq!(
            refreshed.dictionary_at(&revision).unwrap(),
            expected.dictionary_at(&revision).unwrap()
        );
    }
    assert_ne!(refreshed, pinned);
    assert_eq!(
        OwnedJournalReader::open(&root).unwrap().snapshot().unwrap(),
        refreshed
    );
}

#[test]
fn snapshot_import_rejects_wrong_family_missing_parent_and_symlink_without_replacement() {
    let expected = embedded_snapshot(SCHEMA1_SNAPSHOT, SCHEMA1_SEAL);

    let wrong_family = fresh_root("import-wrong-family");
    fs::create_dir(&wrong_family).unwrap();
    let connection = rusqlite::Connection::open(wrong_family.join("owned.sqlite")).unwrap();
    connection
        .execute_batch("CREATE TABLE owned_meta(singleton INTEGER, family TEXT); INSERT INTO owned_meta VALUES(1, 'foreign-family');")
        .unwrap();
    drop(connection);
    let before = fs::read(wrong_family.join("owned.sqlite")).unwrap();
    assert_eq!(
        OwnedJournal::import_snapshot(&wrong_family, &expected)
            .err()
            .unwrap()
            .category(),
        OwnedJournalErrorCategory::WrongFamily
    );
    assert_eq!(fs::read(wrong_family.join("owned.sqlite")).unwrap(), before);

    let nonjournal = fresh_root("import-nonjournal");
    fs::create_dir(&nonjournal).unwrap();
    let nonjournal_path = nonjournal.join("owned.sqlite");
    fs::write(&nonjournal_path, b"not a SQLite journal").unwrap();
    let before = fs::read(&nonjournal_path).unwrap();
    assert_eq!(
        OwnedJournal::import_snapshot(&nonjournal, &expected)
            .err()
            .unwrap()
            .category(),
        OwnedJournalErrorCategory::Backend
    );
    assert_eq!(fs::read(&nonjournal_path).unwrap(), before);

    let missing_parent = fresh_root("import-missing-parent");
    let absent_parent = missing_parent.join("absent").join("journal");
    assert_eq!(
        OwnedJournal::import_snapshot(&absent_parent, &expected)
            .err()
            .unwrap()
            .category(),
        OwnedJournalErrorCategory::Io
    );
    assert!(!absent_parent.exists());
    assert!(!absent_parent.parent().unwrap().exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let real_root = fresh_root("import-symlink-target");
        let (writer, _) = OwnedJournal::import_snapshot(&real_root, &expected).unwrap();
        drop(writer);
        let real_before = logical_sql(&real_root);
        let link_parent = fresh_root("import-symlink-parent");
        fs::create_dir(&link_parent).unwrap();
        let link = link_parent.join("journal");
        symlink(&real_root, &link).unwrap();
        assert_eq!(
            OwnedJournal::import_snapshot(&link, &expected)
                .err()
                .unwrap()
                .category(),
            OwnedJournalErrorCategory::Io
        );
        assert_eq!(logical_sql(&real_root), real_before);
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }
}
