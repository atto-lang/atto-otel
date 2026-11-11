# oreslang-otel

Native Oreslang telemetry: explicit trace/span correlation, structured logs,
monotonic duration measurement and metric observations. All telemetry policy,
span lifecycle, integer/JSON encoding and console export live in `src/otel.ores`.
The only runtime dependency is the standard library of `oreslang-source.java`.
No Java imports, OTel SDK, other Oreslang package, network exporter or submodule
is needed by this library. Rust/Maven/JDK are development tools only.

## Use

Add this repository as a pinned Git submodule:

```sh
git submodule add https://github.com/ores-truffle-oreslang/oreslang-otel.git dependencies/otel
git submodule update --init --recursive
```

```ores
import module otel from "../dependencies/otel/src/otel";

pub routine main(): void {
  val span = otel.start("my-service", otel.trace_id(), "", "work");
  span.event("INFO", "work started");
  // Perform work; use try/catch/finally to finish on every exit path.
  span.finish("ok", 0);
  otel.metric("my-service", span.trace_id(), "work.duration", "histogram", "ns", span.elapsed_ns());
  return;
}
```

Use `span.trace_id()` and `span.span_id()` to start children. Span completion is
idempotent; events after completion are ignored; elapsed time freezes at finish.
Keep each live span in its owning actor. Pass immutable IDs explicitly across
actor boundaries, then create a new local child span. There is no thread-local
ambient context, mutable global tracer or actor-shared span registry.
`start_at(service, trace, id, parent, name, started_ns)` supports reconstructing
context with an existing **same-process** monotonic start timestamp.

`log(service, trace, span, severity, body)` emits an unconditional correlated log.
For severity-aware logging, use `should_log(severity, minimum)` **before**
constructing a message. This is explicitly caller-configured, does not read
process environment on a hot path and never stores mutable logging policy
across actors. Supported severities are `TRACE`, `DEBUG`, `INFO`, `WARN`
(or `WARNING`), `ERROR` and `FATAL`; `ALL` and `OFF` are threshold-only.
Unknown severities or thresholds fail closed in `should_log`.

```ores
val minimum = "INFO"; // Read configuration once in the owning actor.
if otel.should_log("DEBUG", minimum); then
  otel.log("my-service", trace, span_id, "DEBUG", expensive_message());
fi
```

The explicit guard avoids evaluating `expensive_message()` when filtered.
Calling `log()` directly remains unconditional for backwards compatibility.
Filtering never implies redaction: callers must not pass secrets, credentials,
request bodies or sensitive attributes into records.

`metric(service, trace, name, kind, unit, value)` emits an integer observation:
use `counter` for nonnegative deltas, `gauge` for current values and `histogram`
for individual samples. Aggregation belongs to the consumer. The native
emitter now creates one immutable JSONL string and enqueues it with
`stdio.stdout.log_async(...)`, rather than blocking actor carriers on
stdout's BufferedWriter lock. The shared per-context OresVM async-stdio drain
uses bounded admission (1,024 records, 65,536 bytes each) and explicitly fails
on queue overflow instead of dropping telemetry silently. A successful enqueue
is not equivalent to physical delivery; graceful OresVM shutdown automatically
drains accepted records and reports sink failures. For an in-process checkpoint
call `await stdio.flush_async_logs()`.

The OTel module keeps no separate unbounded queue or background exporter.
The OresVM stdout queue and external telemetry collector have different delivery
guarantees. SIGKILL, a stuck sink beyond the configured deadline, collector
failures, and power loss remain outside the graceful flush guarantee.
Ordering is per Ores context; independent contexts or processes do not share
one total order.

Records use `schema: "oreslang-otel.v1"`, an epoch millisecond timestamp,
`service`, `ores_trace_id`, and `signal`. Span records add opaque `span_id`,
`parent_span_id`, `name`, `duration_ns`, `outcome`, and `http_status` (0 for no
HTTP status). JSON string controls are escaped and signed int64 values are
encoded exactly, without floating-point conversion. Consumers such as JavaScript
must preserve integer precision for values beyond their safe integer range.

## Runtime and validation

`SOURCE_REF` pins the required compiler/standard library. It supplies
`process.monotonic_ns()`, `process.unix_ms()`, `process.unique_id()`, Unicode
string escapes and (for HTTP consumers) `HttpExchange.completion/admitted_ns`.
Build that revision with JDK 25 and Maven:

```sh
mvn -q -f "$ORESLANG_SOURCE_DIR/pom.xml" -DskipTests package dependency:build-classpath -Dmdep.outputFile=target/classpath.txt
ORES_JAVA="$JAVA_HOME/bin/java" ./scripts/check.sh
```

Export `ORESLANG_SOURCE_DIR` to the compiler checkout. Tests execute actual
Oreslang and independently parse emitted records, checking correlation,
idempotence, every JSON control character, Unicode and int64 boundaries.
The demo server has real HTTP integration tests covering shared/isolated actors,
framework responses, request completion and handler failures.

## Relationship to ORES telemetry and OpenTelemetry

