//! Targeted projections of retained collection metadata.

use std::collections::{BTreeMap, BTreeSet};

use rusqlite::{OptionalExtension, params};
use serde::{
    Deserialize, Serialize,
    de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor},
};
use serde_json::value::RawValue;
use std::fmt;
use vlajmaji_lexicon::{
    CaptureMetadata, CaptureRef, CodeNotice, CollectionRecord, ComponentRecord, ContributorId,
    DistributorId, LocalRef, OccurrenceAttribution, OccurrenceRef, PackId, RegistryDocument,
    RelationTarget, ReleaseAccounting, SnapshotAttribution, SourceAlias, SourceId, SourceLineage,
    SourceRecord, TimePrecision,
};

use super::{
    CollectionCatalogue, CollectionCategoryCounts, CollectionEntityReference, CollectionError,
    CollectionReleaseSummary, db_error,
};

/// Targeted retained source/reference projection for one relation target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionReferenceProjection {
    /// Descriptor version.
    pub version: u32,
    /// Stable source pack identity.
    pub pack_id: String,
    /// Exact registered release digest.
    pub release_sha256: String,
    /// Declared payload scope.
    pub payload_kind: String,
    /// Dictionary schema version.
    pub dictionary_version: u32,
    /// Registry schema version.
    pub registry_version: u32,
    /// Upstream catalogue revision.
    pub upstream_catalogue_revision: u64,
    /// Target entity and directly linked entry/assertion descriptors.
    pub entities: Vec<CollectionEntityReference>,
    /// Digests for projected entities only.
    pub body_sha256: BTreeMap<String, String>,
    /// Registry records related to the projected entities.
    pub registry: RegistryDocument,
    /// Related release summaries and target-matching assertion IDs.
    pub releases: Vec<CollectionProjectedRelease>,
    /// Captures referenced by the target, with target locator observations.
    pub captures: Vec<CollectionProjectedCapture>,
    /// Retained code notices.
    pub code_notices: Vec<CodeNotice>,
}

/// One release summary with only matching assertion membership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionProjectedRelease {
    /// Existing compact release provenance.
    pub summary: CollectionReleaseSummary,
    /// Target-matching assertions in original release order.
    pub matching_assertion_ids: Vec<vlajmaji_lexicon::EntityId>,
}

/// One target capture and the requested locator-presence observations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionProjectedCapture {
    /// Capture identity.
    pub capture: CaptureRef,
    /// Capture metadata.
    pub metadata: CaptureMetadata,
    /// Captured byte length.
    pub byte_length: usize,
    /// Presence of only the requested target locators.
    pub locator_presence: BTreeMap<String, bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound(deserialize = "'de: 'a"))]
struct RawDescriptor<'a> {
    version: u32,
    pack_id: String,
    release_sha256: String,
    payload_kind: String,
    dictionary_version: u32,
    registry_version: u32,
    upstream_catalogue_revision: u64,
    registry: &'a RawValue,
    source_metadata: &'a RawValue,
    captures: Vec<&'a RawValue>,
    entities: Vec<&'a RawValue>,
    body_sha256: &'a RawValue,
    extension_row_sha256_ordered_multiset: IgnoredAny,
    category_counts: CollectionCategoryCounts,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSourceMetadata {
    collections: Vec<CollectionRecord>,
    components: Vec<ComponentRecord>,
    releases: Vec<Box<RawValue>>,
    code_notices: Vec<CodeNotice>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct RawRegistry<'a> {
    version: u32,
    sources: Vec<&'a RawValue>,
    contributors: Vec<&'a RawValue>,
    distributors: Vec<&'a RawValue>,
    aliases: Vec<&'a RawValue>,
    lineage: Vec<&'a RawValue>,
    snapshots: Vec<&'a RawValue>,
    occurrences: Vec<&'a RawValue>,
}

#[derive(Deserialize)]
struct SourceKey {
    id: SourceId,
}

#[derive(Deserialize)]
struct AliasKey {
    source: SourceId,
}

#[derive(Deserialize)]
struct LineageKey {
    source: SourceId,
    upstream: vlajmaji_lexicon::LineageTarget,
}

#[derive(Deserialize)]
struct SnapshotKey {
    snapshot: vlajmaji_lexicon::SnapshotIdentity,
}

#[derive(Deserialize)]
struct OccurrenceKey {
    occurrence: vlajmaji_lexicon::RecordIdentity,
}

type OccurrenceTuple = (String, String, String);

/// Per-call ordered candidate index for the native-only phase of a Catalog
/// projection. Keys are decoded only through the first malformed raw key;
/// complete attribution bodies remain lazy and are decoded only for matching
/// tuples in their original row order.
struct RawOccurrenceNativeLookup<'a> {
    rows: &'a [&'a RawValue],
    ordinals: BTreeMap<OccurrenceTuple, Vec<usize>>,
    first_bad_key: Option<usize>,
    successful_matches: BTreeMap<OccurrenceTuple, bool>,
}

