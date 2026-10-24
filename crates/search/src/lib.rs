//! Source-preserving lexical queries and bounded phonological filters.
//!
//! Captured records, computed analyses, and caller proposals remain separate.
//! Results keep source snapshot identity and exact record locators.
//!
//! # Example
//! ```
//! use vlajmaji_search::{Query, SearchEngine, SearchOptions};
//! # fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let engine = SearchEngine::open("lexicon.sqlite")?;
//! let result = engine.search(Query::Exact { spelling: "farvi".into() }, SearchOptions::default())?;
//! println!("{} matches", result.total_records);
//! # Ok(())
//! # }
//! ```

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    path::Path,
    str::FromStr,
};

use bauske_phonology::{
    DistanceModel, FollowingContext, NucleusKind, Segment, StressChoice, SyllabificationMode,
    WordKind, analyze_word, assign_stress, syllabify,
};
use serde::{Deserialize, Serialize};
use vlajmaji_lexicon::{CapturedRecord, FieldPresence, FormRole, Lexicon, Proposal, SourceFormat};
use vlajmaji_store::{
    EdgeHit, LexicalStore, SearchQuery as NativeQuery, SqliteStore, StoreError, record_id,
    validate_archive,
};

/// Grouped exact queries and stable dictionary reference resolution.
pub mod dictionary;

pub use dictionary::{
    DictionaryErrorReport, DictionaryExactOptions, DictionaryExactResponse, DictionaryGroup,
    DictionaryHit, DictionaryQueryError, DictionaryRequest, DictionarySearchEngine,
    DictionarySourceAvailability, DictionarySourceTotal, DictionaryTotals, ReferenceAssertionView,
    ReferenceEntity, ReferenceResolution, ReferenceStatus, SourceSelectionFailure,
};

/// Query policy identities included in every response.
pub const SPELLING_POLICY: &str = "scalar-levenshtein-v1";
/// Identity of the unchanged project phonology/context policy.
pub const PHONOLOGY_POLICY: &str = "p01-default-canonical-unspecified-v1";
/// Identity of the optional morphology version and settings contract.
pub const COMPONENT_POLICY: &str = "latkerlo-jvotci-2.10.2601-ordered-standard-v1";

/// One bounded query. All top limits are applied after the complete match set is known.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum Query {
    /// Return all records with the byte-exact captured spelling.
    Exact {
        /// Byte-exact captured headword.
        spelling: String,
    },
    /// Find literal substrings in ASCII-lowercased source definitions.
    ReverseDefinition {
        /// Non-empty literal substring, compared after ASCII lowercasing.
        contains: String,
    },
    /// Find an exact source form, optionally by assertion role.
    Form {
        /// Exact form bytes.
        value: String,
        /// Optional exact source assertion role.
        role: Option<FormRole>,
    },
    /// Follow captured source links to the requested maximum depth (one or two).
    Links {
        /// Exact captured start spelling.
        spelling: String,
        /// Maximum traversal depth, one or two.
        depth: u8,
    },
    /// Scan every distinct captured spelling with Unicode-scalar Levenshtein distance.
    SpellingDistance {
        /// Exact spelling used as the distance anchor.
        anchor: String,
        /// Optional inclusive edit-distance bound.
        max_distance: Option<usize>,
    },
    /// Filter source records by resolved syllables, written nuclei, and default primary stress.
    Prosody {
        /// Optional category filter; source records retain their own mapped class.
        kind: Option<String>,
        /// Optional resolved syllable count filter.
        syllables: Option<usize>,
        /// Optional written nucleus count filter.
        nuclei: Option<usize>,
        /// Optional zero-based default primary nucleus filter.
        primary_nucleus: Option<usize>,
    },
    /// Find source records whose nuclei exactly match this sequence and declared source class.
    Nuclei {
        /// Required source-mapped category.
        kind: String,
        /// Exact sequence of nucleus spellings.
        pattern: Vec<String>,
    },
    /// Compare each aligned single-vowel nucleus to an explicit anchor.
    Contrast {
        /// Explicit anchor spelling, analyzed under its declared kind.
        anchor: String,
        /// Declared anchor category; candidate records keep their source mapping.
        kind: String,
        /// Require at least one positive P01 within-vowel mismatch.
        require_mismatch: bool,
    },
    /// Find source spellings whose computed ordered decomposition matches these components.
    Components {
        /// Ordered component sequence to compare with computed analyses.
        components: Vec<String>,
    },
    /// Compose literal source candidates with a minimum written-nucleus filter.
    Compose {
        /// Optional literal source-definition substring candidate query.
        definition_contains: Option<String>,
        /// Optional exact start spelling for depth-one source links.
        links_from: Option<String>,
        /// Optional minimum written-nucleus count.
        min_nuclei: Option<usize>,
        /// Optional category filter over source-derived mappings.
        kind: Option<String>,
    },
    /// Return isolated caller-authored proposals with the exact spelling.
    ProposalExact {
        /// Exact proposal spelling.
        spelling: String,
    },
    /// Generate a computed lujvo with one explicit optional morphology setting.
    Generate {
        /// Ordered source components supplied to the engine.
        components: Vec<String>,
        /// One explicit y-hyphen setting.
        setting: MorphologySetting,
    },
}

/// One of the three bounded Latkerlo y-hyphen settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MorphologySetting {
    /// Use the library default.
    Standard,
    /// Allow y as a replacement for r/n hyphens.
    AllowY,
    /// Require y hyphens.
    ForceY,
}

/// Stable presentation options; limit zero means return no hits and retain the total.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Maximum displayed hits; zero returns the complete total and no hits.
    pub limit: usize,
    /// Include matching proposals alongside source hits for `Query::Exact` only.
    pub include_proposals: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 20,
            include_proposals: false,
        }
    }
}

