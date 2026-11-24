> **AI agent instructions (organization-wide):** Before working on code, issues, or pull requests, read and follow the [organization `AGENTS.md`](https://github.com/atto-lang/.github/blob/main/AGENTS.md), including its **merge-first stealth-development policy**. Honor any repository-local `AGENTS.md`, explicit holds, and GitHub-enforced rules.

# atto-otel

OpenTelemetry tracing, metrics, and logging integration for AttoLang.

## Naming migration and language authority

This repository is maintained at [atto-lang/atto-otel](https://github.com/atto-lang/atto-otel); use `https://github.com/atto-lang/atto-otel.git` for Git remotes and canonical `.atto` source examples. The normative language contract is [atto-spec](https://github.com/atto-lang/atto-spec), with executable guest evidence tracked by [atto-platform-conformance](https://github.com/atto-lang/atto-platform-conformance). Follow current ownership, borrowing, effects, actor, and syntax rules there; do not infer implemented features from a README or unexecuted CI.

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

AttoLang is the canonical project name; new source, examples and documentation use `.atto` and the `atto-lang` repository organization. This README defines scope, not feature readiness.

**Acceptance:** name exact source SHA/tree, runtime/toolchain, positive and negative checks and execution receipt before claiming support. Host-only tests and empty CI steps do not prove guest execution.

## W3C trace context v00 (host reference only)

A small, dependency-free `reference/trace_context.py` parses and formats 55-character W3C `traceparent` version-00 headers and exposes the sampled flag. Rejects all-zero trace/span IDs, uppercase/non-hex input, unsupported versions and invalid flag widths. Use `python3 -m unittest discover -s tests -v` to validate the reference behavior.

This does not implement Atto actor context propagation, OTLP export, metrics or logging, and it makes no guest-execution claim. Follow-up backend adapters must ensure tenant boundaries do not leak trace data.