impl<'a> RawOccurrenceNativeLookup<'a> {
    fn new(rows: &'a [&'a RawValue]) -> Self {
        let mut ordinals = BTreeMap::<OccurrenceTuple, Vec<usize>>::new();
        let mut first_bad_key = None;
        for (ordinal, row) in rows.iter().enumerate() {
            let Ok(key) = serde_json::from_str::<OccurrenceKey>(row.get()) else {
                first_bad_key = Some(ordinal);
                break;
            };
            ordinals
                .entry((
                    key.occurrence.snapshot().namespace().to_owned(),
                    key.occurrence.snapshot().sha256().to_owned(),
                    key.occurrence.locator().to_owned(),
                ))
                .or_default()
                .push(ordinal);
        }
        Self {
            rows,
            ordinals,
            first_bad_key,
            successful_matches: BTreeMap::new(),
        }
    }

    fn matches(
        &mut self,
        occurrence: &OccurrenceRef,
        native_entry_id: Option<&str>,
        native_assertion_id: Option<&str>,
        pack: PackId,
        release_sha256: &str,
    ) -> Result<bool, CollectionError> {
        let tuple = (
            occurrence.capture.namespace.clone(),
            occurrence.capture.sha256.clone(),
            occurrence.locator.clone(),
        );
        if let Some(matched) = self.successful_matches.get(&tuple) {
            return Ok(*matched);
        }

        if let Some(ordinals) = self.ordinals.get(&tuple) {
            for ordinal in ordinals {
                let row: OccurrenceAttribution = serde_json::from_str(self.rows[*ordinal].get())
                    .map_err(|source| CollectionError::ReferenceProjection {
                        pack,
                        release_sha256: release_sha256.to_owned(),
                        source,
                    })?;
                let matches = native_entry_id
                    .is_none_or(|wanted| row.native_entry_id.as_deref() == Some(wanted))
                    && native_assertion_id
                        .is_none_or(|wanted| row.native_assertion_id.as_deref() == Some(wanted));
                if matches {
                    self.successful_matches.insert(tuple, true);
                    return Ok(true);
                }
            }
        }

        if let Some(ordinal) = self.first_bad_key {
            return match serde_json::from_str::<OccurrenceKey>(self.rows[ordinal].get()) {
                Err(source) => Err(CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                }),
                Ok(_) => unreachable!("the deferred occurrence key was invalid during indexing"),
            };
        }

        self.successful_matches.insert(tuple, false);
        Ok(false)
    }
}

#[derive(Deserialize)]
struct ContributorKey {
    id: ContributorId,
}

#[derive(Deserialize)]
struct DistributorKey {
    id: DistributorId,
}

#[derive(Deserialize)]
struct EntityKey {
    role: String,
    id: String,
}

#[derive(Deserialize)]
struct EntityLinks {
    role: String,
    id: String,
    entry_ids: Vec<String>,
    assertion_ids: Vec<String>,
    source_ids: Vec<String>,
    occurrence: Option<OccurrenceRef>,
}

#[derive(Deserialize)]
struct CaptureKey {
    capture: CaptureRef,
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
struct CaptureProjection<'a> {
    capture: CaptureRef,
    metadata: CaptureMetadata,
    byte_length: usize,
    occurrences: &'a RawValue,
}

#[derive(Deserialize)]
struct ReleaseKey {
    id: vlajmaji_lexicon::EntityId,
    component: vlajmaji_lexicon::EntityId,
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
struct ReleaseProjection<'a> {
    id: vlajmaji_lexicon::EntityId,
    component: vlajmaji_lexicon::EntityId,
    distributor: Option<DistributorId>,
    upstream_revision: vlajmaji_lexicon::Presence<String>,
    distributor_revision: vlajmaji_lexicon::Presence<String>,
    captured_at: vlajmaji_lexicon::Presence<String>,
    time_precision: TimePrecision,
    schema_version: u32,
    normalizer: String,
    normalizer_version: String,
    captures: Vec<CaptureRef>,
    assertions: &'a RawValue,
    accounting: ReleaseAccounting,
    diagnostics: &'a RawValue,
    selected: bool,
    availability: vlajmaji_lexicon::Availability,
    content_notices: Vec<NoticeProjection>,
    redistribution: vlajmaji_lexicon::RedistributionStatus,
    redistribution_evidence: Vec<String>,
}

#[derive(Deserialize)]
struct NoticeProjection {
    license: vlajmaji_lexicon::Presence<String>,
    locator: vlajmaji_lexicon::Presence<String>,
}

fn projected_assertion_membership(
    raw: &RawValue,
    wanted: &BTreeSet<String>,
) -> Result<Vec<vlajmaji_lexicon::EntityId>, serde_json::Error> {
    struct Seed<'a>(&'a BTreeSet<String>);
    impl<'de> DeserializeSeed<'de> for Seed<'_> {
        type Value = Vec<vlajmaji_lexicon::EntityId>;
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct V<'a>(&'a BTreeSet<String>);
            impl<'de> Visitor<'de> for V<'_> {
                type Value = Vec<vlajmaji_lexicon::EntityId>;
                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("a release assertion ID array")
                }
                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let mut matching = Vec::new();
                    while let Some(id) = seq.next_element::<vlajmaji_lexicon::EntityId>()? {
                        if self.0.contains(id.as_str()) {
                            matching.push(id);
                        }
                    }
                    Ok(matching)
                }
            }
            deserializer.deserialize_seq(V(self.0))
        }
    }
    let mut deserializer = serde_json::Deserializer::from_str(raw.get());
    Seed(wanted).deserialize(&mut deserializer)
}

fn raw_array_len(raw: &RawValue) -> Result<usize, serde_json::Error> {
    struct Count;
    impl<'de> DeserializeSeed<'de> for Count {
        type Value = usize;
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct V;
            impl<'de> Visitor<'de> for V {
                type Value = usize;
                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("an array")
                }
                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let mut count = 0usize;
                    while seq.next_element::<IgnoredAny>()?.is_some() {
                        count += 1;
                    }
                    Ok(count)
                }
            }
            deserializer.deserialize_seq(V)
        }
    }
    let mut deserializer = serde_json::Deserializer::from_str(raw.get());
    Count.deserialize(&mut deserializer)
}