/// A source-backed row with preserved identity and separate text fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hit {
    /// Stable source-record tuple identity.
    pub record_id: String,
    /// Source snapshot namespace.
    pub namespace: String,
    /// SHA-256 of the source snapshot bytes.
    pub source_sha256: String,
    /// Exact source locator in its snapshot.
    pub locator: String,
    /// Captured source format label.
    pub source_format: String,
    /// Exact captured headword, when present.
    pub spelling: Option<String>,
    /// Source definition text, when present.
    pub definition: Option<String>,
    /// Source gloss text, when present.
    pub gloss: Option<String>,
    /// Source notes text, when present.
    pub notes: Option<String>,
    /// Why the row matched.
    pub reason: String,
    /// Exact source-native word class, if mapped.
    pub source_class: Option<String>,
    /// Caller-facing P01 category from the explicit source-class mapping.
    pub mapped_word_kind: Option<String>,
    /// Stable mapping rule identifier.
    pub mapping_rule: Option<String>,
    /// Full-scan spelling distance, when this was a spelling query.
    pub spelling_distance: Option<usize>,
    /// Selected written nuclei from P01.
    pub nuclei: Vec<Nucleus>,
    /// Syllable count only when conventional grouping resolves.
    pub syllable_count: Option<usize>,
    /// Default primary nucleus; None is explained by stress_status.
    pub primary_nucleus: Option<usize>,
    /// Assigned, known-unassigned, or error state for default stress.
    pub stress_status: Option<String>,
    /// Named unresolved grouping/analysis limitations.
    pub diagnostics: Vec<String>,
    /// Per-position P01 explanations; no whole-word total is provided.
    pub distance_positions: Vec<DistancePosition>,
    /// Exact ordered component sequence returned by the morphology engine.
    pub computed_components: Option<Vec<String>>,
}

/// One nucleus fact from the unchanged P01 implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nucleus {
    /// Exact spelling of this written nucleus.
    pub spelling: String,
    /// Vowel, diphthong, or selected syllabic-consonant kind.
    pub kind: String,
    /// Whether this nucleus is eligible for stress under P01.
    pub stress_countable: bool,
}

/// One explained aligned within-vowel comparison; position scores are not summed across a word.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistancePosition {
    /// Zero-based written nucleus position.
    pub index: usize,
    /// Anchor vowel spelling.
    pub anchor: String,
    /// Candidate vowel spelling.
    pub candidate: String,
    /// P01 categorical mismatch cost at this position.
    pub total: u32,
    /// Pronunciation profile identity.
    pub profile_id: String,
    /// Acquired source identity.
    pub source_id: String,
    /// Per-coordinate values and costs.
    pub contributions: Vec<DistanceContribution>,
}

/// One P01 feature-coordinate explanation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistanceContribution {
    /// Stable feature dimension name.
    pub dimension: String,
    /// Left representative value.
    pub left: String,
    /// Right representative value.
    pub right: String,
    /// Weighted mismatch cost.
    pub cost: u32,
}

/// A computed morphology result, separate from captured lexicon membership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputedValue {
    /// Ordered components passed to Latkerlo.
    pub components: Vec<String>,
    /// Selected engine setting.
    pub setting: MorphologySetting,
    /// Computed spelling.
    pub spelling: String,
    /// Engine score, not a source fact or poetic metric.
    pub engine_score: i32,
    /// Engine-provided component index spans.
    pub component_indices: Vec<[usize; 2]>,
    /// Name of the exact pinned score-producing API.
    pub score_function: String,
    /// Library and policy identity.
    pub engine_id: String,
}

/// One traversed source edge with its exact origin and target explanation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkExplanation {
    /// Stable source-record ID of the edge origin.
    pub origin_id: String,
    /// Stable target ID when the reference resolves in the captured corpus.
    pub target_id: Option<String>,
    /// Exact target spelling in the source clause.
    pub target_spelling: String,
    /// Inclusive UTF-8 byte start in decoded notes.
    pub byte_start: usize,
    /// Exclusive UTF-8 byte end in decoded notes.
    pub byte_end: usize,
    /// Bounded source-reference parser rule ID.
    pub rule_version: String,
}

/// Complete match census followed by a stable bounded display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResponse {
    /// SHA-256 of the validated canonical archive.
    pub source_archive_sha256: String,
    /// Query and computation policy identifiers.
    pub policy_ids: Vec<String>,
    /// Complete match count before limiting.
    pub total_records: usize,
    /// Number of captured input records in the archive.
    pub corpus_records: usize,
    /// Number of distinct exact captured headwords in the archive.
    pub corpus_distinct_spellings: usize,
    /// Number of distinct exact spellings before limiting.
    pub distinct_spellings: usize,
    /// Applied display limit, including zero.
    pub limit: usize,
    /// Deterministically ordered selected hits.
    pub hits: Vec<Hit>,
    /// Exact source-edge explanations for link queries.
    pub links: Vec<LinkExplanation>,
    /// Full-scan exclusion or diagnostic counts.
    pub exclusions: BTreeMap<String, usize>,
    /// Native storage query path when used.
    pub native_path: Option<String>,
    /// Native fallback reason when used.
    pub native_fallback_reason: Option<String>,
    /// Backend candidate counts, labelled by the native stage that produced them.
    pub native_candidate_counts: Vec<NativeCandidateCount>,
    /// Computed values, kept separate from source hits.
    pub computed: Vec<ComputedValue>,
}

/// One backend candidate count with its unambiguous native-query stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeCandidateCount {
    /// Stable stage label (for example `definition_contains` or `links_from`).
    pub stage: String,
    /// Candidate rows considered by that backend execution.
    pub candidates: usize,
}

