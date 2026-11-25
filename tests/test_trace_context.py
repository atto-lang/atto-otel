import unittest
from reference.trace_context import TraceContextError, TraceParent, parse_traceparent


class TraceContextTest(unittest.TestCase):
    def test_round_trip_and_sampling(self):
        parent = TraceParent("a" * 32, "b" * 16, 0x81)
        self.assertEqual(parse_traceparent(parent.format()), parent)
        self.assertTrue(parent.sampled)
        self.assertFalse(TraceParent("a" * 32, "b" * 16, 0x02).sampled)

    def test_invalid_versions_and_ids(self):
        good = TraceParent("a" * 32, "b" * 16).format()
        for bad in [good.replace("00-", "ff-", 1),
                    good.replace("a" * 32, "0" * 32),
                    good.replace("b" * 16, "0" * 16),
                    good.upper(), good + "!", good[:-1], good + ",extra", None]:
            with self.subTest(bad=bad), self.assertRaises(TraceContextError):
                parse_traceparent(bad)

    def test_flag_validation(self):
        for flags in [-1, 256, True, "01"]:
            with self.assertRaises(TraceContextError):
                TraceParent("a" * 32, "b" * 16, flags)


if __name__ == "__main__":
    unittest.main()
