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
for individual samples. Aggregation belongs to the consumer. No buffering,
retry queue or unbounded attribute cache is allocated. Output uses one stdout
println per JSON record; stdout I/O and serialization add measured overhead.
A single call does not guarantee atomic records across separate Ores contexts,
processes or unsynchronized output sinks; collectors must delimit JSON Lines
and provide any required multi-writer serialization.

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

The development launcher validates that `ORESLANG_SOURCE_DIR` is a clean compiler checkout at `SOURCE_REF` before invoking Java. Build its classes and classpath from that revision first. The checkout check identifies source; it does not attest the ignored compiled classes.

The compiler-launch host admission regression suite is implemented in Rust (no Python):
`cargo test --locked --manifest-path tools/compiler-admission/Cargo.toml`.
It covers exact clean source revisions, invalid/floating pins, dirty and untracked
files, and rejection of a nested directory masquerading as a Git checkout.
These use isolated mock Java launchers; they do not certify guest compiler behavior.
