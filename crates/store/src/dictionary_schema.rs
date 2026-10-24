//! Concrete twelve-area relational schema and projection for a validated dictionary.

use std::collections::{BTreeMap, BTreeSet};

use rusqlite::{Connection, Result, params_from_iter, types::Value};
use serde_json::Value as J;
use sha2::Digest;
use vlajmaji_lexicon::Dictionary;

pub(crate) const FORMAT: &str = "vlajmaji-dictionary";
pub(crate) const SCHEMA_VERSION: &str = "1";
pub(crate) const DATABASE_FILE: &str = "dictionary.sqlite3";

// The schema intentionally has typed role tables and ordered child tables.
// Raw native/extension values and the complete canonical envelope are the only
// opaque payloads; typed metadata is never hidden in JSON/EAV rows.
const DDL: &str = r#"
CREATE TABLE metadata(key TEXT NOT NULL PRIMARY KEY COLLATE BINARY,value TEXT NOT NULL COLLATE BINARY);
CREATE TABLE canonical_document(id INTEGER PRIMARY KEY CHECK(id=1),json BLOB NOT NULL,legacy_archive BLOB);
CREATE TABLE entity_ids(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,role TEXT NOT NULL COLLATE BINARY,ordinal INTEGER NOT NULL CHECK(ordinal>=0),UNIQUE(id,role),UNIQUE(role,ordinal));
CREATE TABLE sources(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),display_name TEXT NOT NULL COLLATE BINARY,abbreviation TEXT NOT NULL UNIQUE COLLATE BINARY,abbreviation_kind TEXT NOT NULL CHECK(abbreviation_kind IN ('Conventional','ProjectAssigned')),abbreviation_evidence TEXT COLLATE BINARY,origin TEXT NOT NULL CHECK(origin IN ('Upstream','OwnedDraft')));
CREATE TABLE contributors(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),display_name TEXT NOT NULL COLLATE BINARY);
CREATE TABLE distributors(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),display_name TEXT NOT NULL COLLATE BINARY);
CREATE TABLE source_aliases(ordinal INTEGER PRIMARY KEY CHECK(ordinal>=0),source_id TEXT NOT NULL REFERENCES sources(id) DEFERRABLE INITIALLY DEFERRED,text TEXT NOT NULL COLLATE BINARY,status TEXT NOT NULL CHECK(status IN ('Active','Retired')),evidence TEXT COLLATE BINARY);
CREATE TABLE source_lineage(ordinal INTEGER PRIMARY KEY CHECK(ordinal>=0),source_id TEXT NOT NULL REFERENCES sources(id) DEFERRABLE INITIALLY DEFERRED,target_kind TEXT NOT NULL CHECK(target_kind IN ('Known','Unresolved')),upstream_source_id TEXT REFERENCES sources(id) DEFERRABLE INITIALLY DEFERRED,unresolved_text TEXT COLLATE BINARY,evidence TEXT COLLATE BINARY,CHECK((target_kind='Known' AND upstream_source_id IS NOT NULL AND unresolved_text IS NULL) OR (target_kind='Unresolved' AND upstream_source_id IS NULL AND unresolved_text IS NOT NULL)));
CREATE TABLE capture_refs(namespace TEXT NOT NULL COLLATE BINARY,sha256 TEXT NOT NULL COLLATE BINARY,PRIMARY KEY(namespace,sha256));
CREATE TABLE snapshot_bindings(namespace TEXT NOT NULL COLLATE BINARY,sha256 TEXT NOT NULL COLLATE BINARY,ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),source_id TEXT NOT NULL REFERENCES sources(id) DEFERRABLE INITIALLY DEFERRED,distributor_id TEXT REFERENCES distributors(id) DEFERRABLE INITIALLY DEFERRED,PRIMARY KEY(namespace,sha256),FOREIGN KEY(namespace,sha256) REFERENCES capture_refs(namespace,sha256) DEFERRABLE INITIALLY DEFERRED);
CREATE TABLE occurrence_refs(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,namespace TEXT NOT NULL COLLATE BINARY,sha256 TEXT NOT NULL COLLATE BINARY,locator TEXT NOT NULL COLLATE BINARY,UNIQUE(namespace,sha256,locator),UNIQUE(id,namespace,sha256),FOREIGN KEY(namespace,sha256) REFERENCES capture_refs(namespace,sha256) DEFERRABLE INITIALLY DEFERRED);
CREATE TABLE occurrence_attributions(occurrence_id TEXT NOT NULL PRIMARY KEY COLLATE BINARY REFERENCES occurrence_refs(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),contributor_id TEXT REFERENCES contributors(id) DEFERRABLE INITIALLY DEFERRED,native_entry_id TEXT COLLATE BINARY,native_assertion_id TEXT COLLATE BINARY);
CREATE TABLE raw_captures(namespace TEXT NOT NULL COLLATE BINARY,sha256 TEXT NOT NULL COLLATE BINARY,ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),bytes BLOB NOT NULL,source_name TEXT NOT NULL COLLATE BINARY,format_label TEXT NOT NULL COLLATE BINARY,locator TEXT NOT NULL COLLATE BINARY,revision_state TEXT NOT NULL,revision_value TEXT COLLATE BINARY,captured_at_state TEXT NOT NULL,captured_at_value TEXT COLLATE BINARY,time_precision TEXT NOT NULL,notice_locator_state TEXT NOT NULL,notice_locator_value TEXT COLLATE BINARY,PRIMARY KEY(namespace,sha256),FOREIGN KEY(namespace,sha256) REFERENCES snapshot_bindings(namespace,sha256) DEFERRABLE INITIALLY DEFERRED,CHECK(revision_state IN ('missing','null','value') AND ((revision_state='value')=(revision_value IS NOT NULL))),CHECK(captured_at_state IN ('missing','null','value') AND ((captured_at_state='value')=(captured_at_value IS NOT NULL))),CHECK(notice_locator_state IN ('missing','null','value') AND ((notice_locator_state='value')=(notice_locator_value IS NOT NULL))));
CREATE TABLE raw_occurrences(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,namespace TEXT NOT NULL COLLATE BINARY,sha256 TEXT NOT NULL COLLATE BINARY,ordinal INTEGER NOT NULL CHECK(ordinal>=0),raw_line TEXT COLLATE BINARY,UNIQUE(namespace,sha256,ordinal),FOREIGN KEY(id,namespace,sha256) REFERENCES occurrence_refs(id,namespace,sha256) DEFERRABLE INITIALLY DEFERRED,FOREIGN KEY(namespace,sha256) REFERENCES raw_captures(namespace,sha256) DEFERRABLE INITIALLY DEFERRED);
CREATE TABLE native_fields(occurrence_id TEXT NOT NULL REFERENCES raw_occurrences(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),name TEXT NOT NULL COLLATE BINARY,encoding TEXT NOT NULL CHECK(encoding IN ('Json','Text')),raw_value TEXT NOT NULL COLLATE BINARY,PRIMARY KEY(occurrence_id,ordinal));
CREATE TABLE collections(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'collection' CHECK(entity_role='collection'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),source_id TEXT NOT NULL REFERENCES sources(id) DEFERRABLE INITIALLY DEFERRED,display_name TEXT NOT NULL COLLATE BINARY,description_state TEXT NOT NULL CHECK(description_state IN ('missing','null','value')),description_text TEXT COLLATE BINARY,description_language_state TEXT,description_language_value TEXT COLLATE BINARY,description_markup_state TEXT,description_markup_value TEXT COLLATE BINARY,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED,CHECK((description_state='value' AND description_text IS NOT NULL AND description_language_state IN ('missing','null','value') AND ((description_language_state='value')=(description_language_value IS NOT NULL)) AND description_markup_state IN ('missing','null','value') AND ((description_markup_state='value')=(description_markup_value IS NOT NULL))) OR (description_state IN ('missing','null') AND description_text IS NULL AND description_language_state IS NULL AND description_language_value IS NULL AND description_markup_state IS NULL AND description_markup_value IS NULL)));
CREATE TABLE components(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'component' CHECK(entity_role='component'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),collection_id TEXT NOT NULL REFERENCES collections(id) DEFERRABLE INITIALLY DEFERRED,native_name TEXT NOT NULL COLLATE BINARY,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED);
CREATE TABLE component_languages(component_id TEXT NOT NULL REFERENCES components(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),language TEXT NOT NULL COLLATE BINARY,PRIMARY KEY(component_id,ordinal));
CREATE TABLE releases(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'release' CHECK(entity_role='release'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),component_id TEXT NOT NULL REFERENCES components(id) DEFERRABLE INITIALLY DEFERRED,distributor_id TEXT REFERENCES distributors(id) DEFERRABLE INITIALLY DEFERRED,upstream_revision_state TEXT NOT NULL,upstream_revision_value TEXT COLLATE BINARY,distributor_revision_state TEXT NOT NULL,distributor_revision_value TEXT COLLATE BINARY,captured_at_state TEXT NOT NULL,captured_at_value TEXT COLLATE BINARY,time_precision TEXT NOT NULL,schema_version INTEGER NOT NULL CHECK(schema_version BETWEEN 0 AND 4294967295),normalizer TEXT NOT NULL COLLATE BINARY,normalizer_version TEXT NOT NULL COLLATE BINARY,accounting_input_occurrences TEXT,accounting_normalized_occurrences TEXT,accounting_query_excluded_occurrences TEXT,accounting_unit TEXT NOT NULL COLLATE BINARY,selected INTEGER NOT NULL CHECK(selected IN (0,1)),availability TEXT NOT NULL,redistribution TEXT NOT NULL,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED,CHECK(upstream_revision_state IN ('missing','null','value') AND ((upstream_revision_state='value')=(upstream_revision_value IS NOT NULL))),CHECK(distributor_revision_state IN ('missing','null','value') AND ((distributor_revision_state='value')=(distributor_revision_value IS NOT NULL))),CHECK(captured_at_state IN ('missing','null','value') AND ((captured_at_state='value')=(captured_at_value IS NOT NULL))),CHECK(availability IN ('Available','Disabled','Missing','Removed')),CHECK(redistribution IN ('Unknown','Held','Permitted')));
CREATE TABLE release_captures(release_id TEXT NOT NULL REFERENCES releases(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),namespace TEXT NOT NULL COLLATE BINARY,sha256 TEXT NOT NULL COLLATE BINARY,PRIMARY KEY(release_id,ordinal),UNIQUE(release_id,namespace,sha256),FOREIGN KEY(namespace,sha256) REFERENCES capture_refs(namespace,sha256) DEFERRABLE INITIALLY DEFERRED);
CREATE TABLE release_assertions(release_id TEXT NOT NULL REFERENCES releases(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),assertion_id TEXT NOT NULL REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,PRIMARY KEY(release_id,ordinal),UNIQUE(release_id,assertion_id));
CREATE TABLE release_diagnostics(release_id TEXT NOT NULL REFERENCES releases(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),occurrence_id TEXT REFERENCES occurrence_refs(id) DEFERRABLE INITIALLY DEFERRED,code TEXT NOT NULL COLLATE BINARY,message TEXT NOT NULL COLLATE BINARY,PRIMARY KEY(release_id,ordinal));
CREATE TABLE code_notices(ordinal INTEGER PRIMARY KEY CHECK(ordinal>=0),text TEXT NOT NULL COLLATE BINARY,locator_state TEXT NOT NULL,locator_value TEXT COLLATE BINARY,license_state TEXT NOT NULL,license_value TEXT COLLATE BINARY,CHECK(locator_state IN ('missing','null','value') AND ((locator_state='value')=(locator_value IS NOT NULL))),CHECK(license_state IN ('missing','null','value') AND ((license_state='value')=(license_value IS NOT NULL))));
CREATE TABLE content_notices(release_id TEXT NOT NULL REFERENCES releases(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),text_state TEXT NOT NULL,text_value TEXT COLLATE BINARY,locator_state TEXT NOT NULL,locator_value TEXT COLLATE BINARY,license_state TEXT NOT NULL,license_value TEXT COLLATE BINARY,PRIMARY KEY(release_id,ordinal),CHECK(text_state IN ('missing','null','value') AND ((text_state='value')=(text_value IS NOT NULL))),CHECK(locator_state IN ('missing','null','value') AND ((locator_state='value')=(locator_value IS NOT NULL))),CHECK(license_state IN ('missing','null','value') AND ((license_state='value')=(license_value IS NOT NULL))));
CREATE TABLE redistribution_evidence(release_id TEXT NOT NULL REFERENCES releases(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),evidence TEXT NOT NULL COLLATE BINARY,PRIMARY KEY(release_id,ordinal));
CREATE TABLE entries(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'entry' CHECK(entity_role='entry'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),source_id TEXT NOT NULL REFERENCES sources(id) DEFERRABLE INITIALLY DEFERRED,spelling_state TEXT NOT NULL CHECK(spelling_state IN ('missing','null','value')),spelling_value TEXT COLLATE BINARY,language_state TEXT NOT NULL CHECK(language_state IN ('missing','null','value')),language_value TEXT COLLATE BINARY,script_state TEXT NOT NULL CHECK(script_state IN ('missing','null','value')),script_value TEXT COLLATE BINARY,native_word_class_state TEXT NOT NULL CHECK(native_word_class_state IN ('missing','null','value')),native_word_class_value TEXT COLLATE BINARY,selmaho_state TEXT NOT NULL CHECK(selmaho_state IN ('missing','null','value')),selmaho_value TEXT COLLATE BINARY,mapped_pos_state TEXT NOT NULL CHECK(mapped_pos_state IN ('missing','null','value')),mapped_pos_value TEXT COLLATE BINARY,mapped_pos_rule TEXT COLLATE BINARY,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED,CHECK((spelling_state='value')=(spelling_value IS NOT NULL)),CHECK((language_state='value')=(language_value IS NOT NULL)),CHECK((script_state='value')=(script_value IS NOT NULL)),CHECK((native_word_class_state='value')=(native_word_class_value IS NOT NULL)),CHECK((selmaho_state='value')=(selmaho_value IS NOT NULL)),CHECK((mapped_pos_state='value' AND mapped_pos_value IS NOT NULL AND mapped_pos_rule IS NOT NULL) OR (mapped_pos_state IN ('missing','null') AND mapped_pos_value IS NULL AND mapped_pos_rule IS NULL)));
CREATE TABLE assertions(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'assertion' CHECK(entity_role='assertion'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),entry_id TEXT NOT NULL REFERENCES entries(id) DEFERRABLE INITIALLY DEFERRED,occurrence_id TEXT NOT NULL UNIQUE REFERENCES occurrence_refs(id) DEFERRABLE INITIALLY DEFERRED,native_sense_id_state TEXT NOT NULL CHECK(native_sense_id_state IN ('missing','null','value')),native_sense_id_value TEXT COLLATE BINARY,language_state TEXT NOT NULL CHECK(language_state IN ('missing','null','value')),language_value TEXT COLLATE BINARY,definition_state TEXT NOT NULL CHECK(definition_state IN ('missing','null','value')),definition_text TEXT COLLATE BINARY,definition_language_state TEXT,definition_language_value TEXT COLLATE BINARY,definition_markup_state TEXT,definition_markup_value TEXT COLLATE BINARY,source_score_state TEXT NOT NULL CHECK(source_score_state IN ('missing','null','value')),source_score_value TEXT COLLATE BINARY,eligibility TEXT NOT NULL CHECK(eligibility IN ('Lexical','Excluded')),exclusion_reason TEXT COLLATE BINARY,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED,CHECK((native_sense_id_state='value')=(native_sense_id_value IS NOT NULL)),CHECK((language_state='value')=(language_value IS NOT NULL)),CHECK((source_score_state='value')=(source_score_value IS NOT NULL)),CHECK((eligibility='Excluded' AND exclusion_reason IS NOT NULL) OR (eligibility='Lexical' AND exclusion_reason IS NULL)),CHECK((definition_state='value' AND definition_text IS NOT NULL AND definition_language_state IN ('missing','null','value') AND ((definition_language_state='value')=(definition_language_value IS NOT NULL)) AND definition_markup_state IN ('missing','null','value') AND ((definition_markup_state='value')=(definition_markup_value IS NOT NULL))) OR (definition_state IN ('missing','null') AND definition_text IS NULL AND definition_language_state IS NULL AND definition_language_value IS NULL AND definition_markup_state IS NULL AND definition_markup_value IS NULL)));
CREATE TABLE assertion_timestamps(assertion_id TEXT NOT NULL REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),field TEXT NOT NULL COLLATE BINARY,value TEXT NOT NULL COLLATE BINARY,precision TEXT NOT NULL,PRIMARY KEY(assertion_id,ordinal));
CREATE TABLE source_evidence(evidence_id INTEGER PRIMARY KEY CHECK(evidence_id>0),occurrence_id TEXT NOT NULL REFERENCES occurrence_refs(id) DEFERRABLE INITIALLY DEFERRED,field_index TEXT,citation_state TEXT NOT NULL,citation_value TEXT COLLATE BINARY,offset_start TEXT,offset_end TEXT,CHECK((offset_start IS NULL)=(offset_end IS NULL)),CHECK(citation_state IN ('missing','null','value') AND ((citation_state='value' AND citation_value IS NOT NULL) OR (citation_state IN ('missing','null') AND citation_value IS NULL))),CHECK((offset_start IS NULL AND offset_end IS NULL) OR (offset_start IS NOT NULL AND offset_end IS NOT NULL)),CHECK((field_index IS NULL) OR (length(field_index)>0 AND field_index NOT GLOB '*[^0-9]*' AND (field_index='0' OR substr(field_index,1,1)!='0') AND (length(field_index)<20 OR (length(field_index)=20 AND field_index<='18446744073709551615')))),CHECK((offset_start IS NULL) OR (length(offset_start)>0 AND offset_start NOT GLOB '*[^0-9]*' AND (offset_start='0' OR substr(offset_start,1,1)!='0') AND length(offset_start)<=20 AND (length(offset_start)<20 OR offset_start<='18446744073709551615'))),CHECK((offset_end IS NULL) OR (length(offset_end)>0 AND offset_end NOT GLOB '*[^0-9]*' AND (offset_end='0' OR substr(offset_end,1,1)!='0') AND length(offset_end)<=20 AND (length(offset_end)<20 OR offset_end<='18446744073709551615'))));
CREATE TABLE contexts(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'context' CHECK(entity_role='context'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),assertion_id TEXT NOT NULL REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,kind TEXT NOT NULL,value_state TEXT NOT NULL,value_text TEXT COLLATE BINARY,value_language_state TEXT,value_language_value TEXT COLLATE BINARY,value_markup_state TEXT,value_markup_value TEXT COLLATE BINARY,translated_from TEXT REFERENCES contexts(id) DEFERRABLE INITIALLY DEFERRED,evidence_id INTEGER NOT NULL REFERENCES source_evidence(evidence_id) DEFERRABLE INITIALLY DEFERRED,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED,CHECK(kind IN ('Gloss','Note','Label','Register','Domain','Example','Translation','Citation')),CHECK((value_state IN ('missing','null') AND value_text IS NULL AND value_language_state IS NULL AND value_language_value IS NULL AND value_markup_state IS NULL AND value_markup_value IS NULL) OR (value_state='value' AND value_text IS NOT NULL AND value_language_state IS NOT NULL AND value_language_state IN ('missing','null','value') AND ((value_language_state='value' AND value_language_value IS NOT NULL) OR (value_language_state IN ('missing','null') AND value_language_value IS NULL)) AND value_markup_state IS NOT NULL AND value_markup_state IN ('missing','null','value') AND ((value_markup_state='value' AND value_markup_value IS NOT NULL) OR (value_markup_state IN ('missing','null') AND value_markup_value IS NULL)))));
CREATE TABLE argument_frames(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'frame' CHECK(entity_role='frame'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),assertion_id TEXT NOT NULL REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,evidence_id INTEGER NOT NULL REFERENCES source_evidence(evidence_id) DEFERRABLE INITIALLY DEFERRED,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED);
CREATE TABLE argument_slots(frame_id TEXT NOT NULL REFERENCES argument_frames(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),slot_order INTEGER NOT NULL,native_label TEXT NOT NULL COLLATE BINARY,description_state TEXT NOT NULL,description_text TEXT COLLATE BINARY,description_language_state TEXT,description_language_value TEXT COLLATE BINARY,description_markup_state TEXT,description_markup_value TEXT COLLATE BINARY,grammatical_role_state TEXT NOT NULL,grammatical_role_value TEXT COLLATE BINARY,PRIMARY KEY(frame_id,ordinal),UNIQUE(frame_id,slot_order),CHECK(slot_order BETWEEN 0 AND 4294967295),CHECK((description_state IN ('missing','null') AND description_text IS NULL AND description_language_state IS NULL AND description_language_value IS NULL AND description_markup_state IS NULL AND description_markup_value IS NULL) OR (description_state='value' AND description_text IS NOT NULL AND description_language_state IS NOT NULL AND description_language_state IN ('missing','null','value') AND ((description_language_state='value' AND description_language_value IS NOT NULL) OR (description_language_state IN ('missing','null') AND description_language_value IS NULL)) AND description_markup_state IS NOT NULL AND description_markup_state IN ('missing','null','value') AND ((description_markup_state='value' AND description_markup_value IS NOT NULL) OR (description_markup_state IN ('missing','null') AND description_markup_value IS NULL)))),CHECK(grammatical_role_state IN ('missing','null','value') AND ((grammatical_role_state='value' AND grammatical_role_value IS NOT NULL) OR (grammatical_role_state IN ('missing','null') AND grammatical_role_value IS NULL))));
CREATE TABLE forms(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'form' CHECK(entity_role='form'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),entry_id TEXT NOT NULL REFERENCES entries(id) DEFERRABLE INITIALLY DEFERRED,spelling TEXT NOT NULL COLLATE BINARY,language_state TEXT NOT NULL,language_value TEXT COLLATE BINARY,script_state TEXT NOT NULL,script_value TEXT COLLATE BINARY,pronunciation_state TEXT NOT NULL,pronunciation_text TEXT COLLATE BINARY,pronunciation_language_state TEXT,pronunciation_language_value TEXT COLLATE BINARY,pronunciation_markup_state TEXT,pronunciation_markup_value TEXT COLLATE BINARY,kind TEXT NOT NULL,standing_state TEXT NOT NULL,standing_value TEXT COLLATE BINARY,evidence_id INTEGER REFERENCES source_evidence(evidence_id) DEFERRABLE INITIALLY DEFERRED,computation_id TEXT REFERENCES computations(id) DEFERRABLE INITIALLY DEFERRED,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED,CHECK(language_state IN ('missing','null','value') AND ((language_state='value')=(language_value IS NOT NULL))),CHECK(script_state IN ('missing','null','value') AND ((script_state='value')=(script_value IS NOT NULL))),CHECK(standing_state IN ('missing','null','value') AND ((standing_state='value')=(standing_value IS NOT NULL))),CHECK(kind IN ('SourceVariant','SourcePronunciation','SourceReportedForm','AssignedRafsi','ComputedStem','ComputedComponents','GeneratedCandidate')),CHECK((kind IN ('SourceVariant','SourcePronunciation','SourceReportedForm','AssignedRafsi') AND evidence_id IS NOT NULL AND computation_id IS NULL) OR (kind IN ('ComputedStem','ComputedComponents','GeneratedCandidate') AND evidence_id IS NULL AND computation_id IS NOT NULL)),CHECK(pronunciation_state IN ('missing','null','value') AND ((pronunciation_state IN ('missing','null') AND pronunciation_text IS NULL AND pronunciation_language_state IS NULL AND pronunciation_language_value IS NULL AND pronunciation_markup_state IS NULL AND pronunciation_markup_value IS NULL) OR (pronunciation_state='value' AND pronunciation_text IS NOT NULL AND pronunciation_language_state IS NOT NULL AND pronunciation_language_state IN ('missing','null','value') AND ((pronunciation_language_state='value' AND pronunciation_language_value IS NOT NULL) OR (pronunciation_language_state IN ('missing','null') AND pronunciation_language_value IS NULL)) AND pronunciation_markup_state IS NOT NULL AND pronunciation_markup_state IN ('missing','null','value') AND ((pronunciation_markup_state='value' AND pronunciation_markup_value IS NOT NULL) OR (pronunciation_markup_state IN ('missing','null') AND pronunciation_markup_value IS NULL))))));
CREATE TABLE form_components(form_id TEXT NOT NULL REFERENCES forms(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),component_order INTEGER NOT NULL CHECK(component_order BETWEEN 0 AND 4294967295),spelling TEXT NOT NULL COLLATE BINARY,target_kind TEXT,target_entry_id TEXT REFERENCES entries(id) DEFERRABLE INITIALLY DEFERRED,target_assertion_id TEXT REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,target_form_id TEXT REFERENCES forms(id) DEFERRABLE INITIALLY DEFERRED,target_editorial_sense_id TEXT REFERENCES editorial_senses(id) DEFERRABLE INITIALLY DEFERRED,PRIMARY KEY(form_id,ordinal),UNIQUE(form_id,component_order),CHECK(component_order BETWEEN 0 AND 4294967295),CHECK((target_kind IS NULL AND target_entry_id IS NULL AND target_assertion_id IS NULL AND target_form_id IS NULL AND target_editorial_sense_id IS NULL) OR (target_kind='Entry' AND target_entry_id IS NOT NULL AND target_assertion_id IS NULL AND target_form_id IS NULL AND target_editorial_sense_id IS NULL) OR (target_kind='Assertion' AND target_entry_id IS NULL AND target_assertion_id IS NOT NULL AND target_form_id IS NULL AND target_editorial_sense_id IS NULL) OR (target_kind='Form' AND target_entry_id IS NULL AND target_assertion_id IS NULL AND target_form_id IS NOT NULL AND target_editorial_sense_id IS NULL) OR (target_kind='EditorialSense' AND target_entry_id IS NULL AND target_assertion_id IS NULL AND target_form_id IS NULL AND target_editorial_sense_id IS NOT NULL)));
CREATE TABLE etymologies(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'etymology' CHECK(entity_role='etymology'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),assertion_id TEXT NOT NULL REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,statement_state TEXT NOT NULL,statement_text TEXT COLLATE BINARY,statement_language_state TEXT,statement_language_value TEXT COLLATE BINARY,statement_markup_state TEXT,statement_markup_value TEXT COLLATE BINARY,native_kind_state TEXT NOT NULL,native_kind_value TEXT COLLATE BINARY,evidence_id INTEGER NOT NULL REFERENCES source_evidence(evidence_id) DEFERRABLE INITIALLY DEFERRED,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED,CHECK((statement_state IN ('missing','null') AND statement_text IS NULL AND statement_language_state IS NULL AND statement_language_value IS NULL AND statement_markup_state IS NULL AND statement_markup_value IS NULL) OR (statement_state='value' AND statement_text IS NOT NULL AND statement_language_state IS NOT NULL AND statement_language_state IN ('missing','null','value') AND ((statement_language_state='value' AND statement_language_value IS NOT NULL) OR (statement_language_state IN ('missing','null') AND statement_language_value IS NULL)) AND statement_markup_state IS NOT NULL AND statement_markup_state IN ('missing','null','value') AND ((statement_markup_state='value' AND statement_markup_value IS NOT NULL) OR (statement_markup_state IN ('missing','null') AND statement_markup_value IS NULL)))),CHECK(native_kind_state IN ('missing','null','value') AND ((native_kind_state='value' AND native_kind_value IS NOT NULL) OR (native_kind_state IN ('missing','null') AND native_kind_value IS NULL))));
CREATE TABLE etymology_components(etymology_id TEXT NOT NULL REFERENCES etymologies(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),component_order INTEGER NOT NULL CHECK(component_order BETWEEN 0 AND 4294967295),spelling TEXT NOT NULL COLLATE BINARY,target_kind TEXT,target_entry_id TEXT REFERENCES entries(id) DEFERRABLE INITIALLY DEFERRED,target_assertion_id TEXT REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,target_form_id TEXT REFERENCES forms(id) DEFERRABLE INITIALLY DEFERRED,target_editorial_sense_id TEXT REFERENCES editorial_senses(id) DEFERRABLE INITIALLY DEFERRED,PRIMARY KEY(etymology_id,ordinal),UNIQUE(etymology_id,component_order),CHECK(component_order BETWEEN 0 AND 4294967295),CHECK((target_kind IS NULL AND target_entry_id IS NULL AND target_assertion_id IS NULL AND target_form_id IS NULL AND target_editorial_sense_id IS NULL) OR (target_kind='Entry' AND target_entry_id IS NOT NULL AND target_assertion_id IS NULL AND target_form_id IS NULL AND target_editorial_sense_id IS NULL) OR (target_kind='Assertion' AND target_entry_id IS NULL AND target_assertion_id IS NOT NULL AND target_form_id IS NULL AND target_editorial_sense_id IS NULL) OR (target_kind='Form' AND target_entry_id IS NULL AND target_assertion_id IS NULL AND target_form_id IS NOT NULL AND target_editorial_sense_id IS NULL) OR (target_kind='EditorialSense' AND target_entry_id IS NULL AND target_assertion_id IS NULL AND target_form_id IS NULL AND target_editorial_sense_id IS NOT NULL)));
CREATE TABLE relations(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'relation' CHECK(entity_role='relation'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),origin_kind TEXT NOT NULL,origin_entry_id TEXT REFERENCES entries(id) DEFERRABLE INITIALLY DEFERRED,origin_assertion_id TEXT REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,origin_form_id TEXT REFERENCES forms(id) DEFERRABLE INITIALLY DEFERRED,origin_editorial_sense_id TEXT REFERENCES editorial_senses(id) DEFERRABLE INITIALLY DEFERRED,kind TEXT NOT NULL,other_kind TEXT COLLATE BINARY,direction TEXT NOT NULL,scope TEXT NOT NULL,target_kind TEXT NOT NULL,local_kind TEXT,local_entry_id TEXT REFERENCES entries(id) DEFERRABLE INITIALLY DEFERRED,local_assertion_id TEXT REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,local_form_id TEXT REFERENCES forms(id) DEFERRABLE INITIALLY DEFERRED,local_editorial_sense_id TEXT REFERENCES editorial_senses(id) DEFERRABLE INITIALLY DEFERRED,catalog_source_id TEXT REFERENCES sources(id) DEFERRABLE INITIALLY DEFERRED,catalog_release_id TEXT REFERENCES releases(id) DEFERRABLE INITIALLY DEFERRED,catalog_native_entry_id TEXT COLLATE BINARY,catalog_native_assertion_id TEXT COLLATE BINARY,catalog_occurrence_id TEXT REFERENCES occurrence_refs(id) DEFERRABLE INITIALLY DEFERRED,unresolved_text TEXT COLLATE BINARY,evidence_id INTEGER REFERENCES source_evidence(evidence_id) DEFERRABLE INITIALLY DEFERRED,editorial_revision_id TEXT REFERENCES owned_revisions(id) DEFERRABLE INITIALLY DEFERRED,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED,CHECK((origin_kind='Entry' AND origin_entry_id IS NOT NULL AND origin_assertion_id IS NULL AND origin_form_id IS NULL AND origin_editorial_sense_id IS NULL) OR (origin_kind='Assertion' AND origin_entry_id IS NULL AND origin_assertion_id IS NOT NULL AND origin_form_id IS NULL AND origin_editorial_sense_id IS NULL) OR (origin_kind='Form' AND origin_entry_id IS NULL AND origin_assertion_id IS NULL AND origin_form_id IS NOT NULL AND origin_editorial_sense_id IS NULL) OR (origin_kind='EditorialSense' AND origin_entry_id IS NULL AND origin_assertion_id IS NULL AND origin_form_id IS NULL AND origin_editorial_sense_id IS NOT NULL)),CHECK((target_kind='Local' AND local_kind IN ('Entry','Assertion','Form','EditorialSense') AND ((local_kind='Entry' AND local_entry_id IS NOT NULL AND local_assertion_id IS NULL AND local_form_id IS NULL AND local_editorial_sense_id IS NULL) OR (local_kind='Assertion' AND local_entry_id IS NULL AND local_assertion_id IS NOT NULL AND local_form_id IS NULL AND local_editorial_sense_id IS NULL) OR (local_kind='Form' AND local_entry_id IS NULL AND local_assertion_id IS NULL AND local_form_id IS NOT NULL AND local_editorial_sense_id IS NULL) OR (local_kind='EditorialSense' AND local_entry_id IS NULL AND local_assertion_id IS NULL AND local_form_id IS NULL AND local_editorial_sense_id IS NOT NULL)) AND catalog_source_id IS NULL AND catalog_release_id IS NULL AND catalog_native_entry_id IS NULL AND catalog_native_assertion_id IS NULL AND catalog_occurrence_id IS NULL AND unresolved_text IS NULL) OR (target_kind='Catalog' AND local_kind IS NULL AND local_entry_id IS NULL AND local_assertion_id IS NULL AND local_form_id IS NULL AND local_editorial_sense_id IS NULL AND catalog_source_id IS NOT NULL AND catalog_release_id IS NOT NULL AND unresolved_text IS NULL) OR (target_kind='Unknown' AND local_kind IS NULL AND local_entry_id IS NULL AND local_assertion_id IS NULL AND local_form_id IS NULL AND local_editorial_sense_id IS NULL AND catalog_source_id IS NULL AND catalog_release_id IS NULL AND catalog_native_entry_id IS NULL AND catalog_native_assertion_id IS NULL AND catalog_occurrence_id IS NULL AND unresolved_text IS NULL) OR (target_kind='Unresolved' AND local_kind IS NULL AND local_entry_id IS NULL AND local_assertion_id IS NULL AND local_form_id IS NULL AND local_editorial_sense_id IS NULL AND catalog_source_id IS NULL AND catalog_release_id IS NULL AND catalog_native_entry_id IS NULL AND catalog_native_assertion_id IS NULL AND catalog_occurrence_id IS NULL AND unresolved_text IS NOT NULL)),CHECK((kind IN ('SourceSeeAlso','SourceEtymology') AND evidence_id IS NOT NULL AND editorial_revision_id IS NULL) OR (kind='EditorialLink' AND evidence_id IS NULL AND editorial_revision_id IS NOT NULL) OR (kind='Other' AND NOT (evidence_id IS NOT NULL AND editorial_revision_id IS NOT NULL))));
CREATE TABLE owned_revisions(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'owned_revision' CHECK(entity_role='owned_revision'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),source_id TEXT NOT NULL REFERENCES sources(id) DEFERRABLE INITIALLY DEFERRED,parent_id TEXT REFERENCES owned_revisions(id) DEFERRABLE INITIALLY DEFERRED,editor_id TEXT NOT NULL REFERENCES contributors(id) DEFERRABLE INITIALLY DEFERRED,timestamp_state TEXT NOT NULL,timestamp_value TEXT COLLATE BINARY,time_precision TEXT NOT NULL,rationale_state TEXT NOT NULL,rationale_text TEXT COLLATE BINARY,rationale_language_state TEXT,rationale_language_value TEXT COLLATE BINARY,rationale_markup_state TEXT,rationale_markup_value TEXT COLLATE BINARY,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED,CHECK(timestamp_state IN ('missing','null','value') AND ((timestamp_state='value' AND timestamp_value IS NOT NULL) OR (timestamp_state IN ('missing','null') AND timestamp_value IS NULL))),CHECK((rationale_state IN ('missing','null') AND rationale_text IS NULL AND rationale_language_state IS NULL AND rationale_language_value IS NULL AND rationale_markup_state IS NULL AND rationale_markup_value IS NULL) OR (rationale_state='value' AND rationale_text IS NOT NULL AND rationale_language_state IS NOT NULL AND rationale_language_state IN ('missing','null','value') AND ((rationale_language_state='value' AND rationale_language_value IS NOT NULL) OR (rationale_language_state IN ('missing','null') AND rationale_language_value IS NULL)) AND rationale_markup_state IS NOT NULL AND rationale_markup_state IN ('missing','null','value') AND ((rationale_markup_state='value' AND rationale_markup_value IS NOT NULL) OR (rationale_markup_state IN ('missing','null') AND rationale_markup_value IS NULL)))));
CREATE TABLE revision_assertions(revision_id TEXT NOT NULL REFERENCES owned_revisions(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),assertion_id TEXT NOT NULL REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,PRIMARY KEY(revision_id,ordinal),UNIQUE(revision_id,assertion_id));
CREATE TABLE revision_tombstones(revision_id TEXT NOT NULL REFERENCES owned_revisions(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),assertion_id TEXT NOT NULL REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,PRIMARY KEY(revision_id,ordinal),UNIQUE(revision_id,assertion_id));
CREATE TABLE owned_heads(source_id TEXT NOT NULL PRIMARY KEY REFERENCES sources(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),revision_id TEXT NOT NULL REFERENCES owned_revisions(id) DEFERRABLE INITIALLY DEFERRED);
CREATE TABLE editorial_senses(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'editorial_sense' CHECK(entity_role='editorial_sense'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),editor_id TEXT NOT NULL REFERENCES contributors(id) DEFERRABLE INITIALLY DEFERRED,revision_id TEXT NOT NULL REFERENCES owned_revisions(id) DEFERRABLE INITIALLY DEFERRED,rationale_text TEXT NOT NULL COLLATE BINARY,rationale_language_state TEXT NOT NULL,rationale_language_value TEXT COLLATE BINARY,rationale_markup_state TEXT NOT NULL,rationale_markup_value TEXT COLLATE BINARY,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED,CHECK(rationale_language_state IN ('missing','null','value') AND ((rationale_language_state='value' AND rationale_language_value IS NOT NULL) OR (rationale_language_state IN ('missing','null') AND rationale_language_value IS NULL))),CHECK(rationale_markup_state IN ('missing','null','value') AND ((rationale_markup_state='value' AND rationale_markup_value IS NOT NULL) OR (rationale_markup_state IN ('missing','null') AND rationale_markup_value IS NULL))));
CREATE TABLE editorial_members(editorial_sense_id TEXT NOT NULL REFERENCES editorial_senses(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),assertion_id TEXT NOT NULL REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,PRIMARY KEY(editorial_sense_id,ordinal));
CREATE TABLE computations(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'computation' CHECK(entity_role='computation'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),tool TEXT NOT NULL COLLATE BINARY,tool_version TEXT NOT NULL COLLATE BINARY,policy TEXT NOT NULL COLLATE BINARY,profile_state TEXT NOT NULL,profile_value TEXT COLLATE BINARY,explanation_state TEXT NOT NULL,explanation_text TEXT COLLATE BINARY,explanation_language_state TEXT,explanation_language_value TEXT COLLATE BINARY,explanation_markup_state TEXT,explanation_markup_value TEXT COLLATE BINARY,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED,CHECK(profile_state IN ('missing','null','value') AND ((profile_state='value' AND profile_value IS NOT NULL) OR (profile_state IN ('missing','null') AND profile_value IS NULL))),CHECK((explanation_state IN ('missing','null') AND explanation_text IS NULL AND explanation_language_state IS NULL AND explanation_language_value IS NULL AND explanation_markup_state IS NULL AND explanation_markup_value IS NULL) OR (explanation_state='value' AND explanation_text IS NOT NULL AND explanation_language_state IS NOT NULL AND explanation_language_state IN ('missing','null','value') AND ((explanation_language_state='value' AND explanation_language_value IS NOT NULL) OR (explanation_language_state IN ('missing','null') AND explanation_language_value IS NULL)) AND explanation_markup_state IS NOT NULL AND explanation_markup_state IN ('missing','null','value') AND ((explanation_markup_state='value' AND explanation_markup_value IS NOT NULL) OR (explanation_markup_state IN ('missing','null') AND explanation_markup_value IS NULL)))));
CREATE TABLE computation_inputs(computation_id TEXT NOT NULL REFERENCES computations(id) DEFERRABLE INITIALLY DEFERRED,ordinal INTEGER NOT NULL CHECK(ordinal>=0),input_kind TEXT NOT NULL,occurrence_id TEXT REFERENCES occurrence_refs(id) DEFERRABLE INITIALLY DEFERRED,entity_kind TEXT,entity_entry_id TEXT REFERENCES entries(id) DEFERRABLE INITIALLY DEFERRED,entity_assertion_id TEXT REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,entity_form_id TEXT REFERENCES forms(id) DEFERRABLE INITIALLY DEFERRED,entity_editorial_sense_id TEXT REFERENCES editorial_senses(id) DEFERRABLE INITIALLY DEFERRED,literal TEXT COLLATE BINARY,PRIMARY KEY(computation_id,ordinal),CHECK((input_kind='Occurrence' AND occurrence_id IS NOT NULL AND entity_kind IS NULL AND entity_entry_id IS NULL AND entity_assertion_id IS NULL AND entity_form_id IS NULL AND entity_editorial_sense_id IS NULL AND literal IS NULL) OR (input_kind='Entity' AND occurrence_id IS NULL AND entity_kind IN ('Entry','Assertion','Form','EditorialSense') AND literal IS NULL) OR (input_kind='Literal' AND occurrence_id IS NULL AND entity_kind IS NULL AND entity_entry_id IS NULL AND entity_assertion_id IS NULL AND entity_form_id IS NULL AND entity_editorial_sense_id IS NULL AND literal IS NOT NULL)),CHECK((input_kind='Occurrence' AND occurrence_id IS NOT NULL AND entity_kind IS NULL AND entity_entry_id IS NULL AND entity_assertion_id IS NULL AND entity_form_id IS NULL AND entity_editorial_sense_id IS NULL AND literal IS NULL) OR (input_kind='Entity' AND occurrence_id IS NULL AND entity_kind IN ('Entry','Assertion','Form','EditorialSense') AND ((entity_kind='Entry' AND entity_entry_id IS NOT NULL AND entity_assertion_id IS NULL AND entity_form_id IS NULL AND entity_editorial_sense_id IS NULL) OR (entity_kind='Assertion' AND entity_entry_id IS NULL AND entity_assertion_id IS NOT NULL AND entity_form_id IS NULL AND entity_editorial_sense_id IS NULL) OR (entity_kind='Form' AND entity_entry_id IS NULL AND entity_assertion_id IS NULL AND entity_form_id IS NOT NULL AND entity_editorial_sense_id IS NULL) OR (entity_kind='EditorialSense' AND entity_entry_id IS NULL AND entity_assertion_id IS NULL AND entity_form_id IS NULL AND entity_editorial_sense_id IS NOT NULL)) AND literal IS NULL) OR (input_kind='Literal' AND occurrence_id IS NULL AND entity_kind IS NULL AND entity_entry_id IS NULL AND entity_assertion_id IS NULL AND entity_form_id IS NULL AND entity_editorial_sense_id IS NULL AND literal IS NOT NULL)));
CREATE TABLE annotations(id TEXT NOT NULL PRIMARY KEY COLLATE BINARY,entity_role TEXT NOT NULL DEFAULT 'annotation' CHECK(entity_role='annotation'),ordinal INTEGER NOT NULL UNIQUE CHECK(ordinal>=0),computation_id TEXT NOT NULL REFERENCES computations(id) DEFERRABLE INITIALLY DEFERRED,target_kind TEXT NOT NULL,target_entry_id TEXT REFERENCES entries(id) DEFERRABLE INITIALLY DEFERRED,target_assertion_id TEXT REFERENCES assertions(id) DEFERRABLE INITIALLY DEFERRED,target_form_id TEXT REFERENCES forms(id) DEFERRABLE INITIALLY DEFERRED,target_editorial_sense_id TEXT REFERENCES editorial_senses(id) DEFERRABLE INITIALLY DEFERRED,kind TEXT NOT NULL COLLATE BINARY,value TEXT NOT NULL COLLATE BINARY,explanation_state TEXT NOT NULL,explanation_text TEXT COLLATE BINARY,explanation_language_state TEXT,explanation_language_value TEXT COLLATE BINARY,explanation_markup_state TEXT,explanation_markup_value TEXT COLLATE BINARY,UNIQUE(id,entity_role),FOREIGN KEY(id,entity_role) REFERENCES entity_ids(id,role) DEFERRABLE INITIALLY DEFERRED,CHECK((target_kind='Entry' AND target_entry_id IS NOT NULL AND target_assertion_id IS NULL AND target_form_id IS NULL AND target_editorial_sense_id IS NULL) OR (target_kind='Assertion' AND target_entry_id IS NULL AND target_assertion_id IS NOT NULL AND target_form_id IS NULL AND target_editorial_sense_id IS NULL) OR (target_kind='Form' AND target_entry_id IS NULL AND target_assertion_id IS NULL AND target_form_id IS NOT NULL AND target_editorial_sense_id IS NULL) OR (target_kind='EditorialSense' AND target_entry_id IS NULL AND target_assertion_id IS NULL AND target_form_id IS NULL AND target_editorial_sense_id IS NOT NULL)),CHECK((explanation_state IN ('missing','null') AND explanation_text IS NULL AND explanation_language_state IS NULL AND explanation_language_value IS NULL AND explanation_markup_state IS NULL AND explanation_markup_value IS NULL) OR (explanation_state='value' AND explanation_text IS NOT NULL AND explanation_language_state IS NOT NULL AND explanation_language_state IN ('missing','null','value') AND ((explanation_language_state='value' AND explanation_language_value IS NOT NULL) OR (explanation_language_state IN ('missing','null') AND explanation_language_value IS NULL)) AND explanation_markup_state IS NOT NULL AND explanation_markup_state IN ('missing','null','value') AND ((explanation_markup_state='value' AND explanation_markup_value IS NOT NULL) OR (explanation_markup_state IN ('missing','null') AND explanation_markup_value IS NULL)))));
CREATE TABLE extensions(ordinal INTEGER PRIMARY KEY CHECK(ordinal>=0),owner_id TEXT NOT NULL REFERENCES entity_ids(id) DEFERRABLE INITIALLY DEFERRED,name TEXT NOT NULL COLLATE BINARY,value TEXT NOT NULL COLLATE BINARY,evidence_id INTEGER REFERENCES source_evidence(evidence_id) DEFERRABLE INITIALLY DEFERRED);
CREATE INDEX idx_entries_spelling ON entries(spelling_value COLLATE BINARY);
CREATE INDEX idx_entries_source_spelling ON entries(source_id,spelling_value COLLATE BINARY);
CREATE INDEX idx_assertions_entry ON assertions(entry_id);
CREATE INDEX idx_release_assertions_assertion ON release_assertions(assertion_id);
"#;