/// Errors are explicit; invalid or unsupported queries are never empty success.
#[derive(Debug)]
#[non_exhaustive]
pub enum SearchError {
    /// Failure opening, validating, or querying SQLite.
    Store(StoreError),
    /// Invalid caller parameter or unsupported query combination.
    InvalidQuery(String),
    /// The requested optional capability was compiled out.
    FeatureDisabled(&'static str),
    /// Native query identity could not be hydrated from the archive.
    Integrity(String),
    /// Source-grounded phonology or guarded morphology failure.
    Phonology(String),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(error) => write!(f, "{error}"),
            Self::InvalidQuery(message) => write!(f, "invalid query: {message}"),
            Self::FeatureDisabled(feature) => write!(f, "feature disabled: {feature}"),
            Self::Integrity(message) => write!(f, "integrity error: {message}"),
            Self::Phonology(message) => write!(f, "phonology analysis failed: {message}"),
        }
    }
}
impl Error for SearchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            _ => None,
        }
    }
}
impl From<StoreError> for SearchError {
    fn from(value: StoreError) -> Self {
        Self::Store(value)
    }
}

/// Reusable query engine over one validated, caller-owned SQLite archive.
pub struct SearchEngine {
    store: SqliteStore,
    lexicon: Lexicon,
    record_index: BTreeMap<String, usize>,
    archive_sha256: String,
}

impl fmt::Debug for SearchEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SearchEngine")
            .field("corpus_records", &self.lexicon.records().len())
            .field("snapshots", &self.lexicon.snapshots().len())
            .field("archive_sha256", &self.archive_sha256)
            .finish_non_exhaustive()
    }
}

type HydratedNative = (BTreeSet<String>, Option<String>, Option<String>, usize);

#[derive(Clone)]
struct AnalysisEvidence {
    kind: WordKind,
    nuclei: Vec<Nucleus>,
    syllables: Option<usize>,
    primary: Option<usize>,
    stress_status: String,
    diagnostics: Vec<String>,
}