Reviewed [ores-otel-clients](https://github.com/ores-otel/ores-otel-clients/tree/f96b893280541914e904ccc8ea28be8422b8a44a)
and [ores-otel-interfaces](https://github.com/ores-otel/ores-otel-interfaces/tree/345bd97b8564ce97daa60110b41bfd66b68b6bc1).
This implementation follows their explicit correlation and `ores-trace-*` naming,
keeps application IDs distinct from native OTel IDs, and emits no request bodies,
cookies or arbitrary headers automatically.

This is an initial native telemetry library, not full OpenTelemetry or ORES
contract conformance. Its JSON Lines schema is library-local: it is **not OTLP**,
and UUID span IDs are not W3C span IDs. W3C traceparent/tracestate propagation,
OTLP transport, sampling, batching/retries, profiles, distributed context,
configuration loading and canonical ORES contract admission are not implemented.
The demo starts a local trace per admitted request and returns X-Ores-Trace-Id;
it does not trust incoming correlation headers or imply cross-service tracing.

## Exact integer encoding and local checks

Integer encoding stays in native Oreslang. Digit selection uses at most four exact comparisons/subtractions per nonzero decimal position, without floating-point division or a Java formatter. The required core runtime preserves signed int64 ordering beyond 2^53.

The Rust checker replaces tests/check.py and independently parses actual Oreslang output. It retains all smoke checks for correlation, escaping, idempotence and severity gating, then checks 215 decimal boundaries and deterministic random int64 samples against Rust integer formatting. Run `cargo test --locked --manifest-path tools/telemetry-check/Cargo.toml` for malformed-record rejection checks.


## Async completion retry safety and current compiler integration (2026-11-11)

The async OTel draft now **commits `Span.finish()` only after the bounded
`stdio.stdout.log_async` call admits its JSONL record**. Previously the
span was marked ended before enqueue, so an oversized record or a full queue
could permanently discard its completion and prevent retry. The new behavior
preserves a live, retryable span if **synchronous admission** fails.
An admitted record still requires a successful output drain/flush for
delivery, and is not crash durable.

New native guest `tests/span-finish-admission-retry.ores` constructs a
131,072-byte outcome deterministically. The bounded writer must reject it
(65,536-byte record limit); the guest catches the rejection, retries
`finish("ok", 200)` on the same span, requires that a second finish returns
false, and explicitly awaits `stdio.flush_async_logs()`. No fake telemetry
output is generated. `tools/telemetry-check` now validates exactly one
valid retry span record and rejects hostile field/count mutations; output-only
`--check-finish-retry-output` and `--check-smoke-output` modes cap input
and explicitly **do not attest compiler execution**.

`SOURCE_REF` was repinned to the actual Java async-stdio integration draft
[Java PR #2353](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2353)
at exact head `d5aaaaf61d4f668ddcb8b8a39ca9bc342da7a96d`.
That Java draft includes an exact-blob, private-same-org snapshot of the
updated async OTel source, both guest fixtures and the original Rust checker,
plus a **real JVM and Native Image guest workflow**
`.github/workflows/otel-async-native-guest.yml`.
The workflow requires the real Oreslang compiler and original Rust semantic
checks, not simulated output. The Native Image is a native AOT **host** with
an interpreted Oreslang guest, not standalone compiled Oreslang machine code.

**Current status is SOURCE/TESTS COMMITTED, GUEST NOT EXECUTED / NOT CERTIFIED.**
Local Linux Bash guardrail negative tests passed against exact Git blobs, but
the source organization's private GitHub Actions runners are still not
admitting Java jobs. No Maven/JUnit, native Oreslang JVM guest, Native Image
guest or Rust unit results have been obtained at the new PR head. The
external source-free GraalVM capacity probe is *not* authorized to receive
private source: mainline branch protection and independent CODEOWNER/security
review are still outstanding (Java issues #963 and #1501). Keep both drafts
unmerged and do not promote five-backend parity or OTLP readiness from the
source-only evidence.

## 2026-11-11 strict JSON oracle and bounded guest evidence

The original Rust checker now uses a recursive `serde` deserializer that
rejects duplicate JSON keys **before** converting to `serde_json::Value`.
This corrects the previous last-wins parser behavior, which could silently
allow duplicated `signal`, `body`, `ores_trace_id` or nested properties.
Unit fixtures cover escaped-equivalent key names and nested object/array
duplicates as well as valid signed-i64 JSON. `Cargo.toml` and `Cargo.lock`
now explicitly pin `serde` alongside `serde_json`.

The changed Rust source and both Cargo files were copied **byte-for-byte
within the same private source organization** to Java async-stdio PR #2353
and cross-checked using Git blob hashes:

- `tools/telemetry-check/src/main.rs`: `baee942bd7860cd327d34b6c4ab78c03f21cd0ba`
- `tools/telemetry-check/Cargo.toml`: `55fbd9508b735d0248c80339234cd82eb533275a`
- `tools/telemetry-check/Cargo.lock`: `cc5a82761a5188f1a2ed4f9ec67e566ddfed7f45`

The pinned Java guest harness removes all stale/partial success receipts on
an unsuccessful invocation; a disposable source-free Git-repo regression
confirmed the actual shell launcher fails closed and deletes both injected
stale receipts when a required source blob is absent. The exact updated Bash
files passed local Linux syntax and adverse subprocess checks. The **Rust
unit tests, Java JUnit tests, native Oreslang .ores programs and Native
Image interpreter host have NOT executed on these exact heads** because
private source-org GitHub Actions still fails before any runner/step admission.
Do not claim full OTel or cross-backend parity or merge these drafts.
