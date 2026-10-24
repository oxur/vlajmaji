# CC assignment — A02 S01 lossless raw export normalization

You are Code Contributor for Vlajmaji Project01 Arc02 S01. Execute this initial
assignment, return proposed-done evidence to CDC `/root/vlajmaji_p01_a02_cdc`.
Duncan renewed Autonomous/Expedited through Arc02 on2026-10-24, explicitly
OpenAI gpt-6-luna/high CC; dispatch must honor it, unknown backendattestation is
not confirmation. No substitute. CDC owns design/planning/review; rootread-only.
This preserved initialprompt is immutable after issue; current assignment in
slice-plan.md must match. Stop after sourcecommit/freeze and evidence return.
No appthread/newtask/externalmessage. Routine contributor handoff is authorized.

## Roots, authority and ownership

Source `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/vlajmaji`,
feature/project01-full-jbotci-import, clean baseline
`d1e1c928f7999f2220b046cc5a5817f0529312ca`. Planning root
`/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning/project01-multi-source-dictionary`.
Owning slice `arc02-reproducible-jbotci-import/slice01-lossless-export-normalization`.
Projectdirective02 and arcplan authorize scope; sliceplan's BindingAPI/Rawrules/
CLI/Verification sections are the detailed current implementation contract.
All7ledgerrows open. Current source includes fivecrates/crates/cli/umbrella0.3.0;
do not regress relocation/releasework. Bauskesiblingread-only c5801ae.
Main/oldA01source/research/preservedhistory are outside ownership.

CC exclusively owns source mutation/index until frozen return. Allowed exactpaths:
`crates/lexicon/src/jbotci.rs`(new),`crates/lexicon/src/lib.rs`(module/exports only),
`crates/lexicon/tests/jbotci_contract.rs`(new),
`crates/lexicon/examples/import_jbotci.rs`(new),
`crates/cli/src/main.rs`,`crates/cli/Cargo.toml`,`crates/cli/tests/jbotci_contract.rs`(new),
`Cargo.lock` only if directalreadyworkspace dependency classification requires it,
`docs/jbotci-import.md`(new),`docs/migration.md`,`crates/lexicon/README.md`,
`crates/cli/README.md`,`README.md`,`CHANGELOG.md`.
No existing legacyparser/dictionaryvalidator/store/search/umbrella implementation,
newexternaldependency/framework/toolchain/version/CIchange, fullrawbundle or
optionalcorpuspackage. Report needed consequentialwidening before edits.
Only planningwrite `S/artifacts/cc-execution.md`: intake/readback, exactattempts,
selfreview, sevenrowproposals and fullscope/artifactinventory. CDC owns
canonicalplanning/ledgers/closingrecords/commits. You may use task-owned
/tmp/vlajmaji-p01-a02-s01-cc-* outsidecaller/scratch; unique package AND binary
names, sourcehash/offline lock. No source/planning broadcleanup or processkill.

Buildcache exclusively transferred toArc02:
`CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target`.
EveryCargo/Make invocation explicitly sets it, and use `/tmp/vlajmaji-p01-a02-cargo`
(or CARGO=/tmp/vlajmaji-p01-a02-cargo forMake), +1.98.0/locked/offline.
Serialize all builds/imports/scans/sourceedits/SQLite snapshots; one writer.
Use login:false. Free~2.5GiB; recheck before largeoutputs; no cachedeletion.
Allrepo writes/build/network need scoped exec escalation in this sandbox;
ordinarysandboxfailure is not autoreviewrejection. Local scopedcommit authorized,
no push/PR/merge/mainintegration/release/publication/authLensisku/schedule.

## Required reading and intake manifest

Load actualtext in this context, recover alltruncations. Before dependentedits,
record source/state, exactloadedextents/querycoverage and concise source-cited
contractreadback in cc-execution.md. Filehash/headings/anothercontext'sread claims
are not reading; receipts don't prove comprehension. Resolve missing/conflicting
contract before affectedwork, then proceed without repeatedapproval.

Skillbase `/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills`.

