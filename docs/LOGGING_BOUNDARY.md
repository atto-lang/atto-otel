# Application logging boundary: oreslang-otel vs runtime logging

`oreslang-otel` is a **userland Oreslang module**. An application imports it
explicitly and decides which observations to create. The compiler, actors,
scheduler, GC, borrow checker and stdlib native implementations **must not**
import it, invoke its exporters, or route their core diagnostic events through it.

Core diagnostics are CLI/host-controlled and default off. In the Java/Graal
implementation, `--core-perf` and `--core-debug` record numeric data into
bounded primitive buffers for shutdown-time **stderr** export. These flags do
not set this package's log threshold and are not a distributed tracing system.
See the [cross-backend contract](https://github.com/ores-truffle-oreslang/oreslang-common-source/blob/audit/core-logging-separation-2026-10-16/docs/CORE_LOGGING_AND_OTEL.md).

## Correct userland calling pattern

```ores
import module otel from "../dependencies/otel/src/otel";

pub routine main(): void {
  val String minimum = "INFO"; // Configure once in the owning actor.
  val String trace = otel.trace_id();
  val span = otel.start("service", trace, "", "request");

  // IMPORTANT: check before computing any costly or sensitive message.
  if otel.should_log("DEBUG", minimum); then
    span.event("DEBUG", "detail");
  fi
  if otel.should_log("INFO", minimum); then
    span.event("INFO", "started");
  fi
  span.finish("ok", 0);
  return;
}
```

The guard controls **evaluation cost**, not just output. Neither
`otel.log(..., expensive_message())` nor `span.event(..., expensive_message())`
can avoid evaluating the message argument. `otel.log()` and `span.event()`
remain unconditional emitters for backwards compatibility.

This module currently writes one JSON record using `stdio.println` for each
observation. Encoding and console I/O are synchronous, can allocate, and can
block. **Never call it from core scheduler, actor ingress, GC, memory-management,
permission-checker, lock ownership or cancellation critical sections.** User
actors can choose to build their own bounded telemetry exporter actor and send
immutable observations to it, subject to backpressure/capacity policy.

Spans are actor-owned. Do not share mutable live Span instances across
supervisors/actors, use TLS as the context source, or assume monotonic timestamps
are portable between processes. Pass immutable correlation IDs and create a
local child span.

The current emitted schema `oreslang-otel.v1` is not OTLP, with no W3C
traceparent, remote exporter, batching, retry, sampling or automatic redaction.
Keep credentials, request bodies, secrets, and untrusted headers out of logs.
Any future OTLP exporter must be an explicit opt-in application dependency, with
bounded queues and shutdown/error policies. Export failure must not alter core
actor scheduling and cancellation semantics.

## Verification gates

- Ensure userland imports compile and emit parseable JSONL, including edge-case
  escaping and exact i64 values; run `./scripts/check.sh` with the pinned
  `SOURCE_REF` source compiler and `cargo test --locked --manifest-path tools/telemetry-check/Cargo.toml`.
- Ensure severities are checked **before** message construction; tests for
  `OFF`, `ALL`, equal-level admission and unknown severity/threshold remain.
- Confirm no import from `oreslang-otel` into core runtime repos.
- Independently benchmark the disabled guarded path and the synchronous exporter.
- Other backends are bootstrap subsets and do not yet claim userland package import
  parity. Unsupported OTel imports must fail closed.

## Verified cross-repository native guest execution (2026-10-16)

With the source Java logging PR at `6977b900e2601508ed2f1947ca5052b4060a68d9`,
a private test repository mirrored 96 exact Java production/stdlib/test blobs,
plus this repository's unchanged `src/otel.ores` (blob
`6a1b9b940f10e4b273ae2b02be5cfff5e0572afc`) and
`tests/smoke.ores` (blob
`6480413e7a6f8b6bef1a8a279f777f8293dc9937`).
JDK 25 compiled and executed that actual Oreslang guest, and an independent
Python JSON parser validated 9 records (6 logs, 2 spans, 1 metric), their
correlation, suppression behavior and exact signed 64-bit values.

Evidence: [private fallback Actions run 38921925326](https://github.com/ores-dummy-org-oreslang-stack/ores-dummy-org-oreslang-stack-web-server.ores/actions/runs/38921925326).
The Maven run passed 44 tests, including the guest smoke test; the JSONL
verification step passed. The Rust telemetry checker in the exact subtree
also passed 2 tests in [run 38915571390](https://github.com/ores-dummy-org-oreslang-stack/ores-dummy-org-oreslang-stack-web-server.ores/actions/runs/38915571390).

**Limits:** The compiler mirror is an exact selected-file subset, not the full
source Git tree. This is userland JSONL telemetry, not OTLP; no async exporter,
batching, W3C trace context, privacy scrubber, native-image, or five-backend
parity is implied. Do not route guest events through the core diagnostic
flight recorder.
