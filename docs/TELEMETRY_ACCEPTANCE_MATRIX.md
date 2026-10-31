# Exact-head OTel / JSONL conformance matrix (2026-10-31)

**Status: specification and acceptance plan only.** This file is not a claim that tests or compiler-backed fixtures have run. Canonical library at main commit `a22a616567a851f159bbd284830fbc3bb6ac6ddf`; candidate validator in draft PR #13. The stable wire format here is **`oreslang-otel.v1` JSON Lines**, not OTLP, W3C TraceContext, or remote collector delivery.

## Evidence classes (must remain separate)

- **R — admission:** authorized runner assigned, job has nonempty recorded steps and exact head/attempt.
- **H — host validator:** Rust assertions for JSON structure and malformed/extra fields, with actual test exit receipt.
- **G — guest:** pinned clean compiler SHA **and Git tree**, compiler/toolchain/build hashes, actual `.ores` compile/execute and exact stdout/stderr/exit expectations.
- **S — scheduler/stdio:** actual guest actor/sink, blocked-drain timing, capacity and cancellation/retirement, with host task ledger.
- **B — backend:** separate Java/GraalVM JIT and native AOT/hybrid, LLVM native, .NET NativeAOT and Wasmtime evidence. No substitute passes across backends.

## Smallest acceptance table

| ID | Class | Input / schedule | Expected oracle |
| --- | --- | --- | --- |
| ENC-01 | H+G | Quotes, backslashes, NUL, U+0001–U+001F, tab/newline, astral Unicode in one emitted log | One parseable JSONL record per event; exact decoded body; no raw newline splitting |
| ENC-02 | H+G | Signed int64 MIN, MAX, -1, 0, 1; neighboring base-10 boundaries; `2^53+1` | Exact JSON integer values and exact `integer(...)` text, no float conversion |
| ENC-03 | H | Unexpected key `credentials`, missing required key, nested `body`, wrong timestamp type | Reject; no positive conformance receipt |
| ENC-04 | H | Duplicate raw JSON member name (including escaped equivalent) | **PATCHED, NOT EXECUTED**: `UniqueRecord` rejects duplicate top-level keys before `Value` conversion. Escaped-key alias and trailing-object tests authored; requires stepful `cargo test --locked` receipt |
| MET-01 | H+G | Valid counter delta, gauge negative value, histogram sample | Schema, kind, unit and signed integer fields agree; collector aggregation is not implied |
| MET-02 | H; proposed G | Negative counter, unknown kind, missing metric value | Host validator must reject; native `metric` admission behavior remains to be specified and proved |
| SPAN-01 | H+G | Root and child, explicit immutable parent/trace IDs | Nonempty unique span IDs, correct linkage, same trace, root parent empty |
| SPAN-02 | H+G | Finish span twice with different status, then emit event | First finish true, second false; one span record; post-finish event absent; duration frozen |
| SPAN-03 | G | `start_at` with same-process monotonic start and intentionally future timestamp | Duration nonnegative; explicit timestamp provenance; no cross-process monotonic assumption |
| SPAN-04 | G+S | Event/finish race, cancellation during suspension, retire actor generation and deliver late callback | No write after finish/retirement; no cross-generation ownership leak; **unverified** |
| EXP-01 | S | Block stdout writer while actor emits accepted async records | Actor carrier remains eligible; enqueue bounded; no writer-lock wait on carrier |
| EXP-02 | S | Block writer; fill 1,024-record queue, then one extra record | Prompt deterministic overflow error, no silent drop; drain originals after unblocking |
| EXP-03 | S | Block sink after one record; repeatedly call `flush_async_logs` | Bounded flush barrier futures; fail fast after bound; **open core issue #2348** |
| EXP-04 | S | Writer throws I/O error; await record and barrier | Both report terminal failure and context shutdown reports failure |
| EXP-05 | S | Graceful context close with accepted records; forced stop / SIGKILL | Accepted records flushed within deadline or explicit failure; no durability guarantee on forced stop |
| SEC-01 | G+B | Guest without STDOUT capability calls log or flush | Deny; no capability elevation or fallback host stdout |
| SEC-02 | G+S | Just-under / exactly / over 65,536 UTF-8-byte async record | Defined admission boundary; no unbounded in-queue record; serialization allocation before admission measured separately |
| PAR-01 | B | Run supported ENC/MET/SPAN fixtures on each actual backend | Report individual backend and execution mode; unsupported cases remain `NOT_EXECUTED` |

## Run recipe and release restrictions

Host checks (not guest proof):
```sh
cargo test --locked --manifest-path tools/compiler-admission/Cargo.toml
cargo test --locked --manifest-path tools/telemetry-check/Cargo.toml
```
Oreslang `tests/smoke.ores` and generated native int64 parity (requires a clean, built, exact `SOURCE_REF` compiler checkout):
```sh
ORES_JAVA="$JAVA_HOME/bin/java" ./scripts/check.sh
```
For every completed case attach repository/PR/head SHA, Git tree SHA, compiler/runtime commit and tree, target backend, workflow run **and attempt**, job and its nonempty step ledger, exact stdout/stderr or a digest plus preserved artifact, exit status and expectation. Negative cases must fail for their intended reason; runner admission and compiler parse errors are not substitute oracles.

Do not promote PR #13 or async PR #5 on a zero-step failure. PR #5 still depends on draft Java/Graal stdio PR #537. Current source-free runner-admission canary run `41166352071` failed with `steps=null`, which is infrastructure evidence only. `main` is not branch protected. No private compiler source is authorized for cross-organization transfer by this matrix.

## Source-review note: Java/GraalVM main reconciliation (2026-10-31)

Stacked Java PR #2349 at head `e28a7beba666b41e261426d3827a45d1ae46307f` is based on Java #537's old head `78aa824c75c0cddfb17dc1627158d1828e1bfac4`. Its ancestor is **3,783 commits behind Java main** on the 2026-10-31 compare result. Current main does **not** contain `AsyncStdioLogs.java` or `AsyncStdioLogsTest.java`, and its `OresContext.close()` has physical actor/native-I/O join and quarantine fences not present in #537.

Do **not** copy the older `OresContext.close()` implementation or force-update main/parent. Reapply only the bounded async stdio addition after reconciling the current actor and native-I/O shutdown lifecycle, and retain quarantine on failed physical retirement. Re-run old and new regression suites against the *new* exact compiler head before pinning `SOURCE_REF` for OTel PR #5.