| Order/input | Scope | Purpose |
| --- | --- | --- |
| Source AGENTS.md; planning ../AGENTS.md; thisprompt/sliceplan/ledger | required-full/currentbaseline and issuedplanningcandidate | Ownership/gates and completecurrentcontract |
| P/cdc-directive02.md | required-full | Renewedfullarc authority/resource/preservationboundaries |
| P/project-plan.md CurrentArc02authority+Definitionofdone+Boundaries+roadmap; P/design.md Fullimport+LogicalLayers+12area table+Querycontract | required-section fullnamedsections | Currentarc vs futureproject obligations |
| A/arc-plan.md + A/artifacts/readiness.md + A/artifacts/raw-census.json | required-full/readiness currentcandidate; censusrequired-data completeobject | Currentinputdesign/resource/rawreference; query not semanticfullcorpusread |
| collaboration-framework/SKILL.md; project-management/SKILL.md+guides/README.md | required-full | Selectedworkflow/usableincrements/Expedited |
| PMguides02canonical sections perslicedocs/iterations/verificationrecords/retros/existingrecords;03topdown sections per-sliceopen/implementationready/iterations;04closingslices full | required-section except04full | Preservedprompt/intake/return/bubbleup |
| engineering-methods/guides/01-engineering-methodology.md NotesforCodex+SharedInvariants+TwoContributor+AutonomousMode;07-implementation-prompt-authoring.md full | required-section/required-full07 | Contributor/readiness/intakecontract |
| work-verification/SKILL.md + guides01–05 full; testing/SKILL.md + guides01/03full | required-full | Evidence/independence/nonvacuousrealgates |
| rust-guidelines/SKILL.md + guides/11-anti-patterns.md | required-full | Rustsafety/invariant/errorfloor |
| Rustguides02API API02–04,18,23,26–29,39;03errors EH01–04,07–14;05types TD05–07;12structure PS01–07,11–15,25–26 | required-section exactnamedpatterns/fullbodies | AppliedSansIO/typederrors/validation/serde/workspacechoices |
| Rust14-cli-tools/README.md;03-error-handling.md;02-argument-parsing CLI05/06/10/11;04-output-and-ux CLI27 | required-full firsttwo;required-section lasttwo | Existingclap/JSON/ExitCode/nooverwriteentrypoints |
| srclexicon lib.rs complete; dictionary.rs1–1140 + validate1240–1620 and validate_evidence end; registry.rs1–520; clisrcmain/Cargo; lexiconCargo; docs/dictionary-model.md | required-full lib/main/manifests/docs;required-section dictionary/registry | Actualraw/serde/helper/publicmodel/API shapes; preserve oldparsers |
| P/artifacts/research/README.md; copiedplanningreconnaissance + lexical-model-import report | required-full README;required-section reconnaissance jbotci recheck and modelreport Optionaljbotciprobe+Limits | Historicalsource/codeevidence vs actualpin and acceptedboundaries |
| /tmp/p02a02-jbotci dataREADME, metadata full; import.rs1–240/320–510; build.rs238–304; THIRD-PARTY-NOTICES238–255 | required-full firsttwo; required-section remainder | Supplierexplicitexport/reduction/metadata/code-vs-content witness; not full337KBnoticeclaim |
| Fullraw+metadata in taskdatahome below | required-data | Allrows/allfields/class/missing/null/type/nestedgloss/place/user/ID/duplicates/eligibility/counts; exactqueriesbelow |
| OldA01retrospectives and originalarchives | reference-only unless regressionrequires pinnedarchive | History not currentcorpusoracle |

Required-data: exactlocalhome
`/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/data/jbotci/3d9fd41d9ed32de26311ee27ec86b6b165985aa1/`.
SHA/bytes inarcplan/census. Beforeedit compute/inspect fullrootpopulation, union and
counts/types for everyfield; all17classes/counts; every nestedgloss/place/user
key/type/presence; duplicatecomplete/nativeID rows;9empty/1wiki, wholeeligible/
exactwordcounts. Inspect completeactualfarvi2/cpana/si'e/ma2 rows and wiki body
(using correct word/word_type keys). Structuredqueries can computerunwithout
33526rowdump; retain commands/exits/denominators/results and errors inexecution.
Actualraw byte/lexeme mapping proof later coversall33526/278254, not samplewitnesses.

## Implementation spine and applied design

The sliceplan contains resolved consequentialdecisions; follow it, not supplier's
lossy import. Recommend sequence:

