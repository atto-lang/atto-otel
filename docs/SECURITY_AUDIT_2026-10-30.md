# Oreslang OTel security and correctness audit — 2026-10-30

## Scope and provenance

Audited `oreslang-otel` main at commit `a22a616567a851f159bbd284830fbc3bb6ac6ddf`
(tree `706da6c699a1127274e54411b6e34f46a6e636ca`), its native
`src/otel.ores`, smoke program, Rust telemetry checker, compiler admission
launcher, GitHub Actions workflows, and open async-export PR #5.
This is the only OTel-named/telemetry-named repository found in
`ores-truffle-oreslang` by the connected repository search. The separate
`ores-otel` organization and external collectors are **out of scope**.

## Verified findings

| Priority | Finding | Evidence / acceptance |
| --- | --- | --- |
| P0 governance | `main` branch is **not protected** (`protected: false`; required checks disabled). | Enable branch rules through authorized repository administration, require independent review and exact-head checks before merge. |
| P0 test evidence | Latest inspected host-contract run `40795597295` reports failure with **no job steps**; PR #5 run `38422222086` likewise has null steps. | Neither is a Rust test result, native guest result, nor evidence of a source failure. Require a nonempty step ledger and success at the exact head. |
| P1 export liveness | Current `src/otel.ores` uses synchronous `stdio.println` for all three emitters. Actor carrier stalls on stdout/slow collector are not ruled out. | Async PR #5 depends on unmerged draft Java/GraalVM PR #537; keep both draft pending actual slow-sink, flush, queue overflow and AOT/native tests. |
| P1 lifecycle and delivery | `Span.finish` marks `ended` before attempting output. Delivery failures or process exit can leave an internally finished span without a delivered record. | Define enqueue-vs-delivery contract; test failure reporting, cancellation, actor-generation retirement, double finish and flush. Do not promise crash durability. |
| P1 validator coverage | The main-branch smoke verifier checked record counts, selected payloads and correlation but did not enforce field names, mandatory field presence, field types, allowed metric kind, or span HTTP status on every row. | This PR adds exact per-signal record-field/type validation and adversarial host tests; then strengthens fixed smoke correlation, severity, outcomes and metric profile assertions. |
| P2 contract interoperability | `oreslang-otel.v1` JSONL is **not** OTLP; `ores-trace-*` IDs are not W3C TraceContext. No cross-process propagation or external exporter here. | Version any OTLP/W3C work separately; reject untrusted inbound trace IDs and bound baggage cardinality before admitting propagation. |

## Changes in this hardening PR

* Require exactly the declared common and per-signal fields (no missing/unknown keys) in the Rust smoke oracle.
* Require string/int64 value types, nonempty service/trace/required IDs, monotonic nonnegative duration and bounded HTTP status values.
* Reject negative counters and unrecognized metric kinds **in the validator** (does not change the native emitter's API behavior).
* Assert exact smoke severity sequence, distinct root/child span IDs, parent linkage, completion outcomes/HTTP statuses, and metric name/kind/unit/value.
* Add host-only adversarial test cases for missing fields, leaked extra fields, stringified timestamps, nested bodies, negative durations, invalid statuses and metric violations.
* Leave `SOURCE_REF`, `src/otel.ores`, on-wire schema, runtime behavior and private-source export permissions unchanged.

The checker parses JSON through `serde_json::Value`. JSON objects with duplicate raw member names are normalized by that parser; duplicate-key rejection is **not** proven by this patch. A separate streaming key-uniqueness decoder is required if rejecting duplicates at the raw-bytes boundary becomes an acceptance criterion.

## Evidence boundaries and next release gate

The repository's CI workflow is intended to run:
```bash
cargo test --locked --manifest-path tools/compiler-admission/Cargo.toml
cargo test --locked --manifest-path tools/telemetry-check/Cargo.toml
```
These are **host** tests; their success does not prove guest execution.
The actual Oreslang guest + exact signed-int64 test requires a clean pinned
`oreslang-source.java` checkout with JDK 25/Maven classes and:
```bash
ORES_JAVA="$JAVA_HOME/bin/java" ./scripts/check.sh
```
No native, Java/GraalVM, LLVM, .NET, Wasmtime, slow-stdout, AOT or OTLP
conformance result is claimed from this audit. No private source was copied
to another organization.

Keep async PR #5 draft until the final Java async-stdio implementation is merged
and the **same exact pinned compiler head** produces stepful tests for
bounded queue admission, overflow, sink failure, graceful close,
`flush_async_logs` barrier ordering, actor liveness under slow stdout,
JSON escape control characters, int64 MIN/MAX and full smoke output.
