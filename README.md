# atto-otel

OpenTelemetry tracing, metrics, and logging integration for AttoLang.

## Purpose

Instrumentation interfaces, context propagation, export compatibility, and actor-safe telemetry.

## API and contract expectations

- Expose general-purpose interfaces that are usable without an Atto commercial account.
- Document data/ownership semantics, error behavior, and supported runtimes.
- Keep protocol and serialization details portable across relevant AttoLang backends.

## Development and testing

Use checked-in implementation and CI for actual commands. Add behavior-focused tests, including negative cases and lifecycle edge cases; distinguish local unit tests from executed multi-backend conformance.

## Project status

This describes intended scope, not a verified feature matrix. Consult the source tree and exact-commit CI results for implementation status.

## Ecosystem

- [AttoLang](https://github.com/atto-lang)
- [Language specification](https://github.com/atto-lang/atto-spec)
- [Cross-platform conformance](https://github.com/atto-lang/atto-platform-conformance)

## Repository ownership and contribution routing

**Belongs here:** OpenTelemetry trace, metric and log APIs, actor/task context propagation and exporter adapters.

**Belongs elsewhere:** Instrumentation core and semantic conventions live here; deployment collector configuration belongs with apps/atto-co; compiler tracing intrinsics belong in backend repos.

**Security and portability boundary:** Prevent cross-tenant trace leakage and secret attributes, bound queues and backpressure, and preserve cancellation and actor ownership.

AttoLang is the public name; preserve existing Oreslang/.ores compatibility and use canonical .atto for new portable library source. This README defines scope, not feature readiness.

**Acceptance:** name exact source SHA/tree, runtime/toolchain, positive and negative checks and execution receipt before claiming support. Host-only tests and empty CI steps do not prove guest execution.
