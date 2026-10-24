//! Borrowed source-attribution views over immutable captured occurrences.

use crate::registry::{
    ContributorRecord, DistributorRecord, OccurrenceAttribution, RegistryError,
    SnapshotAttribution, SourceLineage, SourceRecord, SourceRegistry,
};
use crate::{CapturedRecord, Lexicon, RecordIdentity, SnapshotIdentity};

/// A borrowed captured occurrence joined to its registered source identities.
#[derive(Debug, Clone)]
pub struct AttributedOccurrence<'a> {
    record: &'a CapturedRecord,
    source: &'a SourceRecord,
    distributor: Option<&'a DistributorRecord>,
    contributor: Option<&'a ContributorRecord>,
    native_entry_id: Option<&'a str>,
    native_assertion_id: Option<&'a str>,
    lineage: Vec<&'a SourceLineage>,
}

impl<'a> AttributedOccurrence<'a> {
    /// Returns the original immutable captured record.
    #[must_use]
    pub fn record(&self) -> &'a CapturedRecord {
        self.record
    }

    /// Returns the registered asserting dictionary source.
    #[must_use]
    pub fn source(&self) -> &'a SourceRecord {
        self.source
    }

    /// Returns the registered distributor, when the snapshot identifies one.
    #[must_use]
    pub fn distributor(&self) -> Option<&'a DistributorRecord> {
        self.distributor
    }

    /// Returns the registered contributor, when this occurrence identifies one.
    #[must_use]
    pub fn contributor(&self) -> Option<&'a ContributorRecord> {
        self.contributor
    }

    /// Returns the supplied native entry ID, preserving a supplied empty string.
    #[must_use]
    pub fn native_entry_id(&self) -> Option<&'a str> {
        self.native_entry_id
    }

    /// Returns the supplied native assertion ID without treating it as a key.
    #[must_use]
    pub fn native_assertion_id(&self) -> Option<&'a str> {
        self.native_assertion_id
    }

    /// Returns all descriptive lineage rows for the asserting source.
    #[must_use]
    pub fn lineage(&self) -> &[&'a SourceLineage] {
        &self.lineage
    }
}

impl SourceRegistry {
    /// Checks registry snapshot and occurrence bindings against preserved captures.
    ///
    /// Every capture needs a source binding, and every registered binding must
    /// name a capture present in `lexicon`.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError::UnboundSnapshot`], `SnapshotNotFound`, or
    /// `OccurrenceNotFound` when either side of the relationship is absent.
    pub fn validate_lexicon(&self, lexicon: &Lexicon) -> Result<(), RegistryError> {
        let document = self.registry_document();
        let snapshots = lexicon.snapshots();

        for binding in &document.snapshots {
            if !snapshots
                .iter()
                .any(|snapshot| **snapshot == binding.snapshot)
            {
                return Err(RegistryError::SnapshotNotFound(snapshot_key(
                    &binding.snapshot,
                )));
            }
        }
        for snapshot in &snapshots {
            if !document
                .snapshots
                .iter()
                .any(|binding| binding.snapshot == **snapshot)
            {
                return Err(RegistryError::UnboundSnapshot(snapshot_key(snapshot)));
            }
        }

        for binding in &document.occurrences {
            if !lexicon
                .records()
                .iter()
                .any(|record| record.identity() == &binding.occurrence)
            {
                return Err(RegistryError::OccurrenceNotFound(record_key(
                    &binding.occurrence,
                )));
            }
        }
        Ok(())
    }

    /// Builds one borrowed attribution view for a captured record.
    ///
    /// Call with a record obtained from `Lexicon`; otherwise first use
    /// [`SourceRegistry::validate_lexicon`] to establish capture presence.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError::UnboundSnapshot`] if the record's snapshot has
    /// no explicit registry attribution.
    pub fn attribute<'a>(
        &'a self,
        record: &'a CapturedRecord,
    ) -> Result<AttributedOccurrence<'a>, RegistryError> {
        let document = self.registry_document();
        let snapshot = record.identity().snapshot();
        let snapshot_binding: &SnapshotAttribution = document
            .snapshots
            .iter()
            .find(|binding| &binding.snapshot == snapshot)
            .ok_or_else(|| RegistryError::UnboundSnapshot(snapshot_key(snapshot)))?;
        let source = document
            .sources
            .iter()
            .find(|source| source.id == snapshot_binding.source)
            .ok_or_else(|| RegistryError::UnknownSource(snapshot_binding.source.to_string()))?;
        let distributor = snapshot_binding
            .distributor
            .as_ref()
            .map(|id| {
                document
                    .distributors
                    .iter()
                    .find(|distributor| &distributor.id == id)
                    .ok_or_else(|| RegistryError::UnknownDistributor(id.to_string()))
            })
            .transpose()?;

        let occurrence: Option<&OccurrenceAttribution> = document
            .occurrences
            .iter()
            .find(|binding| binding.occurrence == *record.identity());
        let contributor = occurrence
            .and_then(|binding| binding.contributor.as_ref())
            .map(|id| {
                document
                    .contributors
                    .iter()
                    .find(|contributor| &contributor.id == id)
                    .ok_or_else(|| RegistryError::UnknownContributor(id.to_string()))
            })
            .transpose()?;
        let native_entry_id = occurrence.and_then(|binding| binding.native_entry_id.as_deref());
        let native_assertion_id =
            occurrence.and_then(|binding| binding.native_assertion_id.as_deref());
        let lineage = document
            .lineage
            .iter()
            .filter(|lineage| lineage.source == snapshot_binding.source)
            .collect();

        Ok(AttributedOccurrence {
            record,
            source,
            distributor,
            contributor,
            native_entry_id,
            native_assertion_id,
            lineage,
        })
    }

    /// Validates the full registry/capture relationship and returns all records
    /// in original insertion order, retaining every duplicate occurrence.
    ///
    /// # Errors
    ///
    /// Returns a registry validation error if a capture or binding is missing.
    pub fn attributed_records<'a>(
        &'a self,
        lexicon: &'a Lexicon,
    ) -> Result<Vec<AttributedOccurrence<'a>>, RegistryError> {
        self.validate_lexicon(lexicon)?;
        lexicon
            .records()
            .iter()
            .map(|record| self.attribute(record))
            .collect()
    }
}

fn snapshot_key(snapshot: &SnapshotIdentity) -> String {
    format!("{}:{}", snapshot.namespace(), snapshot.sha256())
}

fn record_key(record: &RecordIdentity) -> String {
    format!("{}:{}", snapshot_key(record.snapshot()), record.locator())
}
