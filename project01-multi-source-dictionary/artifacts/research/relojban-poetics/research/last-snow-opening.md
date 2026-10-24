# The last snow — opening resources and first treatments

[Project](../README.md) · [Agreed plan](../.worktrees/planning/project02-wordcraft-reconnaissance/project-plan.md) · [Session notebook](../notebooks/2026-10-23-01-opening-investigation.md)

**Date:** 2026-10-23, America/Chicago. **Author:** Sofie, with Duncan directing the inquiry. **Status:** exploratory, authored candidate packet; no idiomatic, reader, or sung validation. This is planning-stage research, not an accepted framework slice or a completed experiment. English original unchanged.

## Decision and boundary

Choose a resource-supported direction for a first spoken/sung trial of:

```text
on the old plum
the snow     is opening
```

The agreed limit was these two lines, roughly twelve lexical entries, a few constructions, a small poetry sample, and one compound tool. This pass inspected twelve selected lexical entries, three broad construction families (description/relative structure, modifiers/compounds, event contour), two historical short poems and one contemporary bilingual poem, plus four inputs to one hosted compound tool. Ten word-level inspections used the existing local Rust CLI; no new dependency or code was added. One craft essay remained inaccessible. This is not a controlled comparison: candidates change multiple semantic/grammatical features, and no blinded reader or listening observations exist.

## Resource map and reading status

