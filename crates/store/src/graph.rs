use std::{
    collections::{BTreeSet, HashMap},
    fmt,
    path::{Path, PathBuf},
};

use oxigraph::{
    model::{GraphName, Literal, NamedNode, NamedOrBlankNodeRef, Quad, Term, Variable},
    sparql::{QueryResults, SparqlEvaluator},
    store::Store,
};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::FieldPresence;

use crate::{
    EdgeHit, LexicalStore, PresenceState, SearchQuery, SearchResult, StoreError, archive_sha256,
    directory_bytes, empty_result, record_id, require_fresh, role_name, validate_archive,
};

const BASE: &str = "urn:relojban:lexical:";
const REC: &str = "urn:relojban:record:";
const EDGE: &str = "urn:relojban:edge:";
const FORM: &str = "urn:relojban:form:";
const FIELD: &str = "urn:relojban:field:";
const PROP: &str = "urn:relojban:proposal:";
const P_SCHEMA: &str = "urn:relojban:lexical:schemaVersion";
const P_COMPLETE: &str = "urn:relojban:lexical:complete";
const P_ARCHIVE: &str = "urn:relojban:lexical:archive";
const P_ARCHIVE_HASH: &str = "urn:relojban:lexical:archiveSha256";
const P_TYPE: &str = "urn:relojban:lexical:type";
const P_WORD: &str = "urn:relojban:lexical:word";
const P_DEFINITION: &str = "urn:relojban:lexical:definitionAsciiLower";
const P_HAS_FORM: &str = "urn:relojban:lexical:hasForm";
const P_FORM_VALUE: &str = "urn:relojban:lexical:formValue";
const P_FORM_ROLE: &str = "urn:relojban:lexical:formRole";
const P_HAS_FIELD: &str = "urn:relojban:lexical:hasField";
const P_FIELD_NAME: &str = "urn:relojban:lexical:fieldName";
const P_FIELD_STATE: &str = "urn:relojban:lexical:fieldState";
const P_FIELD_RAW: &str = "urn:relojban:lexical:fieldRaw";
const P_HAS_EDGE: &str = "urn:relojban:lexical:hasEdge";
const P_EDGE_ORIGIN: &str = "urn:relojban:lexical:edgeOrigin";
const P_EDGE_TARGET: &str = "urn:relojban:lexical:edgeTarget";
const P_EDGE_SPELLING: &str = "urn:relojban:lexical:edgeTargetSpelling";
const P_EDGE_START: &str = "urn:relojban:lexical:edgeByteStart";
const P_EDGE_END: &str = "urn:relojban:lexical:edgeByteEnd";
const P_EDGE_RULE: &str = "urn:relojban:lexical:edgeRule";
const P_PROPOSAL_SPELLING: &str = "urn:relojban:lexical:proposalSpelling";

/// A synchronous native Oxigraph `Store::open` backed by RocksDB.
pub struct GraphStore {
    path: PathBuf,
    store: Store,
}

impl fmt::Debug for GraphStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GraphStore")
            .field("path", &self.path)
            .field("native_index", &"Oxigraph/RocksDB")
            .finish_non_exhaustive()
    }
}

