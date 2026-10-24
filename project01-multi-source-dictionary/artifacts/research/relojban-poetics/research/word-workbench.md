# Community tools and a possible poetry workbench

[Project](../README.md) · [Phonological space](phonological-space.md) · [Sources and reading](../bibliography/README.md)

**Date:** 2026-10-22, America/Chicago. **Historical survey status (original wording):** initial tool survey and product ideas. Documentation and selected web pages were inspected; no package was installed, compiled, benchmarked, or exercised with test inputs. No Rust implementation has begun.

**Current project status (2026-10-22):** Arc01 now implements a bounded phonology inventory, named within-class representative comparisons, and contextual/prosody foundation; see the [phonology inspection tools](../README.md#phonology-inspection-tools) and [source-coverage note](phonology-coverage.md). This does not implement the proposed dictionary/morphology/poet-facing workbench or perform the proposed comparative tool evaluation. The historical no-install/no-test statement above remains specific to the initial survey; no surveyed tool is claimed as installed, tested, or currently re-evaluated here.

## Existing capabilities

| ID | Tool and primary reference | Documented capability | Verification and limits |
| --- | --- | --- | --- |
| T01 | [Lensisku](https://lensisku.lojban.org/en), [Jbovlaste notice](https://jbovlaste.lojban.org/), [Sutysisku](https://la-lojban.github.io/sutysisku/) | Dictionary infrastructure and lookup. Jbovlaste identifies Lensisku as its replacement; Sutysisku describes full-text search and offline use. | Notice, Lensisku page shell, and Sutysisku description inspected. Searches and modern-browser offline behavior not exercised. Sutysisku's offline description references an older browser caching mechanism. |
| T02 | [Sozysozbot's Jvozba](https://sozysozbot.github.io/sozysozbot_jvozba/sozysozbot_jvozba.html), [repository](https://github.com/sozysozbot/sozysozbot_jvozba/) | Browser compound maker and decomposer; accepts roots or rafsi; exposes an experimental-rafsi option. | Live form text/examples and repository README inspected. No generation was performed. |
| T03 | [latkerlo-jvotci](https://github.com/latkerlo/latkerlo-jvotci), [Rust documentation listing](https://docs.rs/crate/latkerlo-jvotci/latest) | Compound creation/decomposition, Rust library and interactive CLI, plus Python/JavaScript and a browser interface. README documents deliberate differences from reference morphology. | Repository README and indexed Rust documentation inspected; direct docs.rs opens and the browser demo returned fetch failures. Exact dependency version and behavior need a pinned evaluation. Repository declares MIT. |
| T04 | [vlazba](https://github.com/La-Lojban/vlazba), [published API documentation](https://docs.rs/vlazba/latest/vlazba/index.html) | Rust compound generation, decomposition/reconstruction, custom rafsi maps, and gismu-generation functionality. Current repository README describes full enumeration or score-optimal generation. | Repository README and indexed API documentation inspected. Published indexed docs and repository README describe different versions/API signatures; do not mix them. No conformance testing. Repository displays GPL-3.0. |
| T05 | [La-Lojban/lojban](https://github.com/La-Lojban/lojban) | JavaScript/TypeScript package combining dictionary/rafsi lookup, compound candidates and scores, splitting, morphology, and parsing. | README/API descriptions inspected; no execution. Useful comparison implementation. It documents a Lensisku-derived dictionary dump. |
| T06 | Cowan, *The Complete Lojban Language*, [§4.6 rafsi](https://la-lojban.github.io/uncll/romoi/xhtml_section_chunks/section-rafsi.html), [§4.10 lujvo considerations](https://www.lojban.org/publications/cll.before-20160606/cll_v1.1_xhtml-section-chunks/section-lujvo-considerations.html) | Rules and rationale for compound-building alternatives. | Relevant passages inspected. Lojban reference, not proof that every rule/tool choice matches Relojban. |

All T entries are primary software/project documentation or a primary language reference. They are independent new sources; none resolves an archived citation marker.

## A useful distinction for composition

**SOURCE CHECK (T06):** Rafsi are assigned compound-building forms. A root may offer several, yielding different legal compound forms for the same underlying ordered components. The dictionary scoring convention selects a preferred form, while the reference permits other legal alternatives according to sound and intelligibility.

This gives the composer a promising operation: keep the intended components fixed and compare alternative compound realizations for sound, stress, and line length. A conventional morphology score is not a poetic quality score.

Rafsi are not simply freely invented English-style prefixes and suffixes. A compound builder handles word formation; its output still needs a definition and place structure. Substituting a vowel can instead produce another existing word, a legal but unattested form, multiple words, or an invalid string. The tool should display which occurred and retain the selected language rules and dictionary identity.

## Sound search: what is established and what is proposed

This survey confirms dictionaries and compound engines. It has not confirmed an existing interface that combines constrained vowel/diphthong substitution, phonological-distance ranking, stress filtering, and semantic review in the proposed way. That is a limitation of this initial survey, not evidence that no such tool exists.

**NEW PRODUCT PROPOSAL:** A workbench could answer queries such as:

- Keep a consonant frame fixed and vary one nucleus; show attested matches with definitions.
- Hold one vowel or stress pattern fixed and search for contrasting consonantal features.
- Generate legal rafsi alternatives for chosen components, retaining more than the dictionary-score winner.
- Find words near a target in a declared feature model, or related by a named transformation.
- Filter results by canonical syllables, performed syllables where available, stress, word class, and lexical status.
- Preserve chosen candidates, rejected alternatives, pronunciation, source/version, and notes with a poem draft.

For illustration, `br[aeiou]di` is a spelling-pattern query specification, not a tested command in any surveyed product. Diphthong exploration should operate on parsed nuclei and permitted word shapes; inserting a second vowel can change syllabification or word class. Explain the operation and resulting analysis rather than treating all character replacements as equivalent.

## A plausible Rust direction

Begin by evaluating T03 and T04 against the chosen Relojban grammar and pinned lexical data. They are candidate foundations, not adopted dependencies. Their existing morphology work could support a separate layer for phonological search and composition. Inspect the exact version, morphology settings, source terms, and data provenance before selecting one.

The poet-facing result should place spelling, meaning/place structure, component analysis, pronunciation/stress, lexical status, and transformation explanation together. Keep conventional compound ranking, perceptual-model scores, and the composer's aesthetic annotations separate.

The next useful action is a small comparative tool trial with a handful of fixed examples and retained outputs, including differences and invalid cases. This idea is recorded for discussion; it is not an implementation plan or authorization to build a new application.