pub(crate) type Rows = BTreeMap<&'static str, Vec<Vec<Value>>>;
type SchemaObject = (String, String, String, Option<String>);

pub(crate) fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(DDL)
}

pub(crate) fn schema_sql(conn: &Connection) -> Result<Vec<SchemaObject>> {
    let mut stmt = conn.prepare("SELECT type,name,tbl_name,sql FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name")?;
    stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect()
}

pub(crate) fn project(dictionary: &Dictionary, canonical: &[u8]) -> Result<Rows> {
    let doc = serde_json::to_value(dictionary.document())
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    let registry = serde_json::to_value(dictionary.registry().document())
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    let mut out = Rows::new();
    let mut add = |table: &'static str, row: Vec<Value>| out.entry(table).or_default().push(row);
    let version = doc["version"].as_u64().unwrap_or(0);
    let catalogue_revision = doc["metadata"]["catalogue_revision"].as_u64().unwrap_or(0);
    let registry_version = registry["version"].as_u64().unwrap_or(0);
    let legacy = opt_bytes(&doc["legacy_archive"]);
    let digest = format!("{:x}", sha2::Sha256::digest(canonical));
    for (key, value) in [
        ("format", FORMAT.to_owned()),
        ("schema_version", SCHEMA_VERSION.to_owned()),
        ("complete", "1".to_owned()),
        ("canonical_sha256", digest),
        ("dictionary_version", version.to_string()),
        ("registry_version", registry_version.to_string()),
        ("catalogue_revision", catalogue_revision.to_string()),
    ] {
        add("metadata", vec![t(key), t(value)]);
    }
    add(
        "canonical_document",
        vec![i(1), b(canonical.to_vec()), legacy],
    );

    let sources = arr(&registry["sources"]);
    let contributors = arr(&registry["contributors"]);
    let distributors = arr(&registry["distributors"]);
    for (n, x) in sources.iter().enumerate() {
        add(
            "sources",
            vec![
                t(strv(&x["id"])),
                i(n as i64),
                t(strv(&x["display_name"])),
                t(strv(&x["abbreviation"])),
                t(enumv(&x["abbreviation_kind"])),
                opt_text(&x["abbreviation_evidence"]),
                t(enumv(&x["origin"])),
            ],
        );
    }
    for (n, x) in contributors.iter().enumerate() {
        add(
            "contributors",
            vec![t(strv(&x["id"])), i(n as i64), t(strv(&x["display_name"]))],
        );
    }
    for (n, x) in distributors.iter().enumerate() {
        add(
            "distributors",
            vec![t(strv(&x["id"])), i(n as i64), t(strv(&x["display_name"]))],
        );
    }
    for (n, x) in arr(&registry["aliases"]).iter().enumerate() {
        add(
            "source_aliases",
            vec![
                i(n as i64),
                t(strv(&x["source"])),
                t(strv(&x["text"])),
                t(enumv(&x["status"])),
                opt_text(&x["evidence"]),
            ],
        );
    }
    for (n, x) in arr(&registry["lineage"]).iter().enumerate() {
        let (kind, known, unresolved) = if let Some(v) = x["upstream"].get("Known") {
            ("Known", Some(strv(v)), None)
        } else {
            (
                "Unresolved",
                None,
                x["upstream"].get("Unresolved").map(strv),
            )
        };
        add(
            "source_lineage",
            vec![
                i(n as i64),
                t(strv(&x["source"])),
                t(kind),
                known.map(t).unwrap_or(Value::Null),
                unresolved.map(t).unwrap_or(Value::Null),
                opt_text(&x["evidence"]),
            ],
        );
    }

    let mut captures = BTreeSet::<(String, String)>::new();
    for x in arr(&doc["captures"]) {
        captures.insert((
            strv(&x["identity"]["namespace"]),
            strv(&x["identity"]["sha256"]),
        ));
    }
    for x in arr(&registry["snapshots"]) {
        captures.insert((
            strv(&x["snapshot"]["namespace"]),
            strv(&x["snapshot"]["sha256"]),
        ));
    }
    for x in arr(&doc["metadata"]["releases"]) {
        for c in arr(&x["captures"]) {
            captures.insert((strv(&c["namespace"]), strv(&c["sha256"])));
        }
    }
    for x in arr(&doc["metadata"]["releases"]) {
        for d in arr(&x["diagnostics"]) {
            if d["occurrence"].is_object() {
                captures.insert((
                    strv(&d["occurrence"]["capture"]["namespace"]),
                    strv(&d["occurrence"]["capture"]["sha256"]),
                ));
            }
        }
    }
    for key in [
        "contexts",
        "frames",
        "forms",
        "etymologies",
        "relations",
        "extensions",
    ] {
        for x in arr(&doc["metadata"][key]) {
            let ev = &x["evidence"];
            if ev.is_object() {
                captures.insert((
                    strv(&ev["occurrence"]["capture"]["namespace"]),
                    strv(&ev["occurrence"]["capture"]["sha256"]),
                ));
            }
            if key == "relations" {
                let c = &x["target"]["Catalog"];
                if c.is_object() && c["occurrence"].is_object() {
                    captures.insert((
                        strv(&c["occurrence"]["capture"]["namespace"]),
                        strv(&c["occurrence"]["capture"]["sha256"]),
                    ));
                }
            }
        }
    }
    for (ns, sha) in &captures {
        add("capture_refs", vec![t(ns), t(sha)]);
    }
    let mut snapshot_ord = BTreeMap::new();
    for (n, x) in arr(&registry["snapshots"]).iter().enumerate() {
        let ns = strv(&x["snapshot"]["namespace"]);
        let sha = strv(&x["snapshot"]["sha256"]);
        snapshot_ord.insert((ns.clone(), sha.clone()), n as i64);
        add(
            "snapshot_bindings",
            vec![
                t(ns),
                t(sha),
                i(n as i64),
                t(strv(&x["source"])),
                opt_text(&x["distributor"]),
            ],
        );
    }
    let mut occurrences = BTreeMap::<String, (String, String, String)>::new();
    for x in arr(&doc["captures"]) {
        let ns = strv(&x["identity"]["namespace"]);
        let sha = strv(&x["identity"]["sha256"]);
        for row in arr(&x["occurrences"]) {
            let loc = strv(&row["locator"]);
            let id = record_id(&ns, &sha, &loc);
            occurrences.insert(id, (ns.clone(), sha.clone(), loc));
        }
    }
    for x in arr(&registry["occurrences"]) {
        let id = record_id(
            &strv(&x["occurrence"]["snapshot"]["namespace"]),
            &strv(&x["occurrence"]["snapshot"]["sha256"]),
            &strv(&x["occurrence"]["locator"]),
        );
        occurrences.entry(id).or_insert_with(|| {
            (
                strv(&x["occurrence"]["snapshot"]["namespace"]),
                strv(&x["occurrence"]["snapshot"]["sha256"]),
                strv(&x["occurrence"]["locator"]),
            )
        });
    }
    for x in arr(&doc["metadata"]["releases"]) {
        for d in arr(&x["diagnostics"]) {
            if d["occurrence"].is_object() {
                let v = &d["occurrence"];
                let id = record_id(
                    &strv(&v["capture"]["namespace"]),
                    &strv(&v["capture"]["sha256"]),
                    &strv(&v["locator"]),
                );
                occurrences.entry(id).or_insert_with(|| {
                    (
                        strv(&v["capture"]["namespace"]),
                        strv(&v["capture"]["sha256"]),
                        strv(&v["locator"]),
                    )
                });
            }
        }
    }
    for key in [
        "contexts",
        "frames",
        "forms",
        "etymologies",
        "relations",
        "extensions",
    ] {
        for x in arr(&doc["metadata"][key]) {
            let ev = &x["evidence"];
            if ev.is_object() {
                let v = &ev["occurrence"];
                let id = record_id(
                    &strv(&v["capture"]["namespace"]),
                    &strv(&v["capture"]["sha256"]),
                    &strv(&v["locator"]),
                );
                occurrences.entry(id).or_insert_with(|| {
                    (
                        strv(&v["capture"]["namespace"]),
                        strv(&v["capture"]["sha256"]),
                        strv(&v["locator"]),
                    )
                });
            }
            if key == "relations" {
                let v = &x["target"]["Catalog"]["occurrence"];
                if v.is_object() {
                    let id = record_id(
                        &strv(&v["capture"]["namespace"]),
                        &strv(&v["capture"]["sha256"]),
                        &strv(&v["locator"]),
                    );
                    occurrences.entry(id).or_insert_with(|| {
                        (
                            strv(&v["capture"]["namespace"]),
                            strv(&v["capture"]["sha256"]),
                            strv(&v["locator"]),
                        )
                    });
                }
            }
        }
    }
    for (id, (ns, sha, loc)) in &occurrences {
        add("occurrence_refs", vec![t(id), t(ns), t(sha), t(loc)]);
    }
    for (n, x) in arr(&registry["occurrences"]).iter().enumerate() {
        let id = record_id(
            &strv(&x["occurrence"]["snapshot"]["namespace"]),
            &strv(&x["occurrence"]["snapshot"]["sha256"]),
            &strv(&x["occurrence"]["locator"]),
        );
        add(
            "occurrence_attributions",
            vec![
                t(id),
                i(n as i64),
                opt_text(&x["contributor"]),
                opt_text(&x["native_entry_id"]),
                opt_text(&x["native_assertion_id"]),
            ],
        );
    }
    for (n, x) in arr(&doc["captures"]).iter().enumerate() {
        let ns = strv(&x["identity"]["namespace"]);
        let sha = strv(&x["identity"]["sha256"]);
        let m = &x["metadata"];
        add(
            "raw_captures",
            vec![
                t(ns.clone()),
                t(sha.clone()),
                i(*snapshot_ord
                    .get(&(ns.clone(), sha.clone()))
                    .unwrap_or(&(n as i64))),
                b(byte_array(&x["bytes"])),
                t(strv(&m["source_name"])),
                t(strv(&m["format_label"])),
                t(strv(&m["locator"])),
                pstate(&m["revision"]),
                pvalue(&m["revision"]),
                pstate(&m["captured_at"]),
                pvalue(&m["captured_at"]),
                t(enumv(&m["time_precision"])),
                pstate(&m["notice_locator"]),
                pvalue(&m["notice_locator"]),
            ],
        );
        for (ord, row) in arr(&x["occurrences"]).iter().enumerate() {
            let loc = strv(&row["locator"]);
            let id = record_id(&ns, &sha, &loc);
            add(
                "raw_occurrences",
                vec![
                    t(id.clone()),
                    t(ns.clone()),
                    t(sha.clone()),
                    i(ord as i64),
                    opt_text(&row["raw_line"]),
                ],
            );
            for (f, field) in arr(&row["fields"]).iter().enumerate() {
                add(
                    "native_fields",
                    vec![
                        t(id.clone()),
                        i(f as i64),
                        t(strv(&field["name"])),
                        t(enumv(&field["encoding"])),
                        t(strv(&field["raw_value"])),
                    ],
                );
            }
        }
    }
    add_model_rows(&mut add, &doc, &registry, &occurrences)?;
    Ok(out)
}

