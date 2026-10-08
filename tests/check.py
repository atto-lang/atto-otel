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
assert len(logs) == 4  # event after finish is ignored; direct logger added
assert logs[3]['service'] == 'ser\\vice\nquote"'
assert logs[3]['body'] == 'direct \\ log'
assert logs[3]['span_id'] == 'span-direct'
assert spans[0]['outcome'] == 'o"k'
assert len(metrics) == 2
assert metrics[0]['value'] == 1
assert metrics[0]['name'] == 'req\\uests\nquote"'
assert metrics[1]['name'] == 'int64-min'
assert type(metrics[1]['value']) is int and metrics[1]['value'] == -(2**63)
assert all(row['schema'] == 'oreslang-otel.v1' for row in rows)
assert all(type(row['timestamp_unix_ms']) is int for row in rows)
assert all('ores_trace_id' in row and 'signal' in row for row in rows)
print('PASS: native spans, parent correlation, idempotence, template JSON escaping and int64 encoding')
