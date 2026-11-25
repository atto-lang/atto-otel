"""Host-only W3C traceparent version-00 reference (no exporter/network)."""
from dataclasses import dataclass
import re

_PATTERN = re.compile("[0-9a-f]+", re.ASCII)


class TraceContextError(ValueError):
    pass


@dataclass(frozen=True)
class TraceParent:
    trace_id: str
    parent_id: str
    trace_flags: int = 0

    def __post_init__(self):
        if not _hex(self.trace_id, 32) or int(self.trace_id, 16) == 0:
            raise TraceContextError("trace_id must be nonzero 32-character lowercase hex")
        if not _hex(self.parent_id, 16) or int(self.parent_id, 16) == 0:
            raise TraceContextError("parent_id must be nonzero 16-character lowercase hex")
        if type(self.trace_flags) is not int or not 0 <= self.trace_flags <= 255:
            raise TraceContextError("trace_flags must be one byte")

    @property
    def sampled(self):
        return bool(self.trace_flags & 1)

    def format(self):
        return f"00-{self.trace_id}-{self.parent_id}-{self.trace_flags:02x}"


def _hex(value, length):
    return type(value) is str and len(value) == length and _PATTERN.fullmatch(value) is not None


def parse_traceparent(value):
    """Accept only v00 in its precise, lowercase, 55-character wire form."""
    if type(value) is not str or len(value) != 55:
        raise TraceContextError("traceparent must be exactly 55 ASCII characters")
    parts = value.split("-")
    if len(parts) != 4 or parts[0] != "00" or not _hex(parts[3], 2):
        raise TraceContextError("unsupported traceparent version or flags")
    return TraceParent(parts[1], parts[2], int(parts[3], 16))
