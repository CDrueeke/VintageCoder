#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Tests fuer tools/check_schreibweise.py. Nur Standardbibliothek.

Aufruf:
  python3 -m unittest tools.test_check_schreibweise
  python3 tools/test_check_schreibweise.py -v"""
import os, shutil, subprocess, sys, tempfile, unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import check_schreibweise as cs  # noqa: E402

WORT = "".join(["Null", "seite"])
WORT2 = "".join(["Kil", "ler"])


class ScanTests(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="vc-schreibweise-")
        self.addCleanup(shutil.rmtree, self.root, True)

    def put(self, rel, inhalt, binaer=False):
        p = os.path.join(self.root, rel)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, "wb" if binaer else "w", **({} if binaer else {"encoding": "utf-8"})) as f:
            f.write(inhalt)

    def test_treffer_in_text_gross_klein_und_zusammensetzung(self):
        self.put("a.md", "Alles gut.\n%s retten\n" % WORT)
        self.put("src/b.rs", "// %s\n" % WORT.upper())
        self.put("c.asm", "; %sn-Zelle\n" % WORT.lower())
        funde = cs.scan(self.root)
        self.assertEqual([(f[0], f[1]) for f in funde], [("a.md", 2), ("c.asm", 1), ("src/b.rs", 1)])

    def test_zweites_verbotenes_wort_in_text_und_dateinamen_ohne_beachtung_der_gross_klein_schreibung(self):
        self.put("a.md", "Ein %sfeature.\nsauber\n" % WORT2)
        self.put("b.rs", "// %s\n" % WORT2.upper())
        self.put("doc/%s-liste.md" % WORT2.lower(), "sauber\n")
        funde = cs.scan(self.root)
        self.assertEqual([(f[0], f[1]) for f in funde], [("a.md", 1), ("b.rs", 1), ("doc/%s-liste.md" % WORT2.lower(), 0)])

    def test_treffer_im_dateinamen(self):
        self.put("doc/%s.md" % WORT, "sauber\n")
        self.assertEqual(cs.scan(self.root), [("doc/%s.md" % WORT, 0, "doc/%s.md" % WORT)])

    def test_binaerdateien_und_sauberer_text_ohne_fund(self):
        self.put("bild.png", WORT.encode() + b"\0", binaer=True)
        self.put("daten.dat", b"\0" + WORT.encode(), binaer=True)
        self.put("ok.md", "Zeropage-Prüfung, Zeropage retten.\n")
        self.assertEqual(cs.scan(self.root), [])

    def test_die_datei_des_scans_selbst_ist_ausgenommen(self):
        self.put("tools/check_schreibweise.py", "# %s\n" % WORT)
        self.assertEqual(cs.scan(self.root), [])

    def test_repository_ist_sauber(self):
        wurzel = os.path.dirname(HERE)
        if not os.path.exists(os.path.join(wurzel, ".git")):
            self.skipTest("kein Git-Arbeitsordner")
        self.assertEqual(cs.scan(wurzel, git=True), [], "versionierte Dateien enthalten die verbotene Schreibweise")

    def test_aufruf_liefert_rueckgabewert_1_bei_funden(self):
        self.put("x.txt", WORT + "\n")
        r = subprocess.run([sys.executable, os.path.join(HERE, "check_schreibweise.py"), self.root], capture_output=True, text=True)
        self.assertEqual(r.returncode, 1); self.assertIn("x.txt:1", r.stdout)


if __name__ == "__main__":
    unittest.main()