fn arr(v: &J) -> &[J] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn strv(v: &J) -> String {
    v.as_str().unwrap_or("").to_owned()
}
fn enumv(v: &J) -> String {
    v.as_str()
        .map(str::to_owned)
        .or_else(|| v.as_object().and_then(|m| m.keys().next().cloned()))
        .unwrap_or_default()
}
fn t(v: impl Into<String>) -> Value {
    Value::Text(v.into())
}
fn i(v: i64) -> Value {
    Value::Integer(v)
}
fn b(v: Vec<u8>) -> Value {
    Value::Blob(v)
}
fn opt_text(v: &J) -> Value {
    v.as_str().map(t).unwrap_or(Value::Null)
}
fn opt_bytes(v: &J) -> Value {
    v.as_array()
        .map(|a| b(a.iter().map(|n| n.as_u64().unwrap_or(0) as u8).collect()))
        .unwrap_or(Value::Null)
}
fn byte_array(v: &J) -> Vec<u8> {
    v.as_array()
        .map(|a| a.iter().map(|n| n.as_u64().unwrap_or(0) as u8).collect())
        .unwrap_or_default()
}
fn pstate(v: &J) -> Value {
    if v.as_str() == Some("Missing") {
        t("missing")
    } else if v.as_str() == Some("Null") {
        t("null")
    } else {
        t("value")
    }
}
fn pvalue(v: &J) -> Value {
    v.get("Value").map(opt_text).unwrap_or(Value::Null)
}
fn record_id(ns: &str, sha: &str, loc: &str) -> String {
    let mut out = Vec::new();
    for p in [ns.as_bytes(), sha.as_bytes(), loc.as_bytes()] {
        out.extend_from_slice(&(p.len() as u64).to_be_bytes());
        out.extend_from_slice(p);
    }
    let mut s = String::new();
    for x in out {
        use std::fmt::Write;
        let _ = write!(s, "{x:02x}");
    }
    s
}

