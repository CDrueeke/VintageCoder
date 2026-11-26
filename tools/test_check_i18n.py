# SPDX-License-Identifier: GPL-3.0-or-later
"""Tests für tools/check_i18n.py."""
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_i18n as c  # noqa: E402


class CheckI18nTests(unittest.TestCase):
    def test_repository_ist_sauber(self):
        self.assertEqual(c.pruefe(), [])

    def test_parameter_werden_je_schluessel_gelesen(self):
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, "a.ftl")
            open(p, "w", encoding="utf-8").write("# Kommentar { $nicht }\nk-eins = Hallo { $name }\nk-zwei = { $n ->\n    [one] 1\n   *[other] { $n } { $x }\n }\n# veraltet seit 0.4.0\nk-alt = x\n")
            r = c.lies_katalog(p)
            self.assertEqual(r["k-eins"], ({"name"}, False))
            self.assertEqual(r["k-zwei"], ({"n", "x"}, False))
            self.assertEqual(r["k-alt"], (set(), True))

    def test_argumente_der_aufrufe(self):
        t = 'x t!("k", a = f(1, 2), b = "x, y", c = v) + 1'
        m = c.CALL.search(t)
        teile = c.argumente(t, m.end())
        self.assertEqual([x.split("=")[0].strip() for x in teile[1:]], ["a", "b", "c"])

    def test_umlaut_in_englischen_katalogen_wird_erkannt(self):
        self.assertTrue(c.UMLAUT.search("Größe"))
        self.assertFalse(c.UMLAUT.search("Size"))

    def test_feste_deutsche_texte_werden_gemeldet_und_ausnahmen_gelten(self):
        with tempfile.TemporaryDirectory() as d:
            os.makedirs(os.path.join(d, "crates", "vc-gui", "src"))
            os.makedirs(os.path.join(d, "tools"))
            open(os.path.join(d, "crates", "vc-gui", "src", "a.rs"), "w", encoding="utf-8").write(
                'fn f() { let a = "Datei nicht gefunden"; let b = vc_i18n::t!("a-b-c"); let c = "ok"; let e = "Größe"; }\n#[cfg(test)]\nmod tests { fn t() { let x = "Fehler im Test"; } }\n')
            alt = (c.ROOT, c.ANZEIGE_CRATES, c.ANZEIGE_DATEIEN_IDE, c.AUSNAHMEN)
            try:
                c.ROOT, c.ANZEIGE_CRATES, c.ANZEIGE_DATEIEN_IDE = d, ["vc-gui"], []
                c.AUSNAHMEN = os.path.join(d, "tools", "i18n_ausnahmen.txt")
                f = c.feste_texte()
                self.assertEqual(len(f), 2, f)  # „Datei nicht gefunden“ und „Größe“; Schlüssel, „ok“ und der Test zählen nicht
                open(c.AUSNAHMEN, "w", encoding="utf-8").write("crates/vc-gui/src/a.rs\tDatei nicht\ncrates/vc-gui/src/a.rs\tGröße\n")
                self.assertEqual(c.feste_texte(), [])
            finally:
                c.ROOT, c.ANZEIGE_CRATES, c.ANZEIGE_DATEIEN_IDE, c.AUSNAHMEN = alt


if __name__ == "__main__":
    unittest.main()
