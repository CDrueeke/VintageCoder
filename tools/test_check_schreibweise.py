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

    def test_vc20_deutsch_vic20_englisch(self):
        self.put("crates/vc-i18n/catalog/en/x.ftl", "a = VC-20 cartridge\nb = VIC-20 cartridge\n")
        self.put("crates/vc-i18n/catalog/de/x.ftl", "a = VC-20 Modul\nb = VIC-20 Modul\n")
        self.put("crates/vc-ide/help/en/p.md", "VIC-20 ist richtig, `VC-20` im Code nicht.\n")
        self.put("crates/vc-ide/help/q.md", "# VIC-20\n")
        funde = sorted((f[0], f[1]) for f in cs.scan(self.root))
        self.assertEqual(funde, [("crates/vc-i18n/catalog/de/x.ftl", 2), ("crates/vc-i18n/catalog/en/x.ftl", 1), ("crates/vc-ide/help/q.md", 1)])

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

    def test_sprachabhaengige_regeln_gelten_nur_fuer_den_katalog_ihrer_sprache(self):
        zp, onion = "zero" + "page", "onion" + " skin"
        self.put("crates/vc-i18n/catalog/en/x.ftl", "a = %s und %s\n" % (zp, onion))        # englisch: beides verboten
        self.put("crates/vc-i18n/catalog/en/y.ftl", "a = zero page, Onion Skin, zero-page addressing\n")    # richtig
        self.put("crates/vc-i18n/catalog/de/z.ftl", "a = Zeropage und Onion Skin\n")                         # richtig
        self.put("crates/vc-i18n/catalog/de/w.ftl", "a = zero page\n")                                      # deutsch: englischer Ersatz verboten
        self.put("docs/frei.md", "%s\n" % zp)                                                            # andere Dateien: keine Sprachregel
        funde = sorted({(f, z) for f, z, _ in cs.scan(self.root)})
        self.assertEqual(funde, [("crates/vc-i18n/catalog/de/w.ftl", 1), ("crates/vc-i18n/catalog/en/x.ftl", 1)])

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