impl GraphStore {
    /// Builds a new disk graph index from a valid accepted archive.
    ///
    /// Requires the pinned Oxigraph RocksDB feature. The caller path must not
    /// exist. Bulk loading is staged before the completion marker is written.
    pub fn create(path: impl AsRef<Path>, archive: &[u8]) -> Result<Self, StoreError> {
        let path = path.as_ref();
        require_fresh(path)?;
        let lexicon = validate_archive(archive)?;
        std::fs::create_dir_all(path).map_err(StoreError::Io)?;
        let store = Store::open(path).map_err(graph_error)?;
        let mut quads = Vec::new();
        push(
            &mut quads,
            BASE,
            P_SCHEMA,
            Literal::new_simple_literal("1").into(),
        )?;
        push(
            &mut quads,
            BASE,
            P_COMPLETE,
            Literal::new_simple_literal("false").into(),
        )?;
        push(
            &mut quads,
            BASE,
            P_ARCHIVE_HASH,
            Literal::new_simple_literal(archive_sha256(archive)).into(),
        )?;
        let archive_text = std::str::from_utf8(archive)
            .map_err(|error| StoreError::InvalidArchive(error.to_string()))?;
        push(
            &mut quads,
            BASE,
            P_ARCHIVE,
            Literal::new_simple_literal(archive_text).into(),
        )?;

        for record in lexicon.records() {
            let id = record_id(record.identity());
            let uri = format!("{REC}{id}");
            push(
                &mut quads,
                &uri,
                P_TYPE,
                Literal::new_simple_literal("record").into(),
            )?;
            if let Some(word) = record.word() {
                push(
                    &mut quads,
                    &uri,
                    P_WORD,
                    Literal::new_simple_literal(word).into(),
                )?;
            }
            if let Some(definition) = record.definition() {
                push(
                    &mut quads,
                    &uri,
                    P_DEFINITION,
                    Literal::new_simple_literal(definition.to_ascii_lowercase()).into(),
                )?;
            }
        }
        for form in lexicon.forms() {
            let id = record_id(form.source_record());
            let node = format!(
                "{FORM}{}",
                digest_parts(&[
                    id.as_bytes(),
                    form.form().as_bytes(),
                    role_name(form.role()).as_bytes(),
                    form.source_field().as_bytes()
                ])
            );
            let record_uri = format!("{REC}{id}");
            push(
                &mut quads,
                &record_uri,
                P_HAS_FORM,
                NamedNode::new(&node).map_err(graph_error)?.into(),
            )?;
            push(
                &mut quads,
                &node,
                P_FORM_VALUE,
                Literal::new_simple_literal(form.form()).into(),
            )?;
            push(
                &mut quads,
                &node,
                P_FORM_ROLE,
                Literal::new_simple_literal(role_name(form.role())).into(),
            )?;
        }
        let mut names: BTreeSet<String> = ["notes".into(), "selmaho".into(), "unknown".into()]
            .into_iter()
            .collect();
        for record in lexicon.records() {
            names.extend(record.fields().iter().map(|field| field.name().to_owned()));
        }
        for record in lexicon.records() {
            let id = record_id(record.identity());
            let record_uri = format!("{REC}{id}");
            for name in &names {
                let (state, raw) = match record.field(name) {
                    FieldPresence::Missing => continue,
                    FieldPresence::Null => ("null", "null"),
                    FieldPresence::Value(value) => ("value", value),
                    _ => ("value", ""),
                };
                let node = format!("{FIELD}{}", digest_parts(&[id.as_bytes(), name.as_bytes()]));
                push(
                    &mut quads,
                    &record_uri,
                    P_HAS_FIELD,
                    NamedNode::new(&node).map_err(graph_error)?.into(),
                )?;
                push(
                    &mut quads,
                    &node,
                    P_FIELD_NAME,
                    Literal::new_simple_literal(name).into(),
                )?;
                push(
                    &mut quads,
                    &node,
                    P_FIELD_STATE,
                    Literal::new_simple_literal(state).into(),
                )?;
                push(
                    &mut quads,
                    &node,
                    P_FIELD_RAW,
                    Literal::new_simple_literal(raw).into(),
                )?;
            }
        }
        for reference in lexicon.cross_references() {
            let origin_id = record_id(reference.origin());
            let origin_uri = format!("{REC}{origin_id}");
            let targets: Vec<Option<String>> = if reference.targets().is_empty() {
                vec![None]
            } else {
                reference
                    .targets()
                    .iter()
                    .map(|target| Some(record_id(target)))
                    .collect()
            };
            for target_id in targets {
                let target_key = target_id.as_deref().unwrap_or("unresolved");
                let edge_id = digest_parts(&[
                    origin_id.as_bytes(),
                    target_key.as_bytes(),
                    &reference.byte_start().to_be_bytes(),
                    &reference.byte_end().to_be_bytes(),
                ]);
                let node = format!("{EDGE}{edge_id}");
                push(
                    &mut quads,
                    &origin_uri,
                    P_HAS_EDGE,
                    NamedNode::new(&node).map_err(graph_error)?.into(),
                )?;
                push(
                    &mut quads,
                    &node,
                    P_EDGE_ORIGIN,
                    NamedNode::new(&origin_uri).map_err(graph_error)?.into(),
                )?;
                if let Some(target_id) = target_id {
                    push(
                        &mut quads,
                        &node,
                        P_EDGE_TARGET,
                        NamedNode::new(format!("{REC}{target_id}"))
                            .map_err(graph_error)?
                            .into(),
                    )?;
                }
                push(
                    &mut quads,
                    &node,
                    P_EDGE_SPELLING,
                    Literal::new_simple_literal(reference.target_spelling()).into(),
                )?;
                push(
                    &mut quads,
                    &node,
                    P_EDGE_START,
                    Literal::from(reference.byte_start() as i64).into(),
                )?;
                push(
                    &mut quads,
                    &node,
                    P_EDGE_END,
                    Literal::from(reference.byte_end() as i64).into(),
                )?;
                push(
                    &mut quads,
                    &node,
                    P_EDGE_RULE,
                    Literal::new_simple_literal(reference.rule_version()).into(),
                )?;
            }
        }
        for proposal in lexicon.proposals() {
            let node = format!("{PROP}{}", proposal.id());
            push(
                &mut quads,
                &node,
                P_TYPE,
                Literal::new_simple_literal("proposal").into(),
            )?;
            push(
                &mut quads,
                &node,
                P_PROPOSAL_SPELLING,
                Literal::new_simple_literal(proposal.spelling()).into(),
            )?;
        }
        let mut transaction = store.start_transaction().map_err(graph_error)?;
        for quad in &quads {
            transaction.insert(quad);
        }
        transaction.commit().map_err(graph_error)?;
        let incomplete = Quad::new(
            named(BASE)?,
            named(P_COMPLETE)?,
            Literal::new_simple_literal("false"),
            GraphName::DefaultGraph,
        );
        store.remove(&incomplete).map_err(graph_error)?;
        let complete = Quad::new(
            named(BASE)?,
            named(P_COMPLETE)?,
            Literal::new_simple_literal("true"),
            GraphName::DefaultGraph,
        );
        store.insert(&complete).map_err(graph_error)?;
        store.flush().map_err(graph_error)?;
        Ok(Self {
            path: path.to_owned(),
            store,
        })
    }