fn add_model_rows<F>(
    add: &mut F,
    doc: &J,
    _registry: &J,
    occurrences: &BTreeMap<String, (String, String, String)>,
) -> Result<()>
where
    F: FnMut(&'static str, Vec<Value>),
{
    let m = &doc["metadata"];
    let roles = [
        ("collection", "collections"),
        ("component", "components"),
        ("release", "releases"),
        ("entry", "entries"),
        ("assertion", "assertions"),
        ("context", "contexts"),
        ("frame", "frames"),
        ("form", "forms"),
        ("etymology", "etymologies"),
        ("relation", "relations"),
        ("owned_revision", "owned_revisions"),
        ("editorial_sense", "editorial_senses"),
        ("computation", "computations"),
        ("annotation", "annotations"),
    ];
    for (role, key) in roles {
        for (n, x) in arr(&m[key]).iter().enumerate() {
            add("entity_ids", vec![t(strv(&x["id"])), t(role), i(n as i64)]);
        }
    }
    for (n, x) in arr(&m["collections"]).iter().enumerate() {
        let mut r = vec![
            t(strv(&x["id"])),
            t("collection"),
            i(n as i64),
            t(strv(&x["source"])),
            t(strv(&x["display_name"])),
        ];
        r.extend(tp(&x["description"]));
        add("collections", r);
    }
    for (n, x) in arr(&m["components"]).iter().enumerate() {
        add(
            "components",
            vec![
                t(strv(&x["id"])),
                t("component"),
                i(n as i64),
                t(strv(&x["collection"])),
                t(strv(&x["native_name"])),
            ],
        );
        for (o, lang) in arr(&x["languages"]).iter().enumerate() {
            add(
                "component_languages",
                vec![t(strv(&x["id"])), i(o as i64), t(strv(lang))],
            );
        }
    }
    let mut evidence_ids = BTreeMap::<String, i64>::new();
    let mut evidence_rows = Vec::<Vec<Value>>::new();
    {
        let mut evidence = |ev: &J| -> i64 {
            let key = ev.to_string();
            if let Some(id) = evidence_ids.get(&key) {
                return *id;
            }
            let id = evidence_ids.len() as i64 + 1;
            let occurrence = &ev["occurrence"];
            let oid = record_id(
                &strv(&occurrence["capture"]["namespace"]),
                &strv(&occurrence["capture"]["sha256"]),
                &strv(&occurrence["locator"]),
            );
            let (start, end) = if ev["offset"].is_object() {
                (
                    Some(u64str(&ev["offset"]["start"])),
                    Some(u64str(&ev["offset"]["end"])),
                )
            } else {
                (None, None)
            };
            let mut row = vec![i(id), t(oid)];
            row.push(
                ev["field_index"]
                    .as_u64()
                    .map(|v| t(v.to_string()))
                    .unwrap_or(Value::Null),
            );
            row.push(pstate(&ev["citation"]));
            row.push(pvalue(&ev["citation"]));
            row.push(start.map(t).unwrap_or(Value::Null));
            row.push(end.map(t).unwrap_or(Value::Null));
            evidence_rows.push(row);
            evidence_ids.insert(key, id);
            id
        };
        for x in arr(&m["releases"]) {
            for d in arr(&x["diagnostics"]) {
                if d["occurrence"].is_object() {
                    let _ = occurrence_id(&d["occurrence"], occurrences);
                }
            }
        }
        for x in arr(&m["contexts"]) {
            let _ = evidence(&x["evidence"]);
        }
        for x in arr(&m["frames"]) {
            let _ = evidence(&x["evidence"]);
        }
        for x in arr(&m["forms"]) {
            if x["evidence"].is_object() {
                let _ = evidence(&x["evidence"]);
            }
        }
        for x in arr(&m["etymologies"]) {
            let _ = evidence(&x["evidence"]);
        }
        for x in arr(&m["relations"]) {
            if x["evidence"].is_object() {
                let _ = evidence(&x["evidence"]);
            }
        }
        for x in arr(&m["extensions"]) {
            if x["evidence"].is_object() {
                let _ = evidence(&x["evidence"]);
            }
        }
    }
    for row in evidence_rows {
        add("source_evidence", row);
    }
    for (n, x) in arr(&m["releases"]).iter().enumerate() {
        let a = &x["accounting"];
        let mut r = vec![
            t(strv(&x["id"])),
            t("release"),
            i(n as i64),
            t(strv(&x["component"])),
            opt_text(&x["distributor"]),
        ];
        r.extend(p(&x["upstream_revision"]));
        r.extend(p(&x["distributor_revision"]));
        r.extend(p(&x["captured_at"]));
        r.extend([
            t(enumv(&x["time_precision"])),
            u32i(&x["schema_version"]),
            t(strv(&x["normalizer"])),
            t(strv(&x["normalizer_version"])),
            opt_u64(&a["input_occurrences"]),
            opt_u64(&a["normalized_occurrences"]),
            opt_u64(&a["query_excluded_occurrences"]),
            t(strv(&a["unit"])),
            i(if x["selected"].as_bool() == Some(true) {
                1
            } else {
                0
            }),
            t(enumv(&x["availability"])),
            t(enumv(&x["redistribution"])),
        ]);
        add("releases", r);
        for (o, c) in arr(&x["captures"]).iter().enumerate() {
            add(
                "release_captures",
                vec![
                    t(strv(&x["id"])),
                    i(o as i64),
                    t(strv(&c["namespace"])),
                    t(strv(&c["sha256"])),
                ],
            );
        }
        for (o, a) in arr(&x["assertions"]).iter().enumerate() {
            add(
                "release_assertions",
                vec![t(strv(&x["id"])), i(o as i64), t(strv(a))],
            );
        }
        for (o, d) in arr(&x["diagnostics"]).iter().enumerate() {
            let oid = if d["occurrence"].is_object() {
                t(occurrence_id(&d["occurrence"], occurrences))
            } else {
                Value::Null
            };
            add(
                "release_diagnostics",
                vec![
                    t(strv(&x["id"])),
                    i(o as i64),
                    oid,
                    t(strv(&d["code"])),
                    t(strv(&d["message"])),
                ],
            );
        }
        for (o, c) in arr(&x["content_notices"]).iter().enumerate() {
            let mut r = vec![t(strv(&x["id"])), i(o as i64)];
            r.extend(p(&c["text"]));
            r.extend(p(&c["locator"]));
            r.extend(p(&c["license"]));
            add("content_notices", r);
        }
        for (o, e) in arr(&x["redistribution_evidence"]).iter().enumerate() {
            add(
                "redistribution_evidence",
                vec![t(strv(&x["id"])), i(o as i64), t(strv(e))],
            );
        }
    }
    for (n, x) in arr(&m["code_notices"]).iter().enumerate() {
        let mut r = vec![i(n as i64), t(strv(&x["text"]))];
        r.extend(p(&x["locator"]));
        r.extend(p(&x["license"]));
        add("code_notices", r);
    }
    for (n, x) in arr(&m["entries"]).iter().enumerate() {
        let mut r = vec![
            t(strv(&x["id"])),
            t("entry"),
            i(n as i64),
            t(strv(&x["source"])),
        ];
        r.extend(p(&x["spelling"]));
        r.extend(p(&x["language"]));
        r.extend(p(&x["script"]));
        r.extend(p(&x["native_word_class"]));
        r.extend(p(&x["selmaho"]));
        r.extend(pm(&x["mapped_pos"]));
        add("entries", r);
    }
    for (n, x) in arr(&m["assertions"]).iter().enumerate() {
        let mut r = vec![
            t(strv(&x["id"])),
            t("assertion"),
            i(n as i64),
            t(strv(&x["entry"])),
            t(occurrence_id(&x["occurrence"], occurrences)),
        ];
        r.extend(p(&x["native_sense_id"]));
        r.extend(p(&x["language"]));
        r.extend(tp(&x["definition"]));
        r.extend(p(&x["source_score"]));
        let (eligibility, reason) = if x["eligibility"].as_str() == Some("Lexical") {
            ("Lexical", Value::Null)
        } else {
            (
                "Excluded",
                x["eligibility"]["Excluded"]
                    .get("reason")
                    .map(|v| t(strv(v)))
                    .unwrap_or(Value::Null),
            )
        };
        r.extend([t(eligibility), reason]);
        add("assertions", r);
        for (o, ts) in arr(&x["timestamps"]).iter().enumerate() {
            add(
                "assertion_timestamps",
                vec![
                    t(strv(&x["id"])),
                    i(o as i64),
                    t(strv(&ts["field"])),
                    t(strv(&ts["value"])),
                    t(enumv(&ts["precision"])),
                ],
            );
        }
    }
    for (n, x) in arr(&m["contexts"]).iter().enumerate() {
        let mut r = vec![
            t(strv(&x["id"])),
            t("context"),
            i(n as i64),
            t(strv(&x["assertion"])),
            t(enumv(&x["kind"])),
        ];
        r.extend(tp(&x["value"]));
        r.push(opt_text(&x["translated_from"]));
        r.push(i(*evidence_ids.get(&x["evidence"].to_string()).unwrap()));
        add("contexts", r);
    }
    for (n, x) in arr(&m["frames"]).iter().enumerate() {
        let eid = *evidence_ids.get(&x["evidence"].to_string()).unwrap();
        add(
            "argument_frames",
            vec![
                t(strv(&x["id"])),
                t("frame"),
                i(n as i64),
                t(strv(&x["assertion"])),
                i(eid),
            ],
        );
        for (o, s) in arr(&x["slots"]).iter().enumerate() {
            let mut r = vec![
                t(strv(&x["id"])),
                i(o as i64),
                u32i(&s["order"]),
                t(strv(&s["native_label"])),
            ];
            r.extend(tp(&s["description"]));
            r.extend(p(&s["grammatical_role"]));
            add("argument_slots", r);
        }
    }
    for (n, x) in arr(&m["forms"]).iter().enumerate() {
        let mut r = vec![
            t(strv(&x["id"])),
            t("form"),
            i(n as i64),
            t(strv(&x["entry"])),
            t(strv(&x["spelling"])),
        ];
        r.extend(p(&x["language"]));
        r.extend(p(&x["script"]));
        r.extend(tp(&x["pronunciation"]));
        r.extend([t(enumv(&x["kind"]))]);
        r.extend(p(&x["standing"]));
        r.push(
            x["evidence"]
                .as_object()
                .map(|_| i(*evidence_ids.get(&x["evidence"].to_string()).unwrap()))
                .unwrap_or(Value::Null),
        );
        r.push(opt_text(&x["computation"]));
        add("forms", r);
        for (o, c) in arr(&x["components"]).iter().enumerate() {
            let mut row = vec![
                t(strv(&x["id"])),
                i(o as i64),
                u32i(&c["order"]),
                t(strv(&c["spelling"])),
            ];
            row.extend(local_ref(c.get("target")));
            add("form_components", row);
        }
    }
    for (n, x) in arr(&m["etymologies"]).iter().enumerate() {
        let mut r = vec![
            t(strv(&x["id"])),
            t("etymology"),
            i(n as i64),
            t(strv(&x["assertion"])),
        ];
        r.extend(tp(&x["statement"]));
        r.extend(p(&x["native_kind"]));
        r.push(i(*evidence_ids.get(&x["evidence"].to_string()).unwrap()));
        add("etymologies", r);
        for (o, c) in arr(&x["components"]).iter().enumerate() {
            let mut row = vec![
                t(strv(&x["id"])),
                i(o as i64),
                u32i(&c["order"]),
                t(strv(&c["spelling"])),
            ];
            row.extend(local_ref(c.get("target")));
            add("etymology_components", row);
        }
    }
    for (n, x) in arr(&m["relations"]).iter().enumerate() {
        let mut r = vec![t(strv(&x["id"])), t("relation"), i(n as i64)];
        r.extend(local_ref(Some(&x["origin"])));
        let kind = enumv(&x["kind"]);
        let other = x["kind"]
            .get("Other")
            .map(|v| t(strv(v)))
            .unwrap_or(Value::Null);
        r.extend([
            t(kind),
            other,
            t(enumv(&x["direction"])),
            t(enumv(&x["scope"])),
        ]);
        let target = &x["target"];
        if target.get("Local").is_some() {
            r.push(t("Local"));
            r.extend(local_ref(target.get("Local")));
            r.extend([
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
            ]);
        } else if target.get("Catalog").is_some() {
            let c = &target["Catalog"];
            r.extend([
                t("Catalog"),
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                t(strv(&c["source"])),
                t(strv(&c["release"])),
                opt_text(&c["native_entry_id"]),
                opt_text(&c["native_assertion_id"]),
                c["occurrence"]
                    .as_object()
                    .map(|v| t(occurrence_id(&J::Object(v.clone()), occurrences)))
                    .unwrap_or(Value::Null),
                Value::Null,
            ]);
        } else if target.get("Unknown").is_some() || target.as_str() == Some("Unknown") {
            r.extend([
                t("Unknown"),
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
            ]);
        } else {
            r.extend([
                t("Unresolved"),
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                opt_text(&target["Unresolved"]),
            ]);
        }
        r.push(
            x["evidence"]
                .as_object()
                .map(|_| i(*evidence_ids.get(&x["evidence"].to_string()).unwrap()))
                .unwrap_or(Value::Null),
        );
        r.push(opt_text(&x["editorial_revision"]));
        add("relations", r);
    }
    for (n, x) in arr(&m["owned_revisions"]).iter().enumerate() {
        let mut r = vec![
            t(strv(&x["id"])),
            t("owned_revision"),
            i(n as i64),
            t(strv(&x["source"])),
            opt_text(&x["parent"]),
            t(strv(&x["editor"])),
        ];
        r.extend(p(&x["timestamp"]));
        r.push(t(enumv(&x["time_precision"])));
        r.extend(tp(&x["rationale"]));
        add("owned_revisions", r);
        for (o, id) in arr(&x["assertions"]).iter().enumerate() {
            add(
                "revision_assertions",
                vec![t(strv(&x["id"])), i(o as i64), t(strv(id))],
            );
        }
        for (o, id) in arr(&x["tombstones"]).iter().enumerate() {
            add(
                "revision_tombstones",
                vec![t(strv(&x["id"])), i(o as i64), t(strv(id))],
            );
        }
    }
    for (n, x) in arr(&m["owned_heads"]).iter().enumerate() {
        add(
            "owned_heads",
            vec![t(strv(&x["source"])), i(n as i64), t(strv(&x["revision"]))],
        );
    }
    for (n, x) in arr(&m["editorial_senses"]).iter().enumerate() {
        let mut r = vec![
            t(strv(&x["id"])),
            t("editorial_sense"),
            i(n as i64),
            t(strv(&x["editor"])),
            t(strv(&x["revision"])),
        ];
        r.extend(textvalue(&x["rationale"]));
        add("editorial_senses", r);
        for (o, id) in arr(&x["assertions"]).iter().enumerate() {
            add(
                "editorial_members",
                vec![t(strv(&x["id"])), i(o as i64), t(strv(id))],
            );
        }
    }
    for (n, x) in arr(&m["computations"]).iter().enumerate() {
        let mut r = vec![
            t(strv(&x["id"])),
            t("computation"),
            i(n as i64),
            t(strv(&x["tool"])),
            t(strv(&x["tool_version"])),
            t(strv(&x["policy"])),
        ];
        r.extend(p(&x["profile"]));
        r.extend(tp(&x["explanation"]));
        add("computations", r);
        for (o, input) in arr(&x["inputs"]).iter().enumerate() {
            let mut row = vec![t(strv(&x["id"])), i(o as i64)];
            if input.get("Occurrence").is_some() {
                row.extend([
                    t("Occurrence"),
                    t(occurrence_id(&input["Occurrence"], occurrences)),
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    Value::Null,
                ]);
            } else if input.get("Entity").is_some() {
                row.extend([t("Entity"), Value::Null]);
                row.extend(local_ref(input.get("Entity")));
                row.push(Value::Null);
            } else {
                row.extend([
                    t("Literal"),
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    opt_text(&input["Literal"]),
                ]);
            }
            add("computation_inputs", row);
        }
    }
    for (n, x) in arr(&m["annotations"]).iter().enumerate() {
        let mut r = vec![
            t(strv(&x["id"])),
            t("annotation"),
            i(n as i64),
            t(strv(&x["computation"])),
        ];
        r.extend(local_ref(Some(&x["target"])));
        r.extend([t(strv(&x["kind"])), t(strv(&x["value"]))]);
        r.extend(tp(&x["explanation"]));
        add("annotations", r);
    }
    for (n, x) in arr(&m["extensions"]).iter().enumerate() {
        add(
            "extensions",
            vec![
                i(n as i64),
                t(strv(&x["owner"])),
                t(strv(&x["name"])),
                t(strv(&x["value"])),
                x["evidence"]
                    .as_object()
                    .map(|_| i(*evidence_ids.get(&x["evidence"].to_string()).unwrap()))
                    .unwrap_or(Value::Null),
            ],
        );
    }
    Ok(())
}

fn occurrence_id(v: &J, known: &BTreeMap<String, (String, String, String)>) -> String {
    let c = &v["capture"];
    let id = record_id(
        &strv(&c["namespace"]),
        &strv(&c["sha256"]),
        &strv(&v["locator"]),
    );
    let _ = known.get(&id);
    id
}
fn u64str(v: &J) -> String {
    v.as_u64().unwrap_or(0).to_string()
}
fn u32i(v: &J) -> Value {
    i(v.as_u64().unwrap_or(0).min(i64::MAX as u64) as i64)
}
fn opt_u64(v: &J) -> Value {
    v.as_u64().map(|n| t(n.to_string())).unwrap_or(Value::Null)
}
fn p(v: &J) -> Vec<Value> {
    vec![pstate(v), pvalue(v)]
}
fn local_ref(v: Option<&J>) -> Vec<Value> {
    let Some(v) = v else {
        return vec![
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
        ];
    };
    let (kind, id) = if let Some(x) = v.get("Entry") {
        ("Entry", x)
    } else if let Some(x) = v.get("Assertion") {
        ("Assertion", x)
    } else if let Some(x) = v.get("Form") {
        ("Form", x)
    } else if let Some(x) = v.get("EditorialSense") {
        ("EditorialSense", x)
    } else {
        return vec![
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
        ];
    };
    vec![
        t(kind),
        if kind == "Entry" {
            t(strv(id))
        } else {
            Value::Null
        },
        if kind == "Assertion" {
            t(strv(id))
        } else {
            Value::Null
        },
        if kind == "Form" {
            t(strv(id))
        } else {
            Value::Null
        },
        if kind == "EditorialSense" {
            t(strv(id))
        } else {
            Value::Null
        },
    ]
}
fn textvalue(v: &J) -> Vec<Value> {
    vec![
        t(strv(&v["text"])),
        pstate(&v["language"]),
        pvalue(&v["language"]),
        pstate(&v["markup"]),
        pvalue(&v["markup"]),
    ]
}
fn tp(v: &J) -> Vec<Value> {
    let state = pstate(v);
    if let Some(tv) = v.get("Value") {
        let mut r = vec![state];
        r.extend(textvalue(tv));
        r
    } else {
        vec![
            state,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
        ]
    }
}
fn pm(v: &J) -> Vec<Value> {
    let mut r = vec![pstate(v)];
    if let Some(x) = v.get("Value") {
        r.extend([opt_text(&x["value"]), opt_text(&x["rule"])])
    } else {
        r.extend([Value::Null, Value::Null])
    };
    r
}

pub(crate) fn write_projection(
    conn: &Connection,
    dictionary: &Dictionary,
    canonical: &[u8],
) -> Result<()> {
    let rows = project(dictionary, canonical)?;
    for (table, table_rows) in rows {
        if table_rows.is_empty() {
            continue;
        }
        let width = table_rows[0].len();
        let columns: i64 = conn.query_row(
            &format!("SELECT count(*) FROM pragma_table_info('{table}')"),
            [],
            |r| r.get(0),
        )?;
        if width != columns as usize {
            return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(
                std::io::Error::other(format!(
                    "{table} projection has {width} cells; schema has {columns}"
                )),
            )));
        }
        let placeholders = (1..=width)
            .map(|n| format!("?{n}"))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!("INSERT INTO {table} VALUES({placeholders})");
        let mut stmt = conn.prepare(&sql)?;
        for row in table_rows {
            if table == "metadata" && row.first() == Some(&Value::Text("complete".into())) {
                continue;
            }
            if row.len() != width {
                return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(
                    std::io::Error::other(format!(
                        "{table} has inconsistent row width {} versus {width}",
                        row.len()
                    )),
                )));
            }
            stmt.execute(params_from_iter(row))?;
        }
    }
    Ok(())
}

pub(crate) fn check_projection(
    conn: &Connection,
    dictionary: &Dictionary,
    canonical: &[u8],
) -> std::result::Result<(), String> {
    let expected = project(dictionary, canonical).map_err(|e| e.to_string())?;
    let mut names = Vec::new();
    let mut stmt=conn.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").map_err(|e|e.to_string())?;
    names.extend(
        stmt.query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    );
    for table in names {
        let rows = expected
            .get(table.as_str())
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let mut stmt = conn
            .prepare(&format!("SELECT * FROM {table}"))
            .map_err(|e| e.to_string())?;
        let actual = stmt
            .query_map([], |r| {
                let mut row = Vec::new();
                for col in 0..r.as_ref().column_count() {
                    row.push(r.get::<_, Value>(col)?);
                }
                Ok(row)
            })
            .map_err(|e| e.to_string())?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let mut actual = actual;
        let mut wanted = rows.to_vec();
        actual.sort_by_key(|row| format!("{row:?}"));
        wanted.sort_by_key(|row| format!("{row:?}"));
        if actual != wanted {
            return Err(format!(
                "projection differs in {table}: actual {}, expected {} rows",
                actual.len(),
                wanted.len()
            ));
        }
    }
    Ok(())
}