1. New private module: constants/options/typederrors/report/immutable result,
metadata subsetparser + upfront hash/options validation. NoI/O/Rust API takes
borrowedbytes/returns ownedresult (API02/04/23/27/39;EH01/04/07–14). Existing
serdecore is deliberateAPI18SHOULD exception; nonewserdefeatureorerrorcrate.
2. Parse rootarray RawValue rows, orderedfieldvisitor that retainsduplicatekeys;
retain scalarrow via `$row`; fullrawcapture and exactmetadatacapture. Reuse
RawCapture/NativeField/Presence/RawJson/RegistryDocument/DictionaryDocument,
publicvalidation before result. Keep currentlegacy parse_object/checkpaths unchanged.
3. Build single-pass row IDs/fullperoccurrenceentries/assertions, exactsource/
distributor/contributorbindings, conservativefieldmap/SourceReportedForms,
contexts/etymology/extensions, expliciteligibility/diagnostics. Preserve allknown
rafsi includingfarv/maz/duplicates; no selectbest/inference. Prebuildmapping/sets,
no O(N²) perrow lookups. Pinnedsourceclass policy inclnalvla/obsolete;9empty/wiki
excludedbut retained; unsupported future row stays rawexcluded. Null/missing/empty
and exactlarge-number/rawlexeme do notcollapse. See sliceplantable for14fields.
4. Build collection/component/Available selectedrelease(all33526assertions),
versionednormalizer/accounting/diagnostics, pinnednotice/Heldredistribution,
metadatahistory/rawrefs. Validate Dictionary::from_document, returnreport alongside.
5. Tests then localCLIimport/check thinwrappers inactualcrates/cli path; direct
lexicondependency. Deterministic freshoutput/create_new after completevalidation;
checkregen+fullmodelcompare/no-write, countmismatchtruthfulimport/checkreject.
Successreportstdout; structuredruntimeerrorsstderr/nonzero/emptystdout.
6. Publicexample/docs/fieldmap and actualfullpopulationrun, uniqueoutsidecaller,
completefieldoracles and exact gates. Current0.3.0/newcapability separatepublished0.2.0.

Test difficultactual ma18910/id88840 first within implementation: lower-score
alternative retainsexperimental_rafsi maz even while ma18911/id1791 also survives;
sourcebothrawfields/identity/score/user reach normalized/reloadedoutput. Then full
population checks allrows. This is ordinarydelivery, no separatelycommissioned
investigation. If design/API/source mismatch arises, stopaffectedwork and askCDC;
do not invent alternativepublicbehavior. Privatehelpernames/layout localdiscretion.

## Acceptance checks and return

Detailedtestmatrix in sliceplan covers all7ledgerrows. Required decisive results:
fullraw17163482B/33526rows/278254orderedfields, metadata514B exactcapture equality;
33526normalizedassertions/33526perrowentries,33516eligible+10excluded,
31248exacteligiblewords, supplier33526/31248 separatelylabelled;238contributors;
39721glosscontexts,28280notes,8356jargoncontexts,2085etymologies, placekeyword
5193nativeobjects separate. Rawscore/rafsi/selmaho/keywordmeaning and realname
missingness validated through independentactualsourcecomparison, not selfroundtrip.
Syntheticduplicate row/nativeIDpositive remains2distinctoccurrences; unknown/wiki/
nonobject/repeatedkeys retained. Incorrectoptionalfield gets diagnosticwithout
hiding otherwisevalidword; incorrectcriticalfield excludesonlyitsrow. Negative
scorevalid. Allcriticalnegativecases pairedwithvalidcounterparts.

APInegativecases exactvariantclass: wronginputhash/options/metadataformat/root/
syntax/UTF8/metadata duplicates; no panic/mutation. Countmismatch metadata (with
correctnewpinSHA) importsallrows/reportfalse but CLIchecknonzero accounting.
CLI actualsuccess/checkfreshrelease/changedrelease, occupiedoutput(nooverwrite),
spacepaths, missinginput; no-writecheck hashguards. All16lexicalclasses and source
percolumnformstanding preservepositivecases. Existingoldarchive/regression/fixtures
remain exactly scoped; no JSONcounts-as-fidelity shortcut.

Use makeformat thenlint thencheck, explicitCARGO_TARGET_DIR+launcher everytime;
make test-no-default, make test-all-features, affectedumbrellaoutsideno-default/
allfeature gates. These are actualcommands/populations, not promisedscripts.
No publicrawdata package. Before commit inspectdiff/include/packagefilelist;
commit only exactchangedallowlistedsourcepaths with git commit --only and trailers:
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>

Return promptidentity/sourcecommit/clean-or-protectedstate, exactpaths/gates/counts/
failedattempts/intake/readback, sevenrowCCproposals, actualfullpopulationcomparison,
source-vs-deliverydiff, selfreview, limitations and S02carryover. Sourcefreeze
explicit; do not advanceS02orclaimindependentacceptance. Write only attributed
cc-execution.md inplanning; CDC will create canonicalclose/verifier/retro after
actualwork/review. NewlyunresolvedAPI/scope/acceptance questions belongtoCDC.