    /// Opens an initialized disk RocksDB store and validates schema/archive identity.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(StoreError::InvalidStore("store path does not exist".into()));
        }
        let store = Store::open(path).map_err(graph_error)?;
        let version = literal_object(&store, BASE, P_SCHEMA)?;
        let complete = literal_object(&store, BASE, P_COMPLETE)?;
        let hash = literal_object(&store, BASE, P_ARCHIVE_HASH)?;
        let archive = literal_object(&store, BASE, P_ARCHIVE)?;
        let (Some(version), Some(complete), Some(hash), Some(archive)) =
            (version, complete, hash, archive)
        else {
            return Err(StoreError::InvalidStore(
                "required graph metadata is absent".into(),
            ));
        };
        if version != "1" || complete != "true" {
            return Err(StoreError::InvalidStore(
                "schema mismatch or incomplete build marker".into(),
            ));
        }
        if archive_sha256(archive.as_bytes()) != hash {
            return Err(StoreError::InvalidStore(
                "canonical archive hash mismatch".into(),
            ));
        }
        validate_archive(archive.as_bytes())?;
        Ok(Self {
            path: path.to_owned(),
            store,
        })
    }
}

impl LexicalStore for GraphStore {
    fn search(&self, query: &SearchQuery) -> Result<SearchResult, StoreError> {
        let mut result = empty_result("oxigraph-sparql-bound-literals", None);
        let mut bindings: HashMap<&str, String> = HashMap::new();
        let query_text = match query {
            SearchQuery::ExactSpelling(spelling) => {
                bindings.insert("needle", spelling.clone());
                "SELECT ?record ?needle WHERE { ?record <urn:relojban:lexical:word> ?value . FILTER(?value = ?needle) }"
            }
            SearchQuery::DefinitionContains(term) => {
                let needle = term.to_ascii_lowercase();
                if needle.is_empty() {
                    return Ok(result);
                }
                bindings.insert("needle", needle);
                "SELECT ?record ?needle WHERE { ?record <urn:relojban:lexical:definitionAsciiLower> ?value . FILTER(CONTAINS(?value, ?needle)) }"
            }
            SearchQuery::Form { value, role } => {
                bindings.insert("needle", value.clone());
                if let Some(role) = role {
                    bindings.insert("role", role_name(*role).to_owned());
                    "SELECT ?record ?needle ?role WHERE { ?record <urn:relojban:lexical:hasForm> ?form . ?form <urn:relojban:lexical:formValue> ?value . ?form <urn:relojban:lexical:formRole> ?formRole . FILTER(?value = ?needle && ?formRole = ?role) }"
                } else {
                    "SELECT ?record ?needle WHERE { ?record <urn:relojban:lexical:hasForm> ?form . ?form <urn:relojban:lexical:formValue> ?value . FILTER(?value = ?needle) }"
                }
            }
            SearchQuery::CrossReferences { spelling, depth } => {
                if !(1..=2).contains(depth) {
                    return Ok(result);
                }
                bindings.insert("needle", spelling.clone());
                if *depth == 1 {
                    "SELECT ?record ?edge ?needle WHERE { ?origin <urn:relojban:lexical:word> ?word ; <urn:relojban:lexical:hasEdge> ?edge . FILTER(?word = ?needle) . OPTIONAL { ?edge <urn:relojban:lexical:edgeTarget> ?record } }"
                } else {
                    "SELECT ?record ?edge ?edge2 ?needle WHERE { { ?origin <urn:relojban:lexical:word> ?word ; <urn:relojban:lexical:hasEdge> ?edge . FILTER(?word = ?needle) OPTIONAL { ?edge <urn:relojban:lexical:edgeTarget> ?middle . ?middle <urn:relojban:lexical:hasEdge> ?edge2 . OPTIONAL { ?edge2 <urn:relojban:lexical:edgeTarget> ?record } } } UNION { ?origin <urn:relojban:lexical:word> ?word ; <urn:relojban:lexical:hasEdge> ?edge . FILTER(?word = ?needle) OPTIONAL { ?edge <urn:relojban:lexical:edgeTarget> ?record } } }"
                }
            }
            SearchQuery::Field {
                name,
                state,
                raw_value,
            } => {
                bindings.insert("fieldName", name.clone());
                let missing = *state == PresenceState::Missing;
                if missing {
                    if raw_value.is_some() {
                        return Ok(result);
                    }
                    "SELECT ?record ?fieldName WHERE { ?record <urn:relojban:lexical:type> \"record\" . MINUS { ?record <urn:relojban:lexical:hasField> ?field . ?field <urn:relojban:lexical:fieldName> ?fieldName ; <urn:relojban:lexical:fieldState> ?presentState . FILTER(?presentState != \"missing\") } }"
                } else {
                    let state = match state {
                        PresenceState::Missing => "missing",
                        PresenceState::Null => "null",
                        PresenceState::Value => "value",
                    };
                    bindings.insert("state", state.to_owned());
                    if !missing {
                        if let Some(raw) = raw_value {
                            bindings.insert("expectedRaw", raw.clone());
                            "SELECT ?record ?fieldName ?state ?rawValue ?expectedRaw WHERE { ?record <urn:relojban:lexical:hasField> ?field . ?field <urn:relojban:lexical:fieldName> ?fieldName ; <urn:relojban:lexical:fieldState> ?state ; <urn:relojban:lexical:fieldRaw> ?rawValue . FILTER(?rawValue = ?expectedRaw) }"
                        } else {
                            "SELECT ?record ?fieldName ?state WHERE { ?record <urn:relojban:lexical:hasField> ?field . ?field <urn:relojban:lexical:fieldName> ?fieldName ; <urn:relojban:lexical:fieldState> ?state }"
                        }
                    } else {
                        "SELECT ?record ?fieldName ?state WHERE { ?record <urn:relojban:lexical:hasField> ?field . ?field <urn:relojban:lexical:fieldName> ?fieldName ; <urn:relojban:lexical:fieldState> ?state }"
                    }
                }
            }
            SearchQuery::ProposalExact(spelling) => {
                bindings.insert("needle", spelling.clone());
                "SELECT ?record ?needle WHERE { ?record <urn:relojban:lexical:type> \"proposal\" ; <urn:relojban:lexical:proposalSpelling> ?value . FILTER(?value = ?needle) }"
            }
        };
        let rows = sparql_rows(&self.store, query_text, &bindings)?;
        result.candidates = rows.len();
        for row in rows {
            if let Some(uri) = row.get("record") {
                let id = if let Some(id) = uri.strip_prefix(REC) {
                    id.to_owned()
                } else if let Some(id) = uri.strip_prefix(PROP) {
                    id.to_owned()
                } else {
                    uri.clone()
                };
                result.record_ids.insert(id);
            }
            if let Some(edge) = row.get("edge") {
                result.edges.insert(read_edge(&self.store, edge)?);
            }
            if let Some(edge) = row.get("edge2") {
                result.edges.insert(read_edge(&self.store, edge)?);
            }
        }
        if matches!(query, SearchQuery::CrossReferences { .. }) {
            result.path = "oxigraph-sparql-bounded-two-hop".into();
        }
        Ok(result)
    }

