# SPDX-License-Identifier: GPL-3.0-or-later
"""Tests für tools/check_spdx.py."""
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_spdx  # noqa: E402


class SpdxTests(unittest.TestCase):
    def test_repository_ist_sauber(self):
        self.assertEqual(check_spdx.pruefe(), [])

    def test_rmt_player_ist_gpl_nur(self):
        for p in ("examples/atari/music_demo/lib/rmtplayr.asm", "examples/atari/jumprun/lib/rmt.inc"):
            self.assertEqual(check_spdx.kennung(p), "GPL-3.0-only", p)

    def test_ordnervermerk_gefunden(self):
        self.assertTrue(check_spdx.ordnervermerk("examples/c64/prallball/prallball.asm"))
        self.assertTrue(check_spdx.ordnervermerk("data/sys/c64/c64.inc"))

    def test_unbekannte_kennung_wird_gemeldet(self):
        self.assertIsNone(check_spdx.ERLAUBT.match("Proprietary"))
        self.assertTrue(check_spdx.ERLAUBT.match("GPL-3.0-or-later"))
        self.assertTrue(check_spdx.ERLAUBT.match("MIT-0"))


if __name__ == "__main__":
    unittest.main()