impl SearchEngine {
    /// Opens a SQLite store and validates its canonical source-preserving archive.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SearchError> {
        let store = SqliteStore::open(path)?;
        let archive = store.archive_bytes()?;
        let lexicon = validate_archive(&archive)?;
        let record_index = make_record_index(&lexicon);
        let archive_sha256 = vlajmaji_store::archive_sha256(&archive);
        Ok(Self {
            store,
            lexicon,
            record_index,
            archive_sha256,
        })
    }

    /// Builds a fresh SQLite store from validated canonical archive bytes.
    pub fn build(path: impl AsRef<Path>, archive: &[u8]) -> Result<Self, SearchError> {
        let lexicon = validate_archive(archive)?;
        let store = SqliteStore::create(path, archive)?;
        let record_index = make_record_index(&lexicon);
        let archive_sha256 = vlajmaji_store::archive_sha256(archive);
        Ok(Self {
            store,
            lexicon,
            record_index,
            archive_sha256,
        })
    }

    /// Returns the validated corpus record and snapshot counts.
    #[must_use]
    pub fn corpus_counts(&self) -> (usize, usize) {
        (self.lexicon.records().len(), self.lexicon.snapshots().len())
    }

    /// Executes a complete query, computes totals before limit, and keeps alternatives.
    pub fn search(
        &self,
        query: Query,
        options: SearchOptions,
    ) -> Result<SearchResponse, SearchError> {
        if options.include_proposals
            && !matches!(query, Query::Exact { .. } | Query::ProposalExact { .. })
        {
            return Err(SearchError::InvalidQuery(
                "include_proposals is supported only for exact-spelling queries".into(),
            ));
        }
        let mut exclusions = BTreeMap::new();
        let mut links = Vec::new();
        let mut native_path = None;
        let mut native_fallback_reason = None;
        let mut native_candidate_counts = Vec::new();
        let mut extra = BTreeMap::<String, AnalysisEvidence>::new();
        let mut spelling_distances = BTreeMap::<String, usize>::new();
        let mut distance_positions = BTreeMap::<String, Vec<DistancePosition>>::new();
        #[allow(unused_mut)]
        let mut component_sequences = BTreeMap::<String, Vec<String>>::new();
        #[cfg(feature = "morphology")]
        let mut component_cache = BTreeMap::<String, Result<Vec<String>, String>>::new();
        #[allow(unused_mut)]
        let mut computed = Vec::<ComputedValue>::new();
        let mut proposal_hits = Vec::<Hit>::new();
        let mut ids = BTreeSet::<String>::new();
        let mut policy_ids = vec!["source-preserving-v1".to_owned()];

        match &query {
            Query::Exact { spelling } => {
                let result = self
                    .store
                    .search(&NativeQuery::ExactSpelling(spelling.clone()))?;
                let (found, path, fallback, candidates) = self.hydrate_native(result)?;
                ids = found;
                native_path = path;
                native_fallback_reason = fallback;
                native_candidate_counts.push(NativeCandidateCount {
                    stage: "exact_spelling".into(),
                    candidates,
                });
                if options.include_proposals {
                    proposal_hits.extend(
                        self.lexicon
                            .lookup_proposals(spelling)
                            .into_iter()
                            .map(proposal_hit),
                    );
                    policy_ids.push("proposal-namespace-separated-v1".into());
                }
            }
            Query::ReverseDefinition { contains } => {
                if contains.is_empty() {
                    return Err(SearchError::InvalidQuery(
                        "reverse definition term cannot be empty".into(),
                    ));
                }
                let result = self
                    .store
                    .search(&NativeQuery::DefinitionContains(contains.clone()))?;
                let (found, path, fallback, candidates) = self.hydrate_native(result)?;
                ids = found;
                native_path = path;
                native_fallback_reason = fallback;
                native_candidate_counts.push(NativeCandidateCount {
                    stage: "definition_contains".into(),
                    candidates,
                });
            }
            Query::Form { value, role } => {
                let result = self.store.search(&NativeQuery::Form {
                    value: value.clone(),
                    role: *role,
                })?;
                let (found, path, fallback, candidates) = self.hydrate_native(result)?;
                ids = found;
                native_path = path;
                native_fallback_reason = fallback;
                native_candidate_counts.push(NativeCandidateCount {
                    stage: "form".into(),
                    candidates,
                });
            }
            Query::Links { spelling, depth } => {
                if !(1..=2).contains(depth) {
                    return Err(SearchError::InvalidQuery(
                        "link depth must be 1 or 2".into(),
                    ));
                }
                let result = self.store.search(&NativeQuery::CrossReferences {
                    spelling: spelling.clone(),
                    depth: *depth,
                })?;
                let (found, path, fallback, candidates) = self.hydrate_native(result.clone())?;
                ids = found;
                native_path = path;
                native_fallback_reason = fallback;
                native_candidate_counts.push(NativeCandidateCount {
                    stage: "cross_references".into(),
                    candidates,
                });
                links = result.edges.into_iter().map(link_explanation).collect();
            }
            Query::ProposalExact { spelling } => {
                for proposal in self.lexicon.lookup_proposals(spelling) {
                    proposal_hits.push(proposal_hit(proposal));
                }
                policy_ids.push("proposal-namespace-separated-v1".into());
            }
            Query::Generate {
                components,
                setting,
            } => {
                #[cfg(feature = "morphology")]
                {
                    computed.push(guarded_generate(components, *setting)?);
                    policy_ids.push(COMPONENT_POLICY.into());
                }
                #[cfg(not(feature = "morphology"))]
                {
                    let _ = (components, setting);
                    return Err(SearchError::FeatureDisabled("morphology"));
                }
            }
            Query::SpellingDistance {
                anchor,
                max_distance,
            } => {
                if anchor.is_empty() {
                    return Err(SearchError::InvalidQuery(
                        "spelling anchor cannot be empty".into(),
                    ));
                }
                policy_ids.push(SPELLING_POLICY.into());
                let mut distance_cache = BTreeMap::<String, usize>::new();
                for record in self.lexicon.records() {
                    let Some(spelling) = record.word() else {
                        *exclusions.entry("no-headword".into()).or_insert(0) += 1;
                        continue;
                    };
                    let distance = *distance_cache
                        .entry(spelling.to_owned())
                        .or_insert_with(|| strsim::levenshtein(anchor, spelling));
                    if max_distance.is_none_or(|bound| distance <= bound) {
                        let id = record_id(record.identity());
                        ids.insert(id.clone());
                        spelling_distances.insert(id, distance);
                    }
                }
            }
            Query::Prosody {
                kind,
                syllables,
                nuclei,
                primary_nucleus,
            } => {
                policy_ids.push(PHONOLOGY_POLICY.into());
                let requested_kind = kind.as_deref().map(parse_kind).transpose()?;
                for record in self.lexicon.records() {
                    let Some(spelling) = record.word() else {
                        *exclusions.entry("no-headword".into()).or_insert(0) += 1;
                        continue;
                    };
                    let Some((word_kind, _, _)) = source_kind(record) else {
                        *exclusions
                            .entry("unmapped-source-class".into())
                            .or_insert(0) += 1;
                        continue;
                    };
                    if requested_kind.is_some_and(|expected| expected != word_kind) {
                        *exclusions
                            .entry("requested-kind-filter".into())
                            .or_insert(0) += 1;
                        continue;
                    }
                    match analyze(spelling, word_kind) {
                        Ok(evidence) => {
                            if syllables.is_some() && evidence.syllables.is_none() {
                                *exclusions
                                    .entry("unresolved-syllable-grouping".into())
                                    .or_insert(0) += 1;
                                continue;
                            }
                            let syllable_ok =
                                syllables.is_none_or(|wanted| evidence.syllables == Some(wanted));
                            let nuclei_ok =
                                nuclei.is_none_or(|wanted| evidence.nuclei.len() == wanted);
                            let stress_ok = primary_nucleus
                                .is_none_or(|wanted| evidence.primary == Some(wanted));
                            if syllable_ok && nuclei_ok && stress_ok {
                                let id = record_id(record.identity());
                                ids.insert(id.clone());
                                extra.insert(id, evidence);
                            } else {
                                *exclusions.entry("filter-mismatch".into()).or_insert(0) += 1;
                            }
                        }
                        Err(_) => {
                            *exclusions.entry("analysis-error".into()).or_insert(0) += 1;
                        }
                    }
                }
            }
            Query::Nuclei { kind, pattern } => {
                policy_ids.push(PHONOLOGY_POLICY.into());
                let word_kind = parse_kind(kind)?;
                for record in self.lexicon.records() {
                    let Some(spelling) = record.word() else {
                        *exclusions.entry("no-headword".into()).or_insert(0) += 1;
                        continue;
                    };
                    let Some((mapped_kind, _, _)) = source_kind(record) else {
                        *exclusions
                            .entry("unmapped-source-class".into())
                            .or_insert(0) += 1;
                        continue;
                    };
                    if mapped_kind != word_kind {
                        *exclusions
                            .entry("requested-kind-filter".into())
                            .or_insert(0) += 1;
                        continue;
                    }
                    match analyze(spelling, mapped_kind) {
                        Ok(evidence)
                            if evidence
                                .nuclei
                                .iter()
                                .map(|n| n.spelling.as_str())
                                .eq(pattern.iter().map(String::as_str)) =>
                        {
                            let id = record_id(record.identity());
                            ids.insert(id.clone());
                            extra.insert(id, evidence);
                        }
                        Ok(_) => {
                            *exclusions
                                .entry("nucleus-pattern-mismatch".into())
                                .or_insert(0) += 1;
                        }
                        Err(_) => {
                            *exclusions.entry("analysis-error".into()).or_insert(0) += 1;
                        }
                    }
                }
            }
            Query::Contrast {
                anchor,
                kind,
                require_mismatch,
            } => {
                policy_ids.push(PHONOLOGY_POLICY.into());
                let word_kind = parse_kind(kind)?;
                let anchor_evidence = analyze(anchor, word_kind).map_err(SearchError::Phonology)?;
                if anchor_evidence.nuclei.iter().any(|n| n.kind != "vowel") {
                    return Err(SearchError::InvalidQuery(
                        "aligned contrast requires single-vowel nuclei".into(),
                    ));
                }
                let anchors: Vec<_> = anchor_evidence
                    .nuclei
                    .iter()
                    .map(|n| n.spelling.clone())
                    .collect();
                for record in self.lexicon.records() {
                    let Some(spelling) = record.word() else {
                        *exclusions.entry("no-headword".into()).or_insert(0) += 1;
                        continue;
                    };
                    let Some((candidate_kind, _, _)) = source_kind(record) else {
                        *exclusions
                            .entry("unmapped-source-class".into())
                            .or_insert(0) += 1;
                        continue;
                    };
                    if candidate_kind != word_kind {
                        *exclusions
                            .entry("requested-kind-filter".into())
                            .or_insert(0) += 1;
                        continue;
                    }
                    match analyze(spelling, candidate_kind) {
                        Ok(evidence)
                            if evidence.nuclei.len() == anchors.len()
                                && evidence.nuclei.iter().all(|n| n.kind == "vowel") =>
                        {
                            let mut positions = Vec::new();
                            let mut has_mismatch = false;
                            for (index, (left, right)) in
                                anchors.iter().zip(&evidence.nuclei).enumerate()
                            {
                                let left_segment = Segment::from_str(left)
                                    .map_err(|e| SearchError::Phonology(e.to_string()))?;
                                let right_segment = Segment::from_str(&right.spelling)
                                    .map_err(|e| SearchError::Phonology(e.to_string()))?;
                                let explanation = DistanceModel::default()
                                    .compare(left_segment, right_segment)
                                    .map_err(|e| SearchError::Phonology(e.to_string()))?;
                                has_mismatch |= explanation.total() > 0;
                                positions.push(DistancePosition {
                                    index,
                                    anchor: left.clone(),
                                    candidate: right.spelling.clone(),
                                    total: explanation.total(),
                                    profile_id: explanation.profile_id().into(),
                                    source_id: explanation.source_id().into(),
                                    contributions: explanation
                                        .contributions()
                                        .iter()
                                        .map(|contribution| DistanceContribution {
                                            dimension: contribution.dimension().label().into(),
                                            left: contribution.left().label().into(),
                                            right: contribution.right().label().into(),
                                            cost: contribution.cost(),
                                        })
                                        .collect(),
                                });
                            }
                            if !require_mismatch || has_mismatch {
                                let id = record_id(record.identity());
                                ids.insert(id.clone());
                                extra.insert(id.clone(), evidence);
                                distance_positions.insert(id, positions);
                            } else {
                                *exclusions
                                    .entry("no-within-vowel-mismatch".into())
                                    .or_insert(0) += 1;
                            }
                        }
                        Ok(_) => {
                            *exclusions
                                .entry("unaligned-or-non-vowel-nucleus".into())
                                .or_insert(0) += 1;
                        }
                        Err(_) => {
                            *exclusions.entry("analysis-error".into()).or_insert(0) += 1;
                        }
                    }
                }
                policy_ids.push("aligned-single-vowel-contrast-v1".into());
            }
            Query::Components { components } => {
                #[cfg(feature = "morphology")]
                {
                    if components.is_empty() || components.iter().any(|s| !valid_component(s)) {
                        return Err(SearchError::InvalidQuery(
                            "components must be non-empty supported ASCII tokens".into(),
                        ));
                    }
                    policy_ids.push(COMPONENT_POLICY.into());
                    for record in self.lexicon.records() {
                        let Some(spelling) = record.word() else {
                            *exclusions.entry("no-headword".into()).or_insert(0) += 1;
                            continue;
                        };
                        let returned = component_cache
                            .entry(spelling.to_owned())
                            .or_insert_with(|| guarded_veljvo(spelling));
                        match returned {
                            Ok(parts) if ordered_subsequence(parts, components) => {
                                let id = record_id(record.identity());
                                ids.insert(id.clone());
                                component_sequences.insert(id, parts.clone());
                            }
                            Ok(_) => {
                                *exclusions
                                    .entry("component-sequence-mismatch".into())
                                    .or_insert(0) += 1;
                            }
                            Err(reason) => {
                                *exclusions.entry(reason.clone()).or_insert(0) += 1;
                            }
                        }
                    }
                }
                #[cfg(not(feature = "morphology"))]
                {
                    let _ = components;
                    return Err(SearchError::FeatureDisabled("morphology"));
                }
            }
            Query::Compose {
                definition_contains,
                links_from,
                min_nuclei,
                kind,
            } => {
                policy_ids.push(PHONOLOGY_POLICY.into());
                let requested_kind = kind.as_deref().map(parse_kind).transpose()?;
                if let Some(term) = definition_contains {
                    if term.is_empty() {
                        return Err(SearchError::InvalidQuery(
                            "composition definition term cannot be empty".into(),
                        ));
                    }
                    let result = self
                        .store
                        .search(&NativeQuery::DefinitionContains(term.clone()))?;
                    let (found, path, fallback, candidates) = self.hydrate_native(result)?;
                    ids = found;
                    native_path = path;
                    native_fallback_reason = fallback;
                    native_candidate_counts.push(NativeCandidateCount {
                        stage: "definition_contains".into(),
                        candidates,
                    });
                }
                if let Some(spelling) = links_from {
                    let result = self.store.search(&NativeQuery::CrossReferences {
                        spelling: spelling.clone(),
                        depth: 1,
                    })?;
                    let (link_ids, path, fallback, candidates) =
                        self.hydrate_native(result.clone())?;
                    native_candidate_counts.push(NativeCandidateCount {
                        stage: "links_from".into(),
                        candidates,
                    });
                    if definition_contains.is_some() {
                        ids = ids.intersection(&link_ids).cloned().collect();
                    } else {
                        ids = link_ids;
                    }
                    links = result.edges.into_iter().map(link_explanation).collect();
                    native_path = path;
                    native_fallback_reason = fallback;
                }
                if definition_contains.is_none() && links_from.is_none() {
                    return Err(SearchError::InvalidQuery(
                        "composition requires a literal definition term or source-link start"
                            .into(),
                    ));
                }
                for id in ids.clone() {
                    let record = self.record_for_id(&id)?;
                    let Some(spelling) = record.word() else {
                        ids.remove(&id);
                        *exclusions.entry("no-headword".into()).or_insert(0) += 1;
                        continue;
                    };
                    let Some((word_kind, _, _)) = source_kind(record) else {
                        ids.remove(&id);
                        *exclusions
                            .entry("unmapped-source-class".into())
                            .or_insert(0) += 1;
                        continue;
                    };
                    if requested_kind.is_some_and(|expected| expected != word_kind) {
                        ids.remove(&id);
                        *exclusions
                            .entry("requested-kind-filter".into())
                            .or_insert(0) += 1;
                        continue;
                    }
                    match analyze(spelling, word_kind) {
                        Ok(evidence) if min_nuclei.is_none_or(|n| evidence.nuclei.len() >= n) => {
                            extra.insert(id, evidence);
                        }
                        Ok(_) => {
                            ids.remove(&id);
                            *exclusions.entry("nuclei-filter".into()).or_insert(0) += 1;
                        }
                        Err(_) => {
                            ids.remove(&id);
                            *exclusions.entry("analysis-error".into()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        let mut hits: Vec<Hit> = ids
            .iter()
            .map(|id| {
                let record = self.record_for_id(id)?;
                let mut hit = hit_from_record(record, id, reason_for(&query));
                if let Some(evidence) = extra.get(id) {
                    hit.mapped_word_kind = Some(evidence.kind.label().to_owned());
                    hit.nuclei = evidence.nuclei.clone();
                    hit.syllable_count = evidence.syllables;
                    hit.primary_nucleus = evidence.primary;
                    hit.stress_status = Some(evidence.stress_status.clone());
                    hit.diagnostics = evidence.diagnostics.clone();
                }
                hit.spelling_distance = spelling_distances.get(id).copied();
                hit.distance_positions = distance_positions.get(id).cloned().unwrap_or_default();
                hit.computed_components = component_sequences.get(id).cloned();
                hit.source_class = source_native_class(record);
                if let Some((kind, rule, _native_class)) = source_kind(record) {
                    hit.mapping_rule = Some(rule);
                    if hit.mapped_word_kind.is_none() {
                        hit.mapped_word_kind = Some(kind.label().to_owned());
                    }
                }
                Ok(hit)
            })
            .collect::<Result<_, SearchError>>()?;
        hits.sort_by(|a, b| {
            a.spelling
                .as_deref()
                .unwrap_or("")
                .as_bytes()
                .cmp(b.spelling.as_deref().unwrap_or("").as_bytes())
                .then(a.record_id.cmp(&b.record_id))
        });
        if matches!(query, Query::SpellingDistance { .. }) {
            hits.sort_by(|a, b| {
                let anchor = match &query {
                    Query::SpellingDistance { anchor, .. } => anchor.as_str(),
                    _ => "",
                };
                strsim::levenshtein(anchor, a.spelling.as_deref().unwrap_or(""))
                    .cmp(&strsim::levenshtein(
                        anchor,
                        b.spelling.as_deref().unwrap_or(""),
                    ))
                    .then(
                        a.spelling
                            .as_deref()
                            .unwrap_or("")
                            .as_bytes()
                            .cmp(b.spelling.as_deref().unwrap_or("").as_bytes()),
                    )
                    .then(a.record_id.cmp(&b.record_id))
            });
        }
        let show_proposals =
            options.include_proposals || matches!(query, Query::ProposalExact { .. });
        let total_records = hits.len()
            + if show_proposals {
                proposal_hits.len()
            } else {
                0
            };
        let mut spellings = hits
            .iter()
            .filter_map(|hit| hit.spelling.as_deref())
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();
        if show_proposals {
            spellings.extend(proposal_hits.iter().filter_map(|h| h.spelling.clone()));
        }
        hits.extend(if show_proposals {
            proposal_hits
        } else {
            Vec::new()
        });
        if show_proposals {
            hits.sort_by(|a, b| {
                a.spelling
                    .as_deref()
                    .unwrap_or("")
                    .as_bytes()
                    .cmp(b.spelling.as_deref().unwrap_or("").as_bytes())
                    .then_with(|| {
                        (a.source_format == "Proposal").cmp(&(b.source_format == "Proposal"))
                    })
                    .then(a.record_id.cmp(&b.record_id))
            });
        }
        hits.truncate(options.limit);
        Ok(SearchResponse {
            source_archive_sha256: self.archive_sha256.clone(),
            policy_ids,
            total_records,
            corpus_records: self.lexicon.records().len(),
            corpus_distinct_spellings: self
                .lexicon
                .records()
                .iter()
                .filter_map(|record| record.word())
                .map(str::to_owned)
                .collect::<BTreeSet<_>>()
                .len(),
            distinct_spellings: spellings.len(),
            limit: options.limit,
            hits,
            links,
            exclusions,
            native_path,
            native_fallback_reason,
            native_candidate_counts,
            computed,
        })
    }

    fn hydrate_native(
        &self,
        result: vlajmaji_store::SearchResult,
    ) -> Result<HydratedNative, SearchError> {
        for id in &result.record_ids {
            self.record_for_id(id)?;
        }
        Ok((
            result.record_ids,
            Some(result.path),
            result.fallback_reason,
            result.candidates,
        ))
    }

    fn record_for_id(&self, id: &str) -> Result<&CapturedRecord, SearchError> {
        self.record_index
            .get(id)
            .and_then(|index| self.lexicon.records().get(*index))
            .ok_or_else(|| {
                SearchError::Integrity(format!(
                    "native record id {id} is absent from validated archive"
                ))
            })
    }
}

fn make_record_index(lexicon: &Lexicon) -> BTreeMap<String, usize> {
    lexicon
        .records()
        .iter()
        .enumerate()
        .map(|(index, record)| (record_id(record.identity()), index))
        .collect()
}

fn reason_for(query: &Query) -> String {
    match query {
        Query::Exact { .. } => "exact captured spelling".into(),
        Query::ReverseDefinition { .. } => "literal source-definition substring".into(),
        Query::Form { .. } => "exact source form assertion".into(),
        Query::Links { .. } => "captured source cross-reference traversal".into(),
        Query::SpellingDistance { .. } => "full-scan Unicode scalar spelling distance".into(),
        Query::Prosody { .. } | Query::Nuclei { .. } => {
            "P01 phonology analysis under declared mapping".into()
        }
        Query::Contrast { .. } => "aligned single-vowel positions only".into(),
        Query::Components { .. } => "optional computed morphology analysis".into(),
        Query::Compose { .. } => "literal source candidate plus phonology filter".into(),
        Query::ProposalExact { .. } => "separate caller proposal namespace".into(),
        Query::Generate { .. } => "computed morphology output; not source attestation".into(),
    }
}

fn hit_from_record(record: &CapturedRecord, id: &str, reason: String) -> Hit {
    Hit {
        record_id: id.into(),
        namespace: record.identity().snapshot().namespace().into(),
        source_sha256: record.identity().snapshot().sha256().into(),
        locator: record.identity().locator().into(),
        source_format: format!("{:?}", record.format()),
        spelling: record.word().map(str::to_owned),
        definition: record.definition().map(str::to_owned),
        gloss: record.gloss().map(str::to_owned),
        notes: record.notes().map(str::to_owned),
        reason,
        source_class: None,
        mapped_word_kind: None,
        mapping_rule: None,
        spelling_distance: None,
        nuclei: Vec::new(),
        syllable_count: None,
        primary_nucleus: None,
        stress_status: None,
        diagnostics: Vec::new(),
        distance_positions: Vec::new(),
        computed_components: None,
    }
}

fn proposal_hit(proposal: &Proposal) -> Hit {
    Hit {
        record_id: proposal.id().into(),
        namespace: proposal.namespace().into(),
        source_sha256: String::new(),
        locator: "proposal".into(),
        source_format: "Proposal".into(),
        spelling: Some(proposal.spelling().into()),
        definition: None,
        gloss: None,
        notes: Some(proposal.text().into()),
        reason: "caller-authored proposal; not captured source".into(),
        source_class: None,
        mapped_word_kind: None,
        mapping_rule: None,
        spelling_distance: None,
        nuclei: Vec::new(),
        syllable_count: None,
        primary_nucleus: None,
        stress_status: None,
        diagnostics: Vec::new(),
        distance_positions: Vec::new(),
        computed_components: None,
    }
}

fn link_explanation(edge: EdgeHit) -> LinkExplanation {
    LinkExplanation {
        origin_id: edge.origin_id,
        target_id: edge.target_id,
        target_spelling: edge.target_spelling,
        byte_start: edge.byte_start,
        byte_end: edge.byte_end,
        rule_version: edge.rule_version,
    }
}

fn source_kind(record: &CapturedRecord) -> Option<(WordKind, String, String)> {
    let raw = source_native_class(record)?;
    let (kind, mapping) = match raw.as_str() {
        "gismu" => (WordKind::Brivla, "gismu->brivla"),
        "experimental gismu" => (WordKind::Brivla, "experimental-gismu->brivla"),
        "lujvo" => (WordKind::GismuBasedLujvo, "lujvo->gismu-based-lujvo"),
        "fu'ivla" => (WordKind::Zihevla, "fuivla->zihevla"),
        "obsolete fu'ivla" => (WordKind::Zihevla, "obsolete-fuivla->zihevla"),
        "cmavo" => (WordKind::Cmavo, "cmavo->cmavo"),
        "experimental cmavo" => (WordKind::Cmavo, "experimental-cmavo->cmavo"),
        "obsolete cmavo" => (WordKind::Cmavo, "obsolete-cmavo->cmavo"),
        "cmevla" => (WordKind::Cmevla, "cmevla->cmevla"),
        "obsolete cmevla" => (WordKind::Cmevla, "obsolete-cmevla->cmevla"),
        _ => return None,
    };
    let field_rule = match record.format() {
        SourceFormat::LaLojbanJson => "la-word_type",
        SourceFormat::LensiskuJson => "lensisku-type_name",
        SourceFormat::LlgGismuFixed => "llg-gismu-fixed",
        SourceFormat::LlgCmavoFixed => "llg-cmavo-fixed",
        SourceFormat::LlgRafsiRows => return None,
        _ => return None,
    };
    Some((kind, format!("source-class-v1/{field_rule}/{mapping}"), raw))
}

fn source_native_class(record: &CapturedRecord) -> Option<String> {
    match record.format() {
        SourceFormat::LaLojbanJson => json_string_field(record, "word_type"),
        SourceFormat::LensiskuJson => json_string_field(record, "type_name"),
        SourceFormat::LlgGismuFixed => Some("gismu".into()),
        SourceFormat::LlgCmavoFixed => Some("cmavo".into()),
        SourceFormat::LlgRafsiRows => None,
        _ => None,
    }
}

fn json_string_field(record: &CapturedRecord, name: &str) -> Option<String> {
    let FieldPresence::Value(raw) = record.field(name) else {
        return None;
    };
    Some(serde_json::from_str(raw).unwrap_or_else(|_| raw.to_owned()))
}

fn parse_kind(value: &str) -> Result<WordKind, SearchError> {
    match value {
        "brivla" => Ok(WordKind::Brivla),
        "cmavo" => Ok(WordKind::Cmavo),
        "cmevla" => Ok(WordKind::Cmevla),
        "zihevla" => Ok(WordKind::Zihevla),
        "gismu-lujvo" => Ok(WordKind::GismuBasedLujvo),
        _ => Err(SearchError::InvalidQuery(format!(
            "unknown declared kind {value:?}"
        ))),
    }
}

fn analyze(spelling: &str, kind: WordKind) -> Result<AnalysisEvidence, String> {
    let analysis = analyze_word(spelling, kind, &[]).map_err(|e| e.to_string())?;
    let syllables = syllabify(&analysis, SyllabificationMode::Canonical);
    let count = syllables
        .is_resolved()
        .then_some(syllables.syllables().len());
    let (primary, stress_status) = match assign_stress(
        &analysis,
        StressChoice::Default,
        FollowingContext::Unspecified,
    ) {
        Ok(stress) if stress.primary_nucleus().is_some() => {
            (stress.primary_nucleus(), "assigned-primary".to_owned())
        }
        Ok(_) => (None, "known-no-primary".to_owned()),
        Err(error) => (None, format!("stress-error:{error}")),
    };
    let mut diagnostics: Vec<String> = analysis
        .diagnostics()
        .iter()
        .map(|d| format!("{:?}", d.kind()))
        .collect();
    diagnostics.extend(
        syllables
            .limitations()
            .iter()
            .map(|limitation| format!("syllabification:{limitation:?}")),
    );
    let nuclei = analysis
        .nuclei()
        .iter()
        .map(|n| {
            let (kind, countable) = match n.kind() {
                NucleusKind::Vowel(_) => ("vowel", n.stress_countable()),
                NucleusKind::Diphthong(_) => ("diphthong", n.stress_countable()),
                NucleusKind::SyllabicConsonant(_) => ("syllabic-consonant", n.stress_countable()),
                _ => ("unknown", n.stress_countable()),
            };
            Nucleus {
                spelling: n.spelling().into(),
                kind: kind.into(),
                stress_countable: countable,
            }
        })
        .collect();
    Ok(AnalysisEvidence {
        kind,
        nuclei,
        syllables: count,
        primary,
        stress_status,
        diagnostics,
    })
}

#[cfg(feature = "morphology")]
fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value.is_ascii()
        && value.bytes().all(|b| b.is_ascii_alphabetic() || b == b'\'')
}

#[cfg(feature = "morphology")]
fn guarded_generate(
    components: &[String],
    setting: MorphologySetting,
) -> Result<ComputedValue, SearchError> {
    if components.len() < 2
        || components
            .iter()
            .any(|component| !valid_component(component))
    {
        return Err(SearchError::InvalidQuery(
            "generation requires at least two non-empty supported ASCII components".into(),
        ));
    }
    use latkerlo_jvotci::YHyphenSetting;
    let settings = latkerlo_jvotci::Settings {
        y_hyphens: match setting {
            MorphologySetting::Standard => YHyphenSetting::Standard,
            MorphologySetting::AllowY => YHyphenSetting::AllowY,
            MorphologySetting::ForceY => YHyphenSetting::ForceY,
        },
        ..latkerlo_jvotci::Settings::default()
    };
    let tanru = components.join(" ");
    let result =
        std::panic::catch_unwind(|| latkerlo_jvotci::get_lujvo_with_analytics(&tanru, &settings));
    match result {
        Ok(Ok((spelling, score, indices))) => Ok(ComputedValue {
            components: components.to_vec(),
            setting,
            spelling,
            engine_score: score,
            component_indices: indices,
            score_function: "latkerlo_jvotci::get_lujvo_with_analytics".into(),
            engine_id: format!("latkerlo-jvotci-2.10.2601/{setting:?}"),
        }),
        Ok(Err(error)) => Err(SearchError::Phonology(format!(
            "morphology generation failed: {error}"
        ))),
        Err(_) => Err(SearchError::Phonology(
            "morphology generation panic caught (panic=unwind only)".into(),
        )),
    }
}

#[cfg(feature = "morphology")]
fn ordered_subsequence(returned: &[String], requested: &[String]) -> bool {
    if requested.is_empty() {
        return false;
    }
    let mut next = 0;
    for component in returned {
        if component == &requested[next] {
            next += 1;
            if next == requested.len() {
                return true;
            }
        }
    }
    false
}

#[cfg(feature = "morphology")]
fn guarded_veljvo(candidate: &str) -> Result<Vec<String>, String> {
    if candidate.is_empty() || !candidate.is_ascii() {
        return Err("unsupported-candidate-non-ascii".into());
    }
    let settings = latkerlo_jvotci::Settings::default();
    let result = std::panic::catch_unwind(|| latkerlo_jvotci::get_veljvo(candidate, &settings));
    match result {
        Ok(Ok(parts)) => Ok(parts),
        Ok(Err(_)) => Err("morphology-parse-rejected".into()),
        Err(_) => Err("morphology-panic-caught-unwind-only".into()),
    }
}

/// Exposes whether the optional morphology implementation was compiled.
#[must_use]
pub const fn morphology_enabled() -> bool {
    cfg!(feature = "morphology")
}

#[cfg(all(test, feature = "morphology"))]
mod component_semantics {
    use super::ordered_subsequence;

    fn values(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn component_membership_is_an_exact_ordered_subsequence_with_multiplicity() {
        let engine = values(&["a", "b", "c"]);
        assert!(ordered_subsequence(&engine, &values(&["a"])));
        assert!(ordered_subsequence(&engine, &values(&["a", "c"])));
        assert!(ordered_subsequence(&engine, &engine));
        assert!(!ordered_subsequence(&engine, &values(&["c", "a"])));
        assert!(!ordered_subsequence(&engine, &values(&["a", "a"])));
        assert!(!ordered_subsequence(
            &engine,
            &values(&["a", "b", "c", "d"])
        ));
        assert!(!ordered_subsequence(&engine, &[]));
        assert!(!ordered_subsequence(&values(&["ab"]), &values(&["a"])));
    }
}