    fn archive_bytes(&self) -> Result<Vec<u8>, StoreError> {
        literal_object(&self.store, BASE, P_ARCHIVE)?
            .map(String::into_bytes)
            .ok_or_else(|| StoreError::InvalidStore("canonical archive triple is absent".into()))
    }

    fn flush(&self) -> Result<(), StoreError> {
        self.store.flush().map_err(graph_error)
    }
    fn logical_bytes(&self) -> Result<u64, StoreError> {
        directory_bytes(&self.path)
    }
}

fn push(
    quads: &mut Vec<Quad>,
    subject: &str,
    predicate: &str,
    object: Term,
) -> Result<(), StoreError> {
    quads.push(Quad::new(
        named(subject)?,
        named(predicate)?,
        object,
        GraphName::DefaultGraph,
    ));
    Ok(())
}

fn named(value: &str) -> Result<NamedNode, StoreError> {
    NamedNode::new(value).map_err(graph_error)
}

fn digest_parts(parts: &[&[u8]]) -> String {
    let mut digest = Sha256::new();
    for part in parts {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    format!("{:x}", digest.finalize())
}

fn sparql_rows(
    store: &Store,
    text: &str,
    bindings: &HashMap<&str, String>,
) -> Result<Vec<HashMap<String, String>>, StoreError> {
    let mut prepared = SparqlEvaluator::new()
        .parse_query(text)
        .map_err(graph_error)?;
    for (name, value) in bindings {
        prepared = prepared.substitute_variable(
            Variable::new(*name).map_err(graph_error)?,
            Literal::new_simple_literal(value.as_str()),
        );
    }
    let QueryResults::Solutions(solutions) =
        prepared.on_store(store).execute().map_err(graph_error)?
    else {
        return Err(StoreError::Backend(
            "expected SPARQL SELECT solutions".into(),
        ));
    };
    let mut rows = Vec::new();
    for solution in solutions {
        let solution = solution.map_err(graph_error)?;
        let mut row = HashMap::new();
        for name in ["record", "edge", "edge2"] {
            if let Some(term) = solution.get(name) {
                row.insert(name.to_owned(), term_text(term));
            }
        }
        rows.push(row);
    }
    Ok(rows)
}

fn term_text(term: &Term) -> String {
    match term {
        Term::NamedNode(node) => node.as_str().to_owned(),
        Term::Literal(literal) => literal.value().to_owned(),
        other => other.to_string(),
    }
}

fn literal_object(
    store: &Store,
    subject: &str,
    predicate: &str,
) -> Result<Option<String>, StoreError> {
    let subject = named(subject)?;
    let predicate = named(predicate)?;
    for quad in store.quads_for_pattern(
        Some(NamedOrBlankNodeRef::NamedNode(subject.as_ref())),
        Some(predicate.as_ref()),
        None,
        None,
    ) {
        let quad = quad.map_err(graph_error)?;
        if let Term::Literal(value) = quad.object {
            return Ok(Some(value.value().to_owned()));
        }
    }
    Ok(None)
}

fn read_edge(store: &Store, uri: &str) -> Result<EdgeHit, StoreError> {
    let origin = iri_object(store, uri, P_EDGE_ORIGIN)?.unwrap_or_default();
    let target = iri_object(store, uri, P_EDGE_TARGET)?;
    Ok(EdgeHit {
        origin_id: origin.strip_prefix(REC).unwrap_or(&origin).to_owned(),
        target_id: target.map(|value| value.strip_prefix(REC).unwrap_or(&value).to_owned()),
        target_spelling: literal_object(store, uri, P_EDGE_SPELLING)?.unwrap_or_default(),
        byte_start: literal_object(store, uri, P_EDGE_START)?
            .and_then(|v| v.parse().ok())
            .unwrap_or_default(),
        byte_end: literal_object(store, uri, P_EDGE_END)?
            .and_then(|v| v.parse().ok())
            .unwrap_or_default(),
        rule_version: literal_object(store, uri, P_EDGE_RULE)?.unwrap_or_default(),
    })
}

fn iri_object(store: &Store, subject: &str, predicate: &str) -> Result<Option<String>, StoreError> {
    let subject = named(subject)?;
    let predicate = named(predicate)?;
    for quad in store.quads_for_pattern(
        Some(NamedOrBlankNodeRef::NamedNode(subject.as_ref())),
        Some(predicate.as_ref()),
        None,
        None,
    ) {
        let quad = quad.map_err(graph_error)?;
        if let Term::NamedNode(value) = quad.object {
            return Ok(Some(value.as_str().to_owned()));
        }
    }
    Ok(None)
}

fn graph_error(error: impl std::fmt::Display) -> StoreError {
    StoreError::Backend(error.to_string())
}
