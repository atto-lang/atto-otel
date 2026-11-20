# PR #2 — telemetry JSON Lines template reconciliation

Old head `0dfac29fbc309f5ac82fbc10853dcd2416465e15`; canonical main `bb70c02bfbbd18f3f8e853aa1d59a6122861a5f2`.

Do not restore the historical `src/otel.ores` whole-file diff: current main has security-sensitive control escaping, bounded integer decimal encoding for full signed int64, explicit severity filtering and the current pinned `SOURCE_REF` (not the historical SHA). The old implementation's decrement-in-loop decimal conversion and backtick-only JSONL emission are not replacements for these safeguards.

The meaningful old assertions are direct log payload escaping, a negative full-width signed i64 metric, and explicit trace/span identifiers. Add an independent `tests/template-escaping-forward-port.ores` fixture containing those cases. Do not overwrite the larger current `tests/smoke.ores`, change its row counts, or reintroduce the historical `tests/check.py` (repo policy avoids new Python scripts). The fixture should be executed with the pinned compiler and its stdout parsed independently: two JSONL records, same trace ID, escaped original string round-trip, numeric `-9223372036854775808`, and no extra records; timestamps must remain numeric and positive.

The module must continue to use one `stdio.println` per complete JSONL record, with no claim of cross-process atomicity. **Gate:** exact pinned compiler source execution, parsed JSON verification and review. Git conflict cleanup is not test evidence.