fn projected_locator_presence(
    raw: &RawValue,
    requested: &BTreeSet<String>,
) -> Result<BTreeSet<String>, serde_json::Error> {
    struct Seed<'a>(&'a BTreeSet<String>);
    impl<'de> DeserializeSeed<'de> for Seed<'_> {
        type Value = BTreeSet<String>;
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct V<'a>(&'a BTreeSet<String>);
            impl<'de> Visitor<'de> for V<'_> {
                type Value = BTreeSet<String>;
                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("capture occurrence records")
                }
                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let mut present = BTreeSet::new();
                    while let Some(row) = seq.next_element::<&RawValue>()? {
                        #[derive(Deserialize)]
                        struct Locator {
                            locator: String,
                        }
                        let key: Locator =
                            serde_json::from_str(row.get()).map_err(serde::de::Error::custom)?;
                        if self.0.contains(&key.locator) {
                            present.insert(key.locator);
                        }
                    }
                    Ok(present)
                }
            }
            deserializer.deserialize_seq(V(self.0))
        }
    }
    let mut deserializer = serde_json::Deserializer::from_str(raw.get());
    Seed(requested).deserialize(&mut deserializer)
}

fn release_summary(
    release: ReleaseProjection<'_>,
) -> Result<CollectionReleaseSummary, serde_json::Error> {
    let diagnostics_count = raw_array_len(release.diagnostics)?;
    Ok(CollectionReleaseSummary {
        id: release.id.to_string(),
        component_id: release.component.to_string(),
        distributor_id: release.distributor.map(|id| id.to_string()),
        upstream_revision: release.upstream_revision,
        distributor_revision: release.distributor_revision,
        captured_at: release.captured_at,
        time_precision: release.time_precision,
        schema_version: release.schema_version,
        normalizer: release.normalizer,
        normalizer_version: release.normalizer_version,
        captures: release.captures,
        accounting: release.accounting,
        diagnostics_count,
        selected: release.selected,
        availability: release.availability,
        content_notices_count: release.content_notices.len(),
        content_notice_licenses: release
            .content_notices
            .iter()
            .map(|notice| notice.license.clone())
            .collect(),
        content_notice_locators: release
            .content_notices
            .iter()
            .map(|notice| notice.locator.clone())
            .collect(),
        redistribution: format!("{:?}", release.redistribution).to_lowercase(),
        redistribution_evidence: release.redistribution_evidence,
    })
}

fn deserialize_digest_map(
    bytes: &[u8],
    wanted: &BTreeSet<String>,
) -> Result<BTreeMap<String, String>, serde_json::Error> {
    struct Seed<'a>(&'a BTreeSet<String>);
    impl<'de> serde::de::DeserializeSeed<'de> for Seed<'_> {
        type Value = BTreeMap<String, String>;
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct V<'a>(&'a BTreeSet<String>);
            impl<'de> Visitor<'de> for V<'_> {
                type Value = BTreeMap<String, String>;
                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("a digest map")
                }
                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>,
                {
                    let mut result = BTreeMap::new();
                    while let Some(key) = map.next_key::<String>()? {
                        if self.0.contains(&key) {
                            result.insert(key, map.next_value()?);
                        } else {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                    Ok(result)
                }
            }
            deserializer.deserialize_map(V(self.0))
        }
    }
    let raw: &RawValue = serde_json::from_slice(bytes)?;
    let mut deserializer = serde_json::Deserializer::from_str(raw.get());
    Seed(wanted).deserialize(&mut deserializer)
}

