#!/usr/bin/env python3
"""Execute native code and validate the exported wire records independently."""
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[1]
result = subprocess.run([str(root / 'scripts/compiler.sh'), str(root / 'tests/smoke.ores')],
                        check=True, text=True, capture_output=True)
rows = [json.loads(line) for line in result.stdout.splitlines()]
logs = [row for row in rows if row['signal'] == 'log']
spans = [row for row in rows if row['signal'] == 'span']
metrics = [row for row in rows if row['signal'] == 'metric']
assert len(spans) == 2, spans  # repeated finish must not export twice
assert spans[0]['name'] == 'child'
assert spans[0]['parent_span_id'] == spans[1]['span_id']
assert all(row['ores_trace_id'] == 'ores-trace-test' for row in rows)
assert all(row['timestamp_unix_ms'] > 1700000000000 for row in rows)
assert all(row['duration_ns'] >= 0 for row in spans)
assert logs[0]['body'] == 'quote" slash\\ newline\n nul\0 tab\t emoji😀'
assert logs[1]['body'] == ''.join(map(chr, range(32)))
assert logs[2]['body'] == '-9223372036854775808|9223372036854775807|0|-1|100'
assert [row['body'] for row in logs[3:]] == [
    'visible warning', 'visible all', 'visible equal'
]
assert all(row['schema'] == 'oreslang-otel.v1' for row in rows)
assert len(logs) == 6  # after-finish and severity-filtered events are ignored
assert len(metrics) == 1 and metrics[0]['value'] == 1
print('PASS: native spans, correlation, idempotence, JSON escaping, int64 and severity gating')
