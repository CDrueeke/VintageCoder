#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Tests fuer tools/export_check.py (Verbotsscan und Ausschlussliste des Exports). Nur Standardbibliothek.

Aufruf (Arbeitsordner egal):
  python3 tools/test_export_check.py            alle Tests
  python3 tools/test_export_check.py -v         ausfuehrlich
  python3 tools/test_export_check.py -h         diese Hilfe

Beispiel:
  python3 tools/test_export_check.py ScanTests
Namen anderer Entwicklungsumgebungen stehen hier nur in Teilen (wie in release/registry.json)."""
import io, os, shutil, sys, tempfile, unittest
from contextlib import redirect_stdout

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import export_check as ec  # noqa: E402

WORD = "".join(["Inte", "lliJ"])
TECH = "vs" + "code"
REG = {"forbidden_ide_words_parts": [["Inte", "lliJ"], ["CL", "ion"]]}


class TreeCase(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="vc-export-test-")
        self.addCleanup(shutil.rmtree, self.root, True)

    def put(self, rel, content="", binary=False):
        p = os.path.join(self.root, rel)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, "wb" if binary else "w", **({} if binary else {"encoding": "utf-8"})) as f:
            f.write(content)
        return p

    def scan(self):
        return ec.scan_tree(self.root, REG)

    def rules(self, level="Fehler"):
        return [(f.path, f.line, f.rule) for f in self.scan() if f.level == level]


class ScanTests(TreeCase):
    def test_sauberer_baum_ohne_funde(self):
        self.put("README.md", "# Titel\nKontakt: vintagecoder@drueeke.de\nBeispiel 192.168.1.50 und 192.168.0.10\n")
        self.put("crates/a/src/lib.rs", "fn main() {}\n")
        self.assertEqual(self.scan(), [])

    def test_ide_wort_mit_datei_und_zeile(self):
        self.put("docs/a.md", "Zeile eins\nNutze %s dafuer\n" % WORD.lower())
        self.assertEqual(self.rules(), [("docs/a.md", 2, "ide-wort")])

    def test_ide_wort_im_dateinamen(self):
        self.put("docs/%s-notiz.md" % WORD, "x\n")
        self.assertIn(("docs/%s-notiz.md" % WORD, 0, "ide-wort"), self.rules())

    def test_wortlisten_dateien_sind_ausgenommen(self):
        self.put("tools/test_release.py", "x = '%s'\n" % WORD)
        self.assertEqual(self.rules(), [])

    def test_technischer_name_nur_in_editor_erweiterung(self):
        self.put("editors/ext/package.json", '{"engines": {"%s": "^1.80"}}\n' % TECH)
        self.assertEqual(self.rules(), [])
        self.put("docs/b.md", "Die Schnittstelle heisst %s.\n" % TECH)
        self.assertEqual(self.rules(), [("docs/b.md", 1, "ide-technisch")])

    def test_wortliste_gilt_auch_in_editor_erweiterung(self):
        self.put("editors/ext/extension.js", "// %s\n" % WORD)
        self.assertEqual(self.rules(), [("editors/ext/extension.js", 1, "ide-wort")])

    def test_feste_pfade(self):
        self.put("a.sh", "cd /Users/maxmuster/x\nls /Volumes/Daten\nC:\\Users\\hans\\Desktop\nC:\\\\Users\\\\hans\\\\x\n")
        self.assertEqual([r[1] for r in self.rules()], [1, 2, 3, 4])

    def test_platzhalter_und_beispielnamen_sind_kein_fund(self):
        self.put("a.md", "`/Users/<name>/x` und starts_with(\"/Volumes/\")\n/Users/anna/x C:\\Users\\Bob\\y C:/Users/a\n")
        self.assertEqual(self.rules(), [])

    def test_testwerte_in_testdateien_nur_hinweis(self):
        self.put("crates/x/tests/t.rs", 'let password = "geheim123"; // 10.0.0.9\n')
        self.assertEqual(self.rules(), [])
        self.assertEqual({r[2] for r in self.rules("Hinweis")}, {"geheimnis", "interne-ip"})

    def test_pruefzeile_und_dateiendung_im_namen(self):
        self.put("crates/x/src/a.rs", 'assert_eq!(redact("{\\"password\\":\\"geheim\\"}"), "");\n// C:\\Users\\x.img\n')
        self.assertEqual(self.rules(), [])

    def test_versionsnummer_ist_keine_mailadresse(self):
        self.put("a.md", "vice@3.9 und vice@0.1\n")
        self.assertEqual(self.rules(), [])

    def test_privater_name_nur_in_github_adressen_erlaubt(self):
        self.put("a.md", "https://github.com/CDrueeke/VintageCoder\nbuild von cdrueeke\n165030641+CDrueeke@users.noreply.github.com\n")
        self.assertEqual(self.rules(), [("a.md", 2, "privater-name")])

    def test_email_allowlist(self):
        self.put("a.md", "vintagecoder@drueeke.de\n1+x@users.noreply.github.com\nchef@firma.example\n")
        self.assertEqual(self.rules(), [("a.md", 3, "e-mail")])

    def test_interne_ips(self):
        self.put("a.md", "192.168.22.107\n10.0.0.5\n172.20.1.1\n172.32.1.1\n192.168.1.50\n192.168.1.64\nVersion 1.10.0.1\n")
        self.assertEqual([r[1] for r in self.rules()], [1, 2, 3])

    def test_geheimnismuster(self):
        self.put("a.rs", 'let password = "geheim123";\nlet key = API_KEY = \'abcdef\';\n// token: kein wert\n-----BEGIN RSA PRIVATE KEY-----\n')
        self.assertEqual([r[1] for r in self.rules()], [1, 2, 4])

    def test_co_autor(self):
        self.put("a.md", "Co-Authored-By: Jemand\n")
        self.assertEqual(self.rules(), [("a.md", 1, "co-autor")])

    def test_verbotene_ordner_und_dateien(self):
        self.put("assets/docs/x.md", "x\n")
        self.put(".claude/settings.json", "{}\n")
        self.put("examples/c64/hello/build/out.prg", "x\n")
        self.put("docs/konzept-layout.html", "x\n")
        self.put("docs/Mockup-1.svg", "x\n")
        self.put("sub/.DS_Store", "x\n")
        found = {r[0] for r in self.rules()}
        for p in ["assets/docs/x.md", ".claude/settings.json", "examples/c64/hello/build/out.prg", "docs/konzept-layout.html", "docs/Mockup-1.svg", "sub/.DS_Store"]:
            self.assertIn(p, found)

    def test_zu_grosse_datei(self):
        self.put("big.txt", "a" * (ec.MAX_SIZE + 1))
        self.assertEqual(self.rules(), [("big.txt", 0, "zu-gross")])

    def test_binaerdateien_nur_aus_allowlist(self):
        self.put("img/a.png", b"\x89PNG\0\0", binary=True)
        self.put("fonts/a.ttf", b"\0\1\0\0", binary=True)
        self.put("crates/vc-emu/tests/data/audio.bin", b"\0\0\0", binary=True)
        self.put("tools/x.exe", b"MZ\0\0", binary=True)
        self.assertEqual(self.rules(), [("tools/x.exe", 0, "binaer")])

    def test_hinweise_lassen_scan_bestehen(self):
        self.put("docs/a.md", "Mit Claude geschrieben\n")
        self.assertEqual(self.rules(), [])
        self.assertEqual(self.rules("Hinweis"), [("docs/a.md", 1, "werkzeug-name")])

    def test_eigene_dateien_nennen_muster_ohne_fund(self):
        self.put("tools/export_check.py", "x = '/Users/' # cdrueeke 192.168.22.1 Co-Authored\n")
        self.assertEqual(self.rules(), [])

    def test_git_ordner_wird_uebersprungen(self):
        self.put(".git/config", "/Users/anna\n")
        self.assertEqual(self.scan(), [])

    def test_ausgabe_und_rueckgabewert(self):
        self.put("registry-dummy", "")
        os.makedirs(os.path.join(self.root, "release"))
        with open(os.path.join(self.root, "release", "registry.json"), "w") as f:
            f.write('{"forbidden_ide_words_parts": [["Inte", "lliJ"]]}')
        self.put("a.md", "x %s\n" % WORD)
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = ec.main(["scan", self.root])
        self.assertEqual(rc, 1)
        self.assertIn("a.md:1: [ide-wort]", buf.getvalue())
        os.remove(os.path.join(self.root, "a.md"))
        with redirect_stdout(io.StringIO()):
            self.assertEqual(ec.main(["scan", self.root]), 0)


class ExcludeTests(TreeCase):
    def test_liste_lesen(self):
        lst = self.put("ex.txt", "# Kommentar\n\n.github/workflows/ci-local.yml  # nur privat\n docs/intern/*.md \n")
        self.assertEqual(ec.read_excludes(lst), [".github/workflows/ci-local.yml", "docs/intern/*.md"])

    def test_ausschluss_entfernt_dateien_und_leere_ordner(self):
        self.put(".github/workflows/ci-local.yml", "x\n")
        self.put(".github/workflows/ci.yml", "x\n")
        self.put("docs/intern/a.md", "x\n")
        self.put("docs/intern/b.md", "x\n")
        removed, unmatched = ec.apply_excludes(self.root, [".github/workflows/ci-local.yml", "docs/intern/*.md", "gibt/es/nicht"])
        self.assertEqual(removed, [".github/workflows/ci-local.yml", "docs/intern/a.md", "docs/intern/b.md"])
        self.assertEqual(unmatched, ["gibt/es/nicht"])
        self.assertTrue(os.path.exists(os.path.join(self.root, ".github/workflows/ci.yml")))
        self.assertFalse(os.path.exists(os.path.join(self.root, "docs/intern")))

    def test_echte_ausschlussliste_nennt_ci_local(self):
        pats = ec.read_excludes(os.path.join(HERE, "export-exclude.txt"))
        self.assertIn(".github/workflows/ci-local.yml", pats)

    def test_unterbefehl(self):
        self.put("x/y.txt", "x\n")
        lst = self.put("ex.txt", "x/y.txt\n")
        # Liste liegt im Baum: aus dem Baum nehmen, damit sie selbst nicht zaehlt
        with redirect_stdout(io.StringIO()):
            self.assertEqual(ec.main(["exclude", self.root, lst]), 0)
        self.assertFalse(os.path.exists(os.path.join(self.root, "x")))

    def test_ersetzen(self):
        self.put("CHANGELOG.md", "lang\n")
        self.put("release/CHANGELOG.public.md", "kurz\n")
        pairs = ec.read_replacements(self.put("rp.txt", "# Kommentar\nrelease/CHANGELOG.public.md CHANGELOG.md\n"))
        self.assertEqual(pairs, [("release/CHANGELOG.public.md", "CHANGELOG.md")])
        done, missing = ec.apply_replacements(self.root, pairs)
        self.assertEqual((len(done), missing), (1, []))
        with open(os.path.join(self.root, "CHANGELOG.md"), encoding="utf-8") as f:
            self.assertEqual(f.read(), "kurz\n")
        self.assertFalse(os.path.exists(os.path.join(self.root, "release")))
        # fehlende Quelle: Fehler, nichts veraendert
        done, missing = ec.apply_replacements(self.root, [("gibt-es-nicht.md", "CHANGELOG.md")])
        self.assertEqual((done, missing), ([], ["gibt-es-nicht.md"]))
        with self.assertRaises(ValueError):
            ec.read_replacements(self.put("bad.txt", "nur-eine-spalte\n"))


class RealisticTests(unittest.TestCase):
    def test_echte_registry_hat_woerter_in_teilen(self):
        import release
        reg = release.load_json(os.path.join(HERE, "..", "release", "registry.json"))
        self.assertGreater(len(release.ide_words(reg)), 5)
        with open(os.path.join(HERE, "..", "release", "registry.json"), encoding="utf-8") as f:
            self.assertNotIn(WORD, f.read())


if __name__ == "__main__":
    unittest.main()