impl CollectionCatalogue {
    /// Return a metadata-only projection for one typed local or catalog target.
    ///
    /// The retained byte buffer is replayed through `RawValue`; only selected
    /// entity, capture, release, and digest rows are materialized as typed data.
    pub fn reference_projection(
        &self,
        pack: PackId,
        release_sha256: &str,
        target: &RelationTarget,
    ) -> Result<CollectionReferenceProjection, CollectionError> {
        let tables = crate::collections_schema::tables_for(&self.connection, &self.root)?;
        let (payload_kind, bytes): (String, Vec<u8>) = self.connection.query_row(
            &format!("SELECT payload_kind,descriptor_json FROM {} WHERE pack_id=?1 AND release_sha256=?2", tables.registrations),
            params![pack.as_str(), release_sha256],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional().map_err(|error| db_error(&self.root, error))?
          .ok_or_else(|| CollectionError::InvalidRequest(format!("no registered {} release {release_sha256}", pack.as_str())))?;
        let raw: RawDescriptor<'_> = serde_json::from_slice(&bytes).map_err(|source| {
            CollectionError::ReferenceProjection {
                pack,
                release_sha256: release_sha256.to_owned(),
                source,
            }
        })?;
        let _ignored_descriptor_inventory = (
            raw.extension_row_sha256_ordered_multiset,
            &raw.category_counts,
        );
        if raw.version != 1
            || raw.pack_id != pack.as_str()
            || raw.release_sha256 != release_sha256
            || raw.payload_kind != payload_kind
            || raw.dictionary_version != 1
            || raw.registry_version == 0
        {
            return Err(CollectionError::UnsupportedVersion(
                "retained reference descriptor header does not match its registration".into(),
            ));
        }
        let registry: RawRegistry<'_> =
            serde_json::from_str(raw.registry.get()).map_err(|source| {
                CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                }
            })?;
        if registry.version != raw.registry_version {
            return Err(CollectionError::UnsupportedVersion(
                "retained registry descriptor version does not match its registry document".into(),
            ));
        }
        let source_metadata: RawSourceMetadata = serde_json::from_str(raw.source_metadata.get())
            .map_err(|source| CollectionError::ReferenceProjection {
                pack,
                release_sha256: release_sha256.to_owned(),
                source,
            })?;
        let (wanted_role, catalog_target, catalog_release) = match target {
            RelationTarget::Local(local) => {
                let (role, id) = match local {
                    LocalRef::Entry(id) => ("entry", id.as_str()),
                    LocalRef::Assertion(id) => ("assertion", id.as_str()),
                    LocalRef::Form(id) => ("form", id.as_str()),
                    LocalRef::EditorialSense(id) => ("editorial_sense", id.as_str()),
                };
                (Some((role, id)), None, None)
            }
            RelationTarget::Catalog(catalog) => (None, Some(catalog), Some(&catalog.release)),
            RelationTarget::Unknown | RelationTarget::Unresolved(_) => (None, None, None),
        };
        let mut entity_rows = Vec::new();
        for entity_raw in &raw.entities {
            let key: EntityKey = serde_json::from_str(entity_raw.get()).map_err(|source| {
                CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                }
            })?;
            let links: EntityLinks = serde_json::from_str(entity_raw.get()).map_err(|source| {
                CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                }
            })?;
            entity_rows.push((key, links, *entity_raw));
        }
        let mut wanted_ids = BTreeSet::<(String, String)>::new();
        if let Some((role, id)) = wanted_role {
            wanted_ids.insert((role.to_owned(), id.to_owned()));
            // Preserve a same-ID role witness so the search layer can return
            // InvalidReference instead of treating a known ID as absent.
            for (key, _, _) in &entity_rows {
                if key.id == id {
                    wanted_ids.insert((key.role.clone(), key.id.clone()));
                }
            }
            let target_links = entity_rows
                .iter()
                .find(|(key, _, _)| key.role == role && key.id == id)
                .map(|(_, links, _)| links);
            if let Some(links) = target_links {
                wanted_ids.extend(
                    links
                        .entry_ids
                        .iter()
                        .map(|entry| ("entry".to_owned(), entry.clone())),
                );
                wanted_ids.extend(
                    links
                        .assertion_ids
                        .iter()
                        .map(|assertion| ("assertion".to_owned(), assertion.clone())),
                );
            }
            match role {
                "entry" => {
                    for (_, links, _) in &entity_rows {
                        if links.role == "assertion" && links.entry_ids.contains(&id.to_owned()) {
                            wanted_ids.insert((links.role.clone(), links.id.clone()));
                        }
                    }
                }
                "form" => {
                    let entries: BTreeSet<_> = wanted_ids
                        .iter()
                        .filter(|(linked_role, _)| linked_role == "entry")
                        .map(|(_, linked_id)| linked_id.clone())
                        .collect();
                    for (_, links, _) in &entity_rows {
                        if links.role == "assertion"
                            && links.entry_ids.iter().any(|entry| entries.contains(entry))
                        {
                            wanted_ids.insert((links.role.clone(), links.id.clone()));
                        }
                    }
                }
                "editorial_sense" | "assertion" => {
                    let assertions: BTreeSet<_> = wanted_ids
                        .iter()
                        .filter(|(linked_role, _)| linked_role == "assertion")
                        .map(|(_, linked_id)| linked_id.clone())
                        .collect();
                    for (_, links, _) in &entity_rows {
                        if links.role == "assertion" && assertions.contains(&links.id) {
                            wanted_ids.extend(
                                links
                                    .entry_ids
                                    .iter()
                                    .map(|entry| ("entry".to_owned(), entry.clone())),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
        let mut catalog_assertion_ids = BTreeSet::new();
        let mut catalog_release_owned = false;
        if let Some(catalog) = catalog_target {
            let mut catalog_candidate_ids = BTreeSet::new();
            let mut native_lookup = None;
            for (_, assertion, _) in entity_rows
                .iter()
                .filter(|(_, links, _)| links.role == "assertion")
            {
                if !assertion
                    .source_ids
                    .iter()
                    .any(|source| source == &catalog.source.to_string())
                {
                    continue;
                }
                let Some(occurrence) = assertion.occurrence.as_ref() else {
                    continue;
                };
                if catalog
                    .occurrence
                    .as_ref()
                    .is_some_and(|wanted| wanted != occurrence)
                {
                    continue;
                }
                if catalog.native_entry_id.is_some() || catalog.native_assertion_id.is_some() {
                    let lookup = native_lookup.get_or_insert_with(|| {
                        RawOccurrenceNativeLookup::new(&registry.occurrences)
                    });
                    if !lookup.matches(
                        occurrence,
                        catalog.native_entry_id.as_deref(),
                        catalog.native_assertion_id.as_deref(),
                        pack,
                        release_sha256,
                    )? {
                        continue;
                    }
                }
                catalog_candidate_ids.insert(assertion.id.clone());
            }
            let candidate_strings: BTreeSet<_> = catalog_candidate_ids
                .iter()
                .map(|id| id.to_string())
                .collect();
            for release_raw in &source_metadata.releases {
                let key: ReleaseKey =
                    serde_json::from_str(release_raw.get()).map_err(|source| {
                        CollectionError::ReferenceProjection {
                            pack,
                            release_sha256: release_sha256.to_owned(),
                            source,
                        }
                    })?;
                if key.id != catalog.release {
                    continue;
                }
                let component = source_metadata
                    .components
                    .iter()
                    .find(|component| component.id == key.component);
                let collection = component.and_then(|component| {
                    source_metadata
                        .collections
                        .iter()
                        .find(|collection| collection.id == component.collection)
                });
                if !collection.is_some_and(|collection| collection.source == catalog.source) {
                    break;
                }
                catalog_release_owned = true;
                let release: ReleaseProjection<'_> = serde_json::from_str(release_raw.get())
                    .map_err(|source| CollectionError::ReferenceProjection {
                        pack,
                        release_sha256: release_sha256.to_owned(),
                        source,
                    })?;
                catalog_assertion_ids.extend(
                    projected_assertion_membership(release.assertions, &candidate_strings)
                        .map_err(|source| CollectionError::ReferenceProjection {
                            pack,
                            release_sha256: release_sha256.to_owned(),
                            source,
                        })?,
                );
                break;
            }
        }
        for assertion_id in &catalog_assertion_ids {
            wanted_ids.insert(("assertion".into(), assertion_id.to_string()));
            if let Some((_, assertion, _)) = entity_rows.iter().find(|(_, links, _)| {
                links.role == "assertion" && links.id == assertion_id.as_str()
            }) {
                for entry_id in &assertion.entry_ids {
                    wanted_ids.insert(("entry".into(), entry_id.clone()));
                }
            }
        }
        let mut lightweight = Vec::<CollectionEntityReference>::new();
        for (key, _, entity_raw) in entity_rows {
            if !wanted_ids.contains(&(key.role.clone(), key.id.clone())) {
                continue;
            }
            let entity: CollectionEntityReference = serde_json::from_str(entity_raw.get())
                .map_err(|source| CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                })?;
            lightweight.push(entity);
        }
        let entities: Vec<_> = lightweight
            .into_iter()
            .filter(|entity| wanted_ids.contains(&(entity.role.clone(), entity.id.clone())))
            .collect();
        let wanted_digest_keys: BTreeSet<_> = wanted_ids
            .iter()
            .map(|(role, id)| format!("{role}:{id}"))
            .collect();
        let body_sha256 =
            deserialize_digest_map(raw.body_sha256.get().as_bytes(), &wanted_digest_keys).map_err(
                |source| CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                },
            )?;
        let mut requested_occurrences: Vec<OccurrenceRef> = entities
            .iter()
            .filter_map(|entity| entity.occurrence.clone())
            .collect();
        if let Some(occurrence) = catalog_target.and_then(|catalog| catalog.occurrence.clone())
            && !requested_occurrences.contains(&occurrence)
        {
            requested_occurrences.push(occurrence);
        }
        let requested_captures: BTreeSet<_> = requested_occurrences
            .iter()
            .map(|occurrence| occurrence.capture.clone())
            .collect();
        let mut captures = Vec::new();
        for capture_raw in &raw.captures {
            let key: CaptureKey = serde_json::from_str(capture_raw.get()).map_err(|source| {
                CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                }
            })?;
            if requested_captures.contains(&key.capture) {
                let capture: CaptureProjection<'_> = serde_json::from_str(capture_raw.get())
                    .map_err(|source| CollectionError::ReferenceProjection {
                        pack,
                        release_sha256: release_sha256.to_owned(),
                        source,
                    })?;
                let requested_locators: BTreeSet<_> = requested_occurrences
                    .iter()
                    .filter(|occurrence| occurrence.capture == capture.capture)
                    .map(|occurrence| occurrence.locator.clone())
                    .collect();
                let present_locators =
                    projected_locator_presence(capture.occurrences, &requested_locators).map_err(
                        |source| CollectionError::ReferenceProjection {
                            pack,
                            release_sha256: release_sha256.to_owned(),
                            source,
                        },
                    )?;
                let mut locator_presence = BTreeMap::new();
                for occurrence in requested_occurrences
                    .iter()
                    .filter(|occurrence| occurrence.capture == capture.capture)
                {
                    locator_presence.insert(
                        occurrence.locator.clone(),
                        present_locators.contains(&occurrence.locator),
                    );
                }
                captures.push(CollectionProjectedCapture {
                    capture: capture.capture,
                    metadata: capture.metadata,
                    byte_length: capture.byte_length,
                    locator_presence,
                });
            }
        }
        let mut projected_releases = Vec::new();
        let wanted_assertions: BTreeSet<_> = wanted_ids
            .iter()
            .filter(|(role, _)| role == "assertion")
            .map(|(_, id)| id.clone())
            .collect();
        for release_raw in &source_metadata.releases {
            let key: ReleaseKey = serde_json::from_str(release_raw.get()).map_err(|source| {
                CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                }
            })?;
            if catalog_release.is_some_and(|id| id != &key.id) {
                continue;
            }
            let release: ReleaseProjection<'_> =
                serde_json::from_str(release_raw.get()).map_err(|source| {
                    CollectionError::ReferenceProjection {
                        pack,
                        release_sha256: release_sha256.to_owned(),
                        source,
                    }
                })?;
            let matching_assertion_ids =
                projected_assertion_membership(release.assertions, &wanted_assertions).map_err(
                    |source| CollectionError::ReferenceProjection {
                        pack,
                        release_sha256: release_sha256.to_owned(),
                        source,
                    },
                )?;
            let requested_catalog_release =
                catalog_release_owned && catalog_release.is_some_and(|id| id == &key.id);
            if !matching_assertion_ids.is_empty() || requested_catalog_release {
                let summary = release_summary(release).map_err(|source| {
                    CollectionError::ReferenceProjection {
                        pack,
                        release_sha256: release_sha256.to_owned(),
                        source,
                    }
                })?;
                projected_releases.push(CollectionProjectedRelease {
                    summary,
                    matching_assertion_ids,
                });
            }
        }
        let mut related_source_ids: BTreeSet<SourceId> = entities
            .iter()
            .flat_map(|entity| {
                entity
                    .source_ids
                    .iter()
                    .filter_map(|id| SourceId::new(id.clone()).ok())
            })
            .collect();
        if let Some(catalog) = catalog_target {
            related_source_ids.insert(catalog.source.clone());
        }
        let requested_snapshot_ids: BTreeSet<_> = requested_occurrences
            .iter()
            .map(|occurrence| {
                (
                    occurrence.capture.namespace.clone(),
                    occurrence.capture.sha256.clone(),
                )
            })
            .collect();
        let requested_occurrence_ids: BTreeSet<_> = requested_occurrences
            .iter()
            .map(|occurrence| {
                (
                    occurrence.capture.namespace.clone(),
                    occurrence.capture.sha256.clone(),
                    occurrence.locator.clone(),
                )
            })
            .collect();
        let mut sources = Vec::new();
        for row in &registry.sources {
            let key: SourceKey = serde_json::from_str(row.get()).map_err(|source| {
                CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                }
            })?;
            if related_source_ids.contains(&key.id) {
                sources.push(serde_json::from_str::<SourceRecord>(row.get()).map_err(
                    |source| CollectionError::ReferenceProjection {
                        pack,
                        release_sha256: release_sha256.to_owned(),
                        source,
                    },
                )?);
            }
        }
        let related_sources: BTreeSet<_> = sources.iter().map(|row| row.id.clone()).collect();
        let mut aliases = Vec::new();
        for row in &registry.aliases {
            let key: AliasKey = serde_json::from_str(row.get()).map_err(|source| {
                CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                }
            })?;
            if related_sources.contains(&key.source) {
                aliases.push(
                    serde_json::from_str::<SourceAlias>(row.get()).map_err(|source| {
                        CollectionError::ReferenceProjection {
                            pack,
                            release_sha256: release_sha256.to_owned(),
                            source,
                        }
                    })?,
                );
            }
        }
        let mut lineage = Vec::new();
        for row in &registry.lineage {
            let key: LineageKey = serde_json::from_str(row.get()).map_err(|source| {
                CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                }
            })?;
            let related = related_sources.contains(&key.source)
                || matches!(&key.upstream, vlajmaji_lexicon::LineageTarget::Known(id) if related_sources.contains(id));
            if related {
                lineage.push(serde_json::from_str::<SourceLineage>(row.get()).map_err(
                    |source| CollectionError::ReferenceProjection {
                        pack,
                        release_sha256: release_sha256.to_owned(),
                        source,
                    },
                )?);
            }
        }
        let mut snapshots = Vec::new();
        for row in &registry.snapshots {
            let key: SnapshotKey = serde_json::from_str(row.get()).map_err(|source| {
                CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                }
            })?;
            if requested_snapshot_ids.contains(&(
                key.snapshot.namespace().to_owned(),
                key.snapshot.sha256().to_owned(),
            )) {
                snapshots.push(
                    serde_json::from_str::<SnapshotAttribution>(row.get()).map_err(|source| {
                        CollectionError::ReferenceProjection {
                            pack,
                            release_sha256: release_sha256.to_owned(),
                            source,
                        }
                    })?,
                );
            }
        }
        let mut occurrences = Vec::new();
        for row in &registry.occurrences {
            let key: OccurrenceKey = serde_json::from_str(row.get()).map_err(|source| {
                CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                }
            })?;
            let id = (
                key.occurrence.snapshot().namespace().to_owned(),
                key.occurrence.snapshot().sha256().to_owned(),
                key.occurrence.locator().to_owned(),
            );
            if requested_occurrence_ids.contains(&id) {
                occurrences.push(
                    serde_json::from_str::<OccurrenceAttribution>(row.get()).map_err(|source| {
                        CollectionError::ReferenceProjection {
                            pack,
                            release_sha256: release_sha256.to_owned(),
                            source,
                        }
                    })?,
                );
            }
        }
        let contributor_ids: BTreeSet<_> = occurrences
            .iter()
            .filter_map(|row| row.contributor.clone())
            .collect();
        let distributor_ids: BTreeSet<_> = snapshots
            .iter()
            .filter_map(|row| row.distributor.clone())
            .chain(
                projected_releases
                    .iter()
                    .filter_map(|row| row.summary.distributor_id.as_ref())
                    .filter_map(|id| DistributorId::new(id.clone()).ok()),
            )
            .collect();
        let mut contributors = Vec::new();
        for row in &registry.contributors {
            let key: ContributorKey = serde_json::from_str(row.get()).map_err(|source| {
                CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                }
            })?;
            if contributor_ids.contains(&key.id) {
                contributors.push(
                    serde_json::from_str::<vlajmaji_lexicon::ContributorRecord>(row.get())
                        .map_err(|source| CollectionError::ReferenceProjection {
                            pack,
                            release_sha256: release_sha256.to_owned(),
                            source,
                        })?,
                );
            }
        }
        let mut distributors = Vec::new();
        for row in &registry.distributors {
            let key: DistributorKey = serde_json::from_str(row.get()).map_err(|source| {
                CollectionError::ReferenceProjection {
                    pack,
                    release_sha256: release_sha256.to_owned(),
                    source,
                }
            })?;
            if distributor_ids.contains(&key.id) {
                distributors.push(
                    serde_json::from_str::<vlajmaji_lexicon::DistributorRecord>(row.get())
                        .map_err(|source| CollectionError::ReferenceProjection {
                            pack,
                            release_sha256: release_sha256.to_owned(),
                            source,
                        })?,
                );
            }
        }
        let filtered_registry = RegistryDocument {
            version: registry.version,
            sources,
            contributors,
            distributors,
            aliases,
            lineage,
            snapshots,
            occurrences,
        };
        let relevant_notices = source_metadata.code_notices;
        Ok(CollectionReferenceProjection {
            version: raw.version,
            pack_id: raw.pack_id,
            release_sha256: raw.release_sha256,
            payload_kind: raw.payload_kind,
            dictionary_version: raw.dictionary_version,
            registry_version: raw.registry_version,
            upstream_catalogue_revision: raw.upstream_catalogue_revision,
            entities,
            body_sha256,
            registry: filtered_registry,
            releases: projected_releases,
            captures,
            code_notices: relevant_notices,
        })
    }
}