| ID | Primary resource or direct evidence | Inspected scope and use | Limits |
| --- | --- | --- | --- |
| P01 | [Relojban reference](https://www.relojban.org/book/xhtml/index.html), displayed 0.1-alpha | §5.2 and §5.12 selected paragraphs; §6.2, §8.1, §10.10 and §15.5 read; §15.4 scope paragraphs checked; earlier morphology checks supply long-rafsi background. Description, relative clause and aspect choices. | Live presentation is experimental, with inherited material. New chapters were read through verified HTTPS retrieval, not added to the earlier pinned phonology acquisition. Full modern Lojban/Relojban reconciliation and sentence parsing remain open. |
| P10 | LLG [root list](https://www.lojban.org/publications/wordlists/gismu.txt), header 1994-09-26 | Nine selected roots; meanings and relevant argument roles. | Historical baseline; not a complete contemporary lexicon or usage corpus. |
| P11 | Vlasisku entries [ricrprunu](https://test-vs.lojban.org/ricrprunu), [flaume](https://test-vs.lojban.org/flaume), [rutrprunu](https://test-vs.lojban.org/rutrprunu) | Direct dictionary display of three borrowed words, retrieved successfully with the system HTTP client after web-tool failures. | Evidence for the displayed entries, not independent botanical precision, dictionary consensus, or Relojban adoption. |
| P12 | xorxes, [pemcrxaiku](https://mw-test.lojban.org/papri/pemcrxaiku), first two numbered poems, attributed February 1994 | Poem 1 uses a weeping-tree image; poem 2 places summer recollection amid snow. Direct short-text reading. | Historical Lojban; neither a representative corpus nor proof of current rules. German parallel text has encoding damage; do not use it as a reliable gloss. Remaining collection not analyzed as part of this sample. |
| P13 | Filomena / la xrula, [The Doorway](https://thebrokenflower.neocities.org/poetry?title=The%20Doorway), linked from [lojban.io](https://lojban.io/) as published 2025-06-28 | Complete English and Lojban columns read in the rendered author page; ordinary HTML retrieval initially exposed only the archive shell. | Primary artistic practice, not normative grammar. Bilingual columns reorganize expression; experimental-looking constructions were not adopted into our candidates. No full grammatical analysis or audio assessment. |
| P14 | Arnt Richard Johansen, [How to write poetry in Lojban](https://arj.nvg.org/lojban/poetry.html) | Located as an author-hosted craft resource. | Web opens and a 20-second direct request timed out. Search excerpt only; close reading deferred, and its prescriptions were not adopted. |
| T03 | [latkerlo-jvotci hosted interface](https://latkerlo.github.io/latkerlo-jvotci/), [project documentation](https://github.com/latkerlo/latkerlo-jvotci), [differences](https://github.com/latkerlo/latkerlo-jvotci/blob/main/differences.md) | Four live interface inputs below; relevant README sections and differences file read. Generation and decomposition support a morphology trial. | Hosted JavaScript behavior only; deployment revision not pinned. Rust API/package not installed or tested. Documentation declares deliberate differences from other analyzers, so this is not Relojban conformance certification or dependency selection. |

## Twelve lexical entries

**SOURCE CHECK — concise working descriptions, not substitute definitions:**

| Entry | Role in this opening |
| --- | --- |
| `snime` | snow substance |
| `xrula` | flower of plant/species |
| `tricu` | tree of species/cultivar |
| `slabu` | familiar to observer; contextual age reading possible |
| `citno` | youthful by a standard |
| `cpana` | resting atop a support, relative to gravity/frame |
| `kalri` | open passage granting access |
| `banro` | growth in size/form |
| `farvi` | development through stages |
| `ricrprunu` | broad plum/peach/cherry/apricot/sloe tree entry |
| `flaume` | plum fruit entry |
| `rutrprunu` | broad corresponding fruit entry |

**INTERPRETATION:** `slabu` is not simply an error for an aged tree: its source note permits age through context, while making familiarity/observer structure visible. `to'e citno` makes the age opposition explicit. These can suggest different trees in a poem. `ricrprunu` is broader than plum; our scene must supply the intended tree, or a later lexical refinement must narrow it. `flaume` alone does not name a tree. Growth/development roots do not automatically express a bud unfolding.

## The decisive descriptive distinction

**SOURCE CHECK — P01 §6.2:** In the inspected presentation, `le` allows a specific referent described as X without asserting that description is true; `lo` commits to the description. Example 6.7 explicitly illustrates correcting an initial identification. This source-specific distinction gives us a way to consider `le snime` for something first described as snow and later identified as flower. It is not a claim that all current community grammars treat descriptions identically.

**NEW POETIC PROPOSAL:** Give a referent a provisional visual description, hold it across a clause or pause, then let a later predicate reclassify it. This could preserve delayed recognition. We have not observed a reader experience that effect.

**SOURCE CHECK — P01 §10.10:** `co'a` selects a beginning; `ca'o` selects an event in progress. Neither supplies an unfolding action by itself. In particular, a continuative marker on a state of openness is not automatically the process of opening.

## Three learning treatments

All three are **new, provisional sketches**, not source quotations or publication translations. Their explicit terminators expose scope for review. The line breaks below are working phrasing; they do not establish musical rests or idiomatic lineation. `to'e citno` selects aged rather than familiar; replacing it with `slabu` is a later meaningful variant, not a synonym substitution. Each use of `ricrprunu` provisionally means the plum in our scene, with the dictionary breadth retained.

### A — snow beginning to open

```text
le snime poi cpana le to'e citno ricrprunu ku ku'o
cu co'a kalri
```

**Literal structural gloss:** that described as snow, which rests atop the aged Prunus-type tree, begins to be open as a passage.

**Places:** the relative clause supplies the described snow as `cpana` x1 and tree as x2; gravity/frame x3 is omitted. The main relation puts the same referent in `kalri` x1; access destination x2 and accessor x3 remain implicit.

**Intended reading:** snow begins opening, allowing flower recognition to remain unspoken. **Cost:** extending passage-openness to a blossom is a deliberate metaphor, and beginning to be open is not an exact rendering of gradual unfolding. The apparent snow could be read as an actual passage. **Checks:** lexical roles and source construction patterns partly supported; full grammar and idiomatic metaphor unverified.

### B — a coined snow-flower

```text
le snimyxrula poi cpana le to'e citno ricrprunu ku ku'o
cu co'a kalri
```

**Proposed local definition:** `snimyxrula`: x1 is a flower of plant x2 whose appearance evokes snow. This is a proposed two-place definition, not a retrieved dictionary definition. Its equivalence to any existing entry has not been searched or established. Under this intended definition, short and long forms keep the same meaning; changing rafsi is a sound/form choice.

**Literal structural gloss:** the snow-resembling flower, resting atop the aged Prunus-type tree, begins to be open as a passage. **Places:** `cpana` and `kalri` as in A; the coinage's plant x2 is not explicitly equated with the supporting tree.

**Intended reading:** let one word carry snow and blossom together. **Cost:** names the flower before the predicate, weakening delayed recognition; passage metaphor remains. The chosen appearance relationship is authored, not computed by the tool. **Checks:** compound decomposition observed in T03; semantics proposed; sentence grammar and idiom unverified. Short `si'erxrula` is an observed form alternative, not an automatically superior poetic choice.

### C — the apparent snow is blossom

```text
le snime poi cpana le to'e citno ricrprunu ku ku'o
cu xrula
```

**Literal structural gloss:** that described as snow, resting atop the aged Prunus-type tree, is a flower/blossom.

**Places:** the relative clause has the same support/frame roles as A. The main predicate gives the referent `xrula` x1; its plant/species x2 is omitted, so connection to this tree is inferred rather than explicitly asserted.

**Intended reading:** the noun description can survive a change in our recognition; flower arrives at the predicate. **Cost:** replaces opening movement with identification and makes the eventual flower explicit. It may resolve what the English continues to leave open. **Checks:** closely motivated by the inspected descriptive example, with lexical roles partly supported; complete grammar, actual reader timing and idiom unverified.

## Comparison and the next decision

| Criterion | A | B | C |
| --- | --- | --- | --- |
| Concrete scene | Tree, apparent snow, onset of passage-openness | Tree and snow-resembling flower | Tree, apparent snow, then flower |
| Delayed recognition | Potentially sustained; flower unnamed | Flower available inside subject | Held until main predicate, then explicit |
| Main semantic commitment/cost | Passage metaphor | Proposed compound definition plus passage metaphor | Reclassification; no unfolding process |
| Morphology evidence | Listed roots and borrowed-tree entry; sentence not parsed | T03 long form decomposes as intended; no dictionary attestation check | Listed roots and borrowed-tree entry; sentence not parsed |
| Speaking/singing evidence | Not observed | Not observed | Not observed |

**INTERPRETATION:** C is the strongest first *recognition* probe; A is the useful contrast for retaining an opening event. B is a useful probe of compound formation and sound, with the explicit risk of revealing the image early. These are analytical judgments, not reader-study findings or selection of a final translation.

**RESOURCE LIMIT / STOP:** Three annotated sketches are now inspectable, but none is certified idiomatic. The immediate missing evidence is a knowledgeable reader's judgment of these specific constructions and metaphors, followed by actual spoken/sung trials. Additionally, the available plum-tree entry is broad, and the existing tool cannot resolve its canonical syllabification under the pinned phonology's source tension. Stop this initial resource pass here; do not add more engines or widen the corpus to manufacture certainty. Duncan can choose A/C for an oral trial, or prioritize the plant term. No outside messages or recordings were made.

## One compound tool — four observed inputs

Live browser visit on 2026-10-23. Interface started with brivla/cmevla/easy-mode buttons and a source field. No settings were changed; the result table identified itself as hard-mode. The salient displayed results below are transcribed from the accessibility states, not source code calls:

| Input | Displayed result |
| --- | --- |
| `snime xrula` | STANDARD: `si'erxrula`; ALLOW_Y: `si'erxrula`; FORCE_Y: `snimyxrula` |
| `xrula snime` | `rulsi'e` in a single result row, without policy labels |
| `snimyxrula` | LUJVO; `[snim, y, xrula]`; `snime xrula` |
| `xrulysnime` | LUJVO; `[xrul, y, snime]`; `xrula snime` |

There was no negative control or wider conformance suite: four successful illustrative inputs cannot establish general correctness. No additional compound engine, external package or Rust dependency was exercised/adopted. The default/extended behavior caveat in the project's differences file remains relevant to any later dependency evaluation.

## Existing Rust observations

All ten commands had form `target/debug/relojban prosody WORD --kind KIND`, exited 0 and produced empty stderr. This is phonological inspection with caller-declared word kind, not word-formation or sentence parsing. No rebuild was performed. Capitalization below only illustrates reported primary stress; it is not canonical spelling. The selected representative `x` and `c` are not English letter-sound instructions; consult the pinned inventory for IPA before reading aloud.

| Word | Declared kind | Tool-derived canonical plan |
| --- | --- | --- |
| `snime` | `gismu-lujvo` | SNI-me |
| `xrula` | `gismu-lujvo` | XRU-la |
| `citno` | `gismu-lujvo` | CIT-no |
| `slabu` | `gismu-lujvo` | SLA-bu |
| `cpana` | `gismu-lujvo` | CPA-na |
| `kalri` | `gismu-lujvo` | KAL-ri |
| `ricrprunu` | `brivla` | unresolved; primary nucleus index 1 |
| `snimyxrula` | `gismu-lujvo` | sni-my-XRU-la |
| `si'erxrula` | `gismu-lujvo` | si-'er-XRU-la |
| `rulsi'e` | `gismu-lujvo` | rul-SI-'e |

`ricrprunu` reports `syllables.source-length-tension` in `crpr` (original bytes 2..6), tied to §§3.6/3.7; no canonical syllables were fabricated. Its primary nucleus index is still reported separately. The limitation is not evidence that the dictionary word is invalid or unsingable. The other displayed divisions are rule-based proposals, not audio observations. `si'erxrula` has four nuclei just as `snimyxrula` does: a visually shorter form is not necessarily fewer syllables. `rulsi'e` has three planned nuclei; it also reverses the components and therefore is not a synonymous shortening of the other direction.

### Retained outputs for the three new consequential cases

```sh
target/debug/relojban prosody ricrprunu --kind brivla
```

```text
source	P01-S20261022@0195f4b5b1f182ce8e21381210f766b326fb987f
policy	standard-onsets-v1
kind	brivla
result	phonological inspection; morphology unverified
original	ricrprunu
canonical	ricrprunu
written_pause_before	false
written_pause_after	false
required_pause_before	false
required_pause_after	false
comma_positions	none
capital_positions	none
start	end	spelling	kind	stress_countable
1	2	i	vowel:i	true
6	7	u	vowel:u	true
8	9	u	vowel:u	true
diagnostic	analysis.source-length-tension	3.6;3.7	2..6	last-three-witness=none
prosody_policy	stress-v1
syllabification_policy	syllables-v1
syllable_mode	canonical
syllabification_resolved	false
syllable	start	end	spelling	canonical	nucleus_index	nucleus	countable
syllable_limitation	syllables.source-length-tension	3.6;3.7	2..6	source-length tension in crpr
primary_nucleus	1
stress	0	optional-secondary
stress	1	primary
stress	2	optional-secondary
written_nuclei	3
performed_nuclei	3
countable_nuclei	3
realization_policy	buffer-plan-v1
realization	unmeasured caller-annotated plan; no audio inferred
```

```sh
target/debug/relojban prosody si'erxrula --kind gismu-lujvo
```

```text
source	P01-S20261022@0195f4b5b1f182ce8e21381210f766b326fb987f
policy	standard-onsets-v1
kind	gismu-lujvo
result	phonological inspection; morphology unverified
original	si'erxrula
canonical	si'erxrula
written_pause_before	false
written_pause_after	false
required_pause_before	false
required_pause_after	false
comma_positions	none
capital_positions	none
start	end	spelling	kind	stress_countable
1	2	i	vowel:i	true
3	4	e	vowel:e	true
7	8	u	vowel:u	true
9	10	a	vowel:a	true
prosody_policy	stress-v1
syllabification_policy	syllables-v1
syllable_mode	canonical
syllabification_resolved	true
syllable	start	end	spelling	canonical	nucleus_index	nucleus	countable
syllable	0	2	si	si	0	i	true
syllable	2	5	'er	'er	1	e	true
syllable	5	8	xru	xru	2	u	true
syllable	8	10	la	la	3	a	true
primary_nucleus	2
stress	0	optional-secondary
stress	1	optional-secondary
stress	2	primary
stress	3	optional-secondary
written_nuclei	4
performed_nuclei	4
countable_nuclei	4
realization_policy	buffer-plan-v1
realization	unmeasured caller-annotated plan; no audio inferred
```

```sh
target/debug/relojban prosody rulsi'e --kind gismu-lujvo
```

```text
source	P01-S20261022@0195f4b5b1f182ce8e21381210f766b326fb987f
policy	standard-onsets-v1
kind	gismu-lujvo
result	phonological inspection; morphology unverified
original	rulsi'e
canonical	rulsi'e
written_pause_before	false
written_pause_after	false
required_pause_before	false
required_pause_after	false
comma_positions	none
capital_positions	none
start	end	spelling	kind	stress_countable
1	2	u	vowel:u	true
4	5	i	vowel:i	true
6	7	e	vowel:e	true
prosody_policy	stress-v1
syllabification_policy	syllables-v1
syllable_mode	canonical
syllabification_resolved	true
syllable	start	end	spelling	canonical	nucleus_index	nucleus	countable
syllable	0	3	rul	rul	0	u	true
syllable	3	5	si	si	1	i	true
syllable	5	7	'e	'e	2	e	true
primary_nucleus	1
stress	0	optional-secondary
stress	1	primary
stress	2	optional-secondary
written_nuclei	3
performed_nuclei	3
countable_nuclei	3
realization_policy	buffer-plan-v1
realization	unmeasured caller-annotated plan; no audio inferred
```

## Retrieval and preservation notes

Web-tool opens failed for several accessible pages; direct HTTPS retrieval with normal certificate checks recovered the grammar and dictionary entries. Default sandbox DNS failed first; the approved public-source retrieval succeeded after network escalation. The craft essay timed out. The author archive needed rendered browser reading because its raw HTML was a shell. None of these failures is silently treated as a read. Retrieved bytes stayed in temporary storage; the hashes below identify inspected inputs but do not substitute for a frozen acquisition or guarantee future live bytes.

- `lexicon`: SHA-256 `f2fc4cff4f265ad02717d56733b4627ffa78c616db78dcaa44caab694e1dd5b7`.
- `plum-tree`: SHA-256 `7a18cb4b0c06a296ec3b672d6a7c010a761f3c02a66c71e9abbaf7f8bd4b8e82`.
- `plum-fruit`: SHA-256 `332c24e36d723b5447320b09b5e14388a3a648458ca95778cf003f312bbfc45e`.
- `prunus-fruit`: SHA-256 `c50782e97dd1e7b74148a835b100ca6fad37d658d9919781bd474f5bc9acfe9b`.
- `sumti`: SHA-256 `1faa96b97190827c7aabf198842dade21f5f4e4fc45e7a41621f9f9a801ae027`.
- `relatives`: SHA-256 `3aa57f5c279fa1ffdd0c5675f158e61a5d8525205902b4161185544244521150`.
- `aspect`: SHA-256 `e57f263d44a24d189707e39704088ffefb22db2e58e89d97571a3d1c845c91d9`.
- `haiku`: SHA-256 `cfbfd9b8483e7c70adb87f38755504fb147e5e05576675f327d92aaaeea50d28`.
- `differences`: SHA-256 `c6f1b11908c9db4595797d36ebb2407695406e14e1743a0e3b429d804c02da86`.

English complete-file SHA-256 before/after this pass: `9051cdb850e55400bc35e0271aaea34b6aee7240dfb4db102e4d4487968b5ae7`. The accepted poem and 2026-10-22 notebook were not edited. New claims and candidates live here separately.
