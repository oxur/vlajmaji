#!/usr/bin/env python3
"""Mechanical census of the RAW_FIXTURE Rust byte-string source literal."""
from __future__ import annotations

import hashlib
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

source_path = Path(sys.argv[1])
source = source_path.read_bytes()
match = re.search(rb'const RAW_FIXTURE: &\[u8\] = br#"(.*?)"#;', source, re.S)
if not match:
    raise SystemExit("RAW_FIXTURE raw byte-string literal not found")
raw = match.group(1)

class Pairs(list):
    pass

def pairs_hook(items):
    return Pairs(items)

def decode(value):
    return json.loads(value, object_pairs_hook=pairs_hook)

root = decode(raw)
if not isinstance(root, Pairs):
    raise SystemExit("fixture root must be an ordered JSON object")
rows = root

def duplicate_paths(value, path=()):
    found = []
    if isinstance(value, Pairs):
        seen = set()
        for key, child in value:
            here = path + (key,)
            if key in seen:
                found.append(list(here))
            seen.add(key)
            found.extend(duplicate_paths(child, here))
    elif isinstance(value, list):
        for index, child in enumerate(value):
            found.extend(duplicate_paths(child, path + (str(index),)))
    return found

class_counts = Counter()
field_counts = Counter()
root_counts = Counter()
root_rows = defaultdict(list)
all_spellings = set()
eligible_spellings = set()
per_rows = []
contexts = forms = raw_extensions = 0
eligible = excluded = 0
unknown_names = Counter()
row_locations = []
for ordinal, (word, body) in enumerate(rows):
    root_counts[word] += 1
    root_rows[word].append(ordinal)
    all_spellings.add(word)
    root_dups = [word] if root_counts[word] > 1 else []
    nested = duplicate_paths(body, (word,)) if isinstance(body, (Pairs, list)) else []
    child_fields = list(body) if isinstance(body, Pairs) else []
    field_counts["$key"] += 1
    if isinstance(body, Pairs):
        field_counts.update(key for key, _ in body)
        for key, value in body:
            if isinstance(value, str):
                pass
            if key == "t" and isinstance(value, str):
                class_counts[value] += 1
            if key not in {"d", "t", "s", "g", "n", "r", "b"}:
                unknown_names[key] += 1
    else:
        field_counts["$row"] += 1
    eligible_row = (
        isinstance(body, Pairs)
        and not nested
        and isinstance(dict(body).get("d"), str)
        and bool(dict(body)["d"].strip())
        and isinstance(dict(body).get("t"), str)
        and dict(body)["t"] in {
            "bu-letteral", "cmavo", "cmavo-compound", "cmevla",
            "experimental cmavo", "experimental gismu", "fu'ivla",
            "gismu", "lujvo", "nalvla", "obsolete cmavo",
            "obsolete cmevla", "obsolete fu'ivla", "obsolete zei-lujvo",
            "phrase", "zei-lujvo",
        }
    )
    if eligible_row:
        eligible += 1
        eligible_spellings.add(word)
    else:
        excluded += 1
    row_contexts = 0
    row_forms = 0
    row_extensions = 0
    if isinstance(body, Pairs):
        values = dict(body)
        if not nested:
            # Required-field eligibility controls lexical search status only.
            # Valid optional contexts/forms remain projected for excluded rows.
            gloss = values.get("g")
            if isinstance(gloss, list) and all(isinstance(item, str) for item in gloss):
                row_contexts += len(gloss)
            note = values.get("n")
            if isinstance(note, str) or "n" in values and values["n"] is None:
                row_contexts += 1
            reported = values.get("r")
            if isinstance(reported, list) and all(isinstance(item, str) for item in reported):
                row_forms += sum(len(item) in (3, 4) for item in reported)
        row_extensions += sum(key == "g" for key, _ in body)
        row_extensions += sum(key == "b" for key, _ in body)
        row_extensions += sum(key not in {"d", "t", "s", "g", "n", "r", "b"} for key, _ in body)
    contexts += row_contexts
    forms += row_forms
    raw_extensions += row_extensions
    per_rows.append({
        "ordinal": ordinal,
        "decoded_root_key": word,
        "root_value_kind": "object" if isinstance(body, Pairs) else type(body).__name__,
        "ordered_child_keys": [key for key, _ in child_fields],
        "child_field_count": len(child_fields),
        "nested_duplicate_paths": nested,
        "root_duplicate_group_ordinals_so_far": [],
        "policy_eligible": eligible_row,
        "typed_contexts_by_declared_policy": row_contexts,
        "reported_forms_by_declared_policy": row_forms,
        "raw_extensions_by_declared_policy": row_extensions,
    })
    row_locations.append({"ordinal": ordinal, "key": word})
dup_root_groups = {key: ords for key, ords in root_rows.items() if len(ords) > 1}
for row in per_rows:
    row["root_duplicate_group_ordinals_so_far"] = root_rows[row["decoded_root_key"]]
result = {
    "kind": "mechanical fixture-source census; expected values are not product output",
    "source_path": str(source_path),
    "source_sha256": hashlib.sha256(source).hexdigest(),
    "literal_raw_bytes_sha256": hashlib.sha256(raw).hexdigest(),
    "literal_raw_bytes": len(raw),
    "fixture_rows": len(rows),
    "root_key_occurrences": len(rows),
    "root_key_field_count": field_counts["$key"],
    "ordered_root_occurrences": row_locations,
    "root_key_occurrence_counts": dict(sorted(root_counts.items())),
    "duplicate_root_groups": dup_root_groups,
    "child_object_fields": sum(row["child_field_count"] for row in per_rows),
    "non_object_rows": sum(row["root_value_kind"] != "object" for row in per_rows),
    "native_field_counts_including_key_or_row_carriers": dict(sorted(field_counts.items())),
    "native_fields_total": sum(field_counts.values()),
    "string_word_types": dict(sorted(class_counts.items())),
    "all_spellings": len(all_spellings),
    "eligible_spellings_by_declared_policy": len(eligible_spellings),
    "eligible_rows_by_declared_policy": eligible,
    "excluded_rows_by_declared_policy": excluded,
    "contexts_by_declared_policy": contexts,
    "source_reported_forms_by_declared_policy": forms,
    "raw_extensions_by_declared_policy": raw_extensions,
    "unknown_field_names": dict(sorted(unknown_names.items())),
    "per_row": per_rows,
}
print(json.dumps(result, ensure_ascii=False, indent=2))