#[cfg(test)]
mod native_lookup_tests {
    use super::*;

    fn occurrence(namespace: &str, locator: &str) -> OccurrenceRef {
        OccurrenceRef {
            capture: CaptureRef {
                namespace: namespace.to_owned(),
                sha256: "a".repeat(64),
            },
            locator: locator.to_owned(),
        }
    }

    fn raw_key(occurrence: &OccurrenceRef) -> String {
        serde_json::to_string(&occurrence.to_record_identity().unwrap()).unwrap()
    }

    fn raw_attribution(
        occurrence: &OccurrenceRef,
        native_entry_id: Option<&str>,
        native_assertion_id: Option<&str>,
    ) -> Box<RawValue> {
        let value = serde_json::json!({
            "occurrence": serde_json::from_str::<serde_json::Value>(&raw_key(occurrence)).unwrap(),
            "contributor": null,
            "native_entry_id": native_entry_id,
            "native_assertion_id": native_assertion_id,
        });
        RawValue::from_string(serde_json::to_string(&value).unwrap()).unwrap()
    }

    fn row_refs(rows: &[Box<RawValue>]) -> Vec<&RawValue> {
        rows.iter().map(Box::as_ref).collect()
    }

    fn legacy_native_match(
        rows: &[&RawValue],
        occurrence: &OccurrenceRef,
        native_entry_id: Option<&str>,
        native_assertion_id: Option<&str>,
    ) -> Result<bool, serde_json::Error> {
        for raw in rows {
            let key: OccurrenceKey = serde_json::from_str(raw.get())?;
            if key.occurrence.snapshot().namespace() != occurrence.capture.namespace
                || key.occurrence.snapshot().sha256() != occurrence.capture.sha256
                || key.occurrence.locator() != occurrence.locator
            {
                continue;
            }
            let row: OccurrenceAttribution = serde_json::from_str(raw.get())?;
            let matches = native_entry_id
                .is_none_or(|wanted| row.native_entry_id.as_deref() == Some(wanted))
                && native_assertion_id
                    .is_none_or(|wanted| row.native_assertion_id.as_deref() == Some(wanted));
            if matches {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn assert_reference_json_data(error: CollectionError) {
        match error {
            CollectionError::ReferenceProjection { source, .. } => {
                assert_eq!(source.classify(), serde_json::error::Category::Data);
            }
            other => panic!("expected the original serde projection error, got {other:?}"),
        }
    }

    fn matches(
        lookup: &mut RawOccurrenceNativeLookup<'_>,
        occurrence: &OccurrenceRef,
        expected: Option<&str>,
    ) -> Result<bool, CollectionError> {
        lookup.matches(occurrence, expected, None, PackId::Llg, &"d".repeat(64))
    }

    #[test]
    fn raw_native_lookup_keeps_ordered_any_and_first_success() {
        let wanted = occurrence("captured:test", "/wanted");
        let other = occurrence("captured:test", "/other");
        let rows = vec![
            raw_attribution(&wanted, None, None),
            raw_attribution(&wanted, Some("entry-wanted"), None),
            raw_attribution(&other, Some("entry-wanted"), None),
            raw_attribution(&wanted, Some("entry-later"), None),
        ];
        let refs = row_refs(&rows);
        let mut lookup = RawOccurrenceNativeLookup::new(&refs);
        assert_eq!(
            matches(&mut lookup, &wanted, Some("entry-wanted")).unwrap(),
            legacy_native_match(&refs, &wanted, Some("entry-wanted"), None).unwrap()
        );
        assert_eq!(
            matches(&mut lookup, &other, Some("entry-wanted")).unwrap(),
            legacy_native_match(&refs, &other, Some("entry-wanted"), None).unwrap()
        );
        let rows = vec![raw_attribution(&other, Some("entry-wanted"), None)];
        let refs = row_refs(&rows);
        let mut lookup = RawOccurrenceNativeLookup::new(&refs);
        assert_eq!(
            matches(&mut lookup, &other, Some("absent")).unwrap(),
            legacy_native_match(&refs, &other, Some("absent"), None).unwrap()
        );
    }

    #[test]
    fn raw_native_lookup_defers_key_errors_until_the_legacy_visit_point() {
        let first = occurrence("captured:test", "/first");
        let second = occurrence("captured:test", "/second");
        let good = raw_attribution(&first, Some("entry-wanted"), None);
        let malformed_key =
            RawValue::from_string(r#"{"occurrence":"truncated"}"#.to_owned()).unwrap();
        let rows = vec![good, malformed_key];
        let refs = row_refs(&rows);
        let mut lookup = RawOccurrenceNativeLookup::new(&refs);
        assert!(matches(&mut lookup, &first, Some("entry-wanted")).unwrap());
        assert!(legacy_native_match(&refs, &first, Some("entry-wanted"), None).unwrap());
        let error = matches(&mut lookup, &second, Some("entry-wanted")).unwrap_err();
        let legacy_error =
            legacy_native_match(&refs, &second, Some("entry-wanted"), None).unwrap_err();
        assert_eq!(legacy_error.classify(), serde_json::error::Category::Data);
        assert_reference_json_data(error);
    }

    #[test]
    fn raw_native_lookup_errors_on_malformed_key_before_a_candidate_match() {
        let wanted = occurrence("captured:test", "/wanted");
        let bad_key = RawValue::from_string(r#"{"occurrence":"truncated"}"#.to_owned()).unwrap();
        let rows = vec![
            bad_key,
            raw_attribution(&wanted, Some("entry-wanted"), None),
        ];
        let refs = row_refs(&rows);
        let mut lookup = RawOccurrenceNativeLookup::new(&refs);
        let error = matches(&mut lookup, &wanted, Some("entry-wanted")).unwrap_err();
        let legacy_error =
            legacy_native_match(&refs, &wanted, Some("entry-wanted"), None).unwrap_err();
        assert_eq!(legacy_error.classify(), serde_json::error::Category::Data);
        assert_reference_json_data(error);
    }

    #[test]
    fn raw_native_lookup_decodes_only_matching_bodies_through_first_match() {
        let wanted = occurrence("captured:test", "/wanted");
        let other = occurrence("captured:test", "/other");
        let bad_other = RawValue::from_string(format!(
            r#"{{"occurrence":{},"contributor":null,"native_entry_id":17,"native_assertion_id":null}}"#,
            raw_key(&other)
        ))
        .unwrap();
        let bad_matching = RawValue::from_string(format!(
            r#"{{"occurrence":{},"contributor":null,"native_entry_id":17,"native_assertion_id":null}}"#,
            raw_key(&wanted)
        ))
        .unwrap();

        let rows = vec![
            bad_other,
            raw_attribution(&wanted, Some("entry-wanted"), None),
            bad_matching.clone(),
        ];
        let refs = row_refs(&rows);
        let mut lookup = RawOccurrenceNativeLookup::new(&refs);
        assert!(matches(&mut lookup, &wanted, Some("entry-wanted")).unwrap());
        assert!(legacy_native_match(&refs, &wanted, Some("entry-wanted"), None).unwrap());

        let rows = vec![bad_matching];
        let refs = row_refs(&rows);
        let mut lookup = RawOccurrenceNativeLookup::new(&refs);
        let error = matches(&mut lookup, &wanted, Some("entry-wanted")).unwrap_err();
        let legacy_error =
            legacy_native_match(&refs, &wanted, Some("entry-wanted"), None).unwrap_err();
        assert_eq!(legacy_error.classify(), serde_json::error::Category::Data);
        assert_reference_json_data(error);
    }

    #[test]
    fn raw_native_lookup_requires_both_native_ids_on_one_attribution_row() {
        let wanted = occurrence("captured:test", "/wanted");
        let split_rows = vec![
            raw_attribution(&wanted, Some("entry-wanted"), None),
            raw_attribution(&wanted, None, Some("assertion-wanted")),
        ];
        let refs = row_refs(&split_rows);
        let mut lookup = RawOccurrenceNativeLookup::new(&refs);
        assert!(
            !lookup
                .matches(
                    &wanted,
                    Some("entry-wanted"),
                    Some("assertion-wanted"),
                    PackId::Llg,
                    &"d".repeat(64),
                )
                .unwrap(),
            "the entry and assertion filters must be satisfied by one row"
        );
        assert!(
            !legacy_native_match(
                &refs,
                &wanted,
                Some("entry-wanted"),
                Some("assertion-wanted")
            )
            .unwrap()
        );

        let same_row = vec![raw_attribution(
            &wanted,
            Some("entry-wanted"),
            Some("assertion-wanted"),
        )];
        let refs = row_refs(&same_row);
        let mut lookup = RawOccurrenceNativeLookup::new(&refs);
        assert_eq!(
            lookup
                .matches(
                    &wanted,
                    Some("entry-wanted"),
                    Some("assertion-wanted"),
                    PackId::Llg,
                    &"d".repeat(64),
                )
                .unwrap(),
            legacy_native_match(
                &refs,
                &wanted,
                Some("entry-wanted"),
                Some("assertion-wanted")
            )
            .unwrap()
        );
        assert!(
            lookup
                .matches(
                    &wanted,
                    Some("entry-wanted"),
                    Some("assertion-wanted"),
                    PackId::Llg,
                    &"d".repeat(64),
                )
                .unwrap()
        );

        let refs = row_refs(&same_row);
        let mut lookup = RawOccurrenceNativeLookup::new(&refs);
        assert_eq!(
            lookup
                .matches(&wanted, None, None, PackId::Llg, &"d".repeat(64))
                .unwrap(),
            legacy_native_match(&refs, &wanted, None, None).unwrap(),
            "a native-unfiltered candidate accepts any matching occurrence row"
        );
    }
}
