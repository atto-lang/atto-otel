# Native OpenTelemetry guest evidence — release gate v1

**Status (2026-11-05): NOT_EXECUTED.** This document is an acceptance
contract, not an execution receipt, OTLP implementation, or release sign-off.

The current `src/otel.ores` emits library-local JSON Lines through
`stdio.println`. Its spans are actor-local and its `ores_trace_id` and
`span_id` are **not** W3C Trace Context. A Rust parser passing host tests
does not demonstrate a compiler-produced native guest or a reliable,
backpressured OTLP exporter.

## Required evidence classes (never conflate)

| Class | Required action | May claim |
|---|---|---|
| SOURCE_ONLY | Audit `src/otel.ores` and frozen Git blob/tree | Authored source |
| HOST_CHECKER | Run `cargo test --locked --manifest-path tools/telemetry-check/Cargo.toml` | Rust parser/reference tests |
| GUEST_JAVA | Pin a clean `SOURCE_REF` Java/Graal compiler, compile and run `tests/smoke.ores`, capture exact exit/output | That guest/backend only |
| GUEST_NATIVE | Build a pinned LLVM, .NET NativeAOT, Wasm or Forge binary and execute *the same* source on its real target | Only the exact tested backend/target |
| RELEASE | Independent review, source/action provenance, bounded OTLP transport and runtime lifecycle tests | Only the proven feature set |

For **each** guest/backend run, store independently verifiable:

- exact repository commit SHA and Git tree; OTel source, fixture and manifest
  Git blob SHAs; compiler repository commit/tree, compiled compiler-binary SHA-256;
- backend/target ABI, runtime version, command/flags, worker/runner identity,
  workflow run ID, attempt ID and job ID; positive **nonempty executed steps**
  from the same attempt; zero-step jobs are `NOT_EXECUTED`;
- guest executable SHA-256, verified exit status, raw stdout/stderr SHA-256
  and byte counts; assertion count and negative-control outcomes;
- explicit unsupported behavior as **REJECTED**, not silently skipped or
  substituted with Rust/Java host fixtures.

## Testable correlation and lifecycle oracles

1. A parent and child span must preserve the exact trace ID and parent span
   identity; actor-to-actor transfer sends serializable IDs, never a mutable
   `Span` object or native pointer.
2. `finish` is single-use and freezes duration. Logging a span event after
   completion must have no effect.
3. Trace IDs, user-controlled labels, Unicode and control bytes remain
   strictly escaped. Reject duplicate protected JSON keys and unsafe signed
   integer rounding by collector/checker.
4. Lost collector capacity, partial writes, cancellation and actor termination
   must not retain unbounded records, block the carrier, or fabricate a
   delivered-span acknowledgement. This requires **new runtime integration**:
   the current direct `stdio.println` path is not an async, bounded OTLP queue.
5. A cross-backend trace comparison must normalize nondeterministic IDs/times
   *without* ignoring signal kind, parent-child structure, exact integer values,
   event ordering per actor, or completion cardinality.

## Transport acceptance blockers

- Replace or explicitly isolate the synchronous stdout exporter from any
  async-only default standard-library surface. A proposed `log_async` API
  must be tested for bounded admission, cancellation, flush barriers and
  actor shutdown without blocking a carrier thread.
- Implement an opt-in, bounded, versioned OTLP exporter with an explicit
  endpoint/trust policy, TLS/authentication, backpressure, retry budgets,
  serialization limits and stop/drain semantics. Do not reinterpret JSONL
  as OTLP or call local `ores_trace_id` a W3C trace ID.
- Demonstrate actual guest-originated network export, collector receipt
  verification, finite queue observability and a negative slow-collector case.

## Current reproducible checks

```sh
cargo test --locked --manifest-path tools/telemetry-check/Cargo.toml
cargo test --locked --manifest-path tools/compiler-admission/Cargo.toml
ORESLANG_SOURCE_DIR=/absolute/clean/pinned/compiler ./scripts/check.sh
```

Those commands **must be run** and their execution recorded. Merely listing
them here proves nothing. No source has been mirrored outside its private
organization, and this document creates no authorization to do so.

See linked work: Java async-stdio draft stack #537/#2349/#2352/#2353,
OTel async-export PR #5, and checker hardening PR #19. Do not merge a
second implementation over these lineages without semantic reconciliation.
