# PR3 repair02 — independent local verification

Locally accepted / ready for hosted CI at `f3551bc1a1e6be3d27f992f158310f8c7b3db4d3`, directly atop pushedc49 on feature/project01-explicit-store-verification. Exactly crates/search/tests/dictionary_contract.rs changed (22added/4removed lines), both required trailers, clean source. TEST-ONLY transport correction; no runtime/API/model/fixture/manifest/lock/dependency change. LunaHigh CC authored, Sol6.1High CDC independently reviewed/executed (requested/configured, no backend attestation). Root alone pushes/follows PR https://github.com/oxur/vlajmaji/pull/3; new Linux acceptance is pending, not claimed here.

Original PR40163792526/push40163790036 atc49: Check/macOS passed, and Ubuntu actually passed the7pipeline tests, providing platform proof for repair01 before the separate later search-test E2BIG. Preserve that success/failure distinction. The search contract then failed5/1 at replace_canonical_draft sqlite3.arg(fullSQL), Os7 ArgumentListTooLong. Raw original-linux-failure.log SHAe19ff5a9e511487967d03cfa1997eae10b242e5f5d87a13044639724beb24758 remains exact. Earlier repair01 prompt/records/logs were not rewritten.

CDC read the complete frozen one-file diff: full existing canonical→mutation→bytes→digest→hexSQL retained; SQL goes to an exclusivelycreated ownedfixture file via write_all, writer is closed, new File starts at offset0 and becomes sqlite3stdin. Argv contains only the DBpath. No shell, payload argument, environment/argument-limit adjustment, reduced/padded fixture or pipe-write deadlock. Standard output() captures/reaps child and retains prior status/stderr assertion; existing root cleanup owns the SQLfile. The case name is used only for the owned filename and byte-count diagnostic.

The complete four-case duplicate test body is mechanically byte-identical after allowing only the added helper name argument; entry/capture/occurrence/ownedrevision mutations and Integrity expectations survive. All6searchtests remain, none skipped. Repositorywide crates/scripts/tests scan finds only the repaired dynamic SQLitehelper and the unchanged CLIverifytest's small fixedUPDATE. No second large SQLargv caller found in that scope.

Actual frozen independent execution: guard +1.98.0 locked/offline `cargo test -p vlajmaji-search --test dictionary_contract -- --nocapture`, PID25412, exit0/1.495s, all6passed/0failed/0ignored. The4large SQLstdin sizes match the CCfull6test run AND its one bounded repeat of the existing duplicate test (4mutations again). Full base fixture canonical JSON is132946B; the log's canonical_bytes field measures that BASE fixture, not each mutated serialization. Complete SQL payloads:

| Case | SQL stdin bytes |
| --- | ---: |
| entry | 266577 |
| capture | 294231 |
| occurrence | 270015 |
| owned revision | 266649 |

These complete inputs travel through a regular file/stdin, so SQLsize is absent from argv. Local macOS proof is not a new hosted Linux result; root's next actual LinuxCI determines that platform acceptance. No arbitrary extra fixture/new test campaign was introduced.

CC required final exact-source format→lint→check all passed, one serialized fixedtargetwriter, explicit targetENV/offline/Rust1.98 and assignedguard via retained PIDwrapper. Complete log independently queried over all25Rust result rows=109passed/0failed/1existingignored, tooling19passed; actual search6passed. Focused6plus one repeat1containing4mutations passed. All commands/PIDs/log/source/test hashes are retained in pr3-ci-repair02-cc.md and pr3-ci-repair02-evidence/. No local failed product/gate attempt in repair02 and no source edit followed final gates. The original remote failure remains distinct.

Brief retrospective: use subprocess argument lists for small control arguments, stdin/file transport for potentially large payloads. Full-fixture tests exposed the host argument-size boundary without reducing coverage; byte counts and repeated full mutations make the repair assessable. File-backed stdin kept lifecycle and error handling small. No framework/dependency/new contexts or broader campaign was needed.

Source/index/retained target returned idle after independent run; no further local product/gates needed. No fullDB/open/benchmark/feature/nativegraph/release/network/acquisition/tool-install/cleanup campaign, no installeddefault/sourcecontent move, no push/comment/merge by CDC/CC. Protected main/Bauske/earlier accepted records stay untouched, all14projectrowsOPEN/A03unopened. Local ready; root owns push and terminal hosted CI follow-up.
