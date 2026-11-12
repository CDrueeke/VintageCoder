#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Tests fuer tools/release.py (Parser, Abhaengigkeiten, Staging, Verbotsscan). Nur Standardbibliothek.

Aufruf (Arbeitsordner egal):
  python3 tools/test_release.py            alle Tests
  python3 tools/test_release.py -v         ausfuehrlich
  python3 tools/test_release.py -h         diese Hilfe

Beispiel:
  python3 tools/test_release.py ParserTests
"""
import hashlib, io, json, os, shutil, subprocess, sys, tempfile, unittest
from contextlib import redirect_stdout

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import release  # noqa: E402

TESTDATA = os.path.join(HERE, "testdata")


def rows_for(ids, reg, stand=None):
    """Auswahlzeilen fuer alle Registry-Kennungen; nur ids sind angehakt."""
    st = stand or {}
    return [{"id": k, "label": k, "stand": st.get(k, "Stabil"), "checked": k in ids, "section": "", "line": i} for i, k in enumerate(reg["features"], 1)]


class ParserTests(unittest.TestCase):
    def test_haekchen_schreibweisen(self):
        text = "\n".join(["| Kennung | A | Stand | Release |", "|---|---|---|---|"] +
                          ["| `STU-T%d` | x | Beta | %s |" % (i, m) for i, m in enumerate(["[x]", "[X]", "[ X]", "[X ]", "[ x]", "[x ]", "[ ]", "[]", ""])])
        rows, problems = release.parse_features(text)
        self.assertEqual([r["checked"] for r in rows], [True] * 6 + [False] * 3)
        self.assertEqual(problems, [])

    def test_unlesbares_haekchen(self):
        rows, problems = release.parse_features("| `STU-A` | x | Beta | ja |\n")
        self.assertEqual(rows, [])
        self.assertEqual(len(problems), 1)

    def test_kommentare_und_fremde_tabellen_werden_ignoriert(self):
        text = "<!--\n| `STU-MUSTER` | x | Beta | [x] |\n-->\n| Name | Wert |\n|---|---|\n| `foo` | [x] |\n| STU-OHNE | x | Beta | [x] |\n| `STU-ECHT` | Text | Beta | [x] |\n"
        rows, _ = release.parse_features(text)
        self.assertEqual([r["id"] for r in rows], ["STU-ECHT"])

    def test_abschnitt_und_stand(self):
        rows, _ = release.parse_features("## 2. Studios\n\n| Kennung | S | Stand | Release |\n|---|---|---|---|\n| `STU-X` | a | Beta | [x] |\n")
        self.assertEqual((rows[0]["section"], rows[0]["stand"], rows[0]["label"]), ("2. Studios", "Beta", "a"))

    def test_status_parser(self):
        st = release.parse_status("| Kennung | Bestandteil | Stand | Hinweise |\n|---|---|---|---|\n| `BAS-B2A` | x | In Arbeit | h |\n\n| Kennung | Hinweise |\n|---|---|\n| `SYS-C64` | text |\n")
        self.assertEqual(st, {"BAS-B2A": "In Arbeit"})

    def test_echte_features_datei(self):
        with open(os.path.join(release.ROOT, "FEATURES.md"), encoding="utf-8") as f:
            rows, problems = release.parse_features(f.read())
        self.assertEqual(problems, [])
        self.assertEqual(len(rows), 31)
        self.assertEqual(len({r["id"] for r in rows}), 31)


class GlobTests(unittest.TestCase):
    def test_brace_und_stern(self):
        g = release.Globs(["data/chips/{vic2,sid}.chip.json", "data/sys/c64/**", "*/jumprun"])
        self.assertTrue(g.match("data/chips/sid.chip.json"))
        self.assertFalse(g.match("data/chips/ted.chip.json"))
        self.assertTrue(g.match("data/sys/c64/b2a/rt_zp.asm"))
        self.assertTrue(g.match("c64/jumprun"))
        self.assertFalse(g.match("c64/x/jumprun"))
        self.assertEqual(g.unmatched(), [])

    def test_unmatched(self):
        g = release.Globs(["a/*.x", "b/**"])
        g.match("a/1.x")
        self.assertEqual(g.unmatched(), ["b/**"])


class RegistryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.reg = release.load_registry()

    def test_check_auf_dem_repository_ist_gruen(self):
        res = release.Result()
        with open(os.path.join(release.ROOT, "FEATURES.md"), encoding="utf-8") as f:
            rows, problems = release.parse_features(f.read())
        release.check_ids(rows, self.reg, res)
        release.check_filesystem(self.reg, res)
        self.assertEqual(res.errors + problems, [])

    def test_kennung_fehlt_in_registry_und_umgekehrt(self):
        rows = rows_for(set(), self.reg)
        res = release.Result()
        release.check_ids(rows[:-1] + [{"id": "STU-NEU", "label": "", "stand": "Beta", "checked": False, "section": "", "line": 99}], self.reg, res)
        self.assertTrue(any("STU-NEU fehlt in release/registry.json" in e for e in res.errors))
        self.assertTrue(any("steht nicht in FEATURES.md" in e for e in res.errors))

    def test_doppelte_kennung(self):
        rows = rows_for(set(), self.reg)
        res = release.Result()
        release.check_ids(rows + [dict(rows[0], line=99)], self.reg, res)
        self.assertTrue(any("doppelt" in e for e in res.errors))

    def test_jedes_beispiel_hat_genau_ein_paket(self):
        for proj in release.example_projects(self.reg):
            pkg, hits = release.project_package(self.reg, proj)
            self.assertEqual(len(hits), 1, proj)

    def test_vcs2600_nie(self):
        eff = set(self.reg["features"])
        shipped, _ = release.data_files_to_ship(eff, self.reg)
        self.assertFalse([p for p in shipped if "vcs2600" in p])


class DependencyTests(unittest.TestCase):
    """Jede Zeile der Abhaengigkeitstabelle (Konzept 4.2) einzeln: Verletzung = Fehler, Erfuellung = gruen."""

    @classmethod
    def setUpClass(cls):
        cls.reg = release.load_registry()
        cls.feats = cls.reg["features"]

    def run_sel(self, ids, **kw):
        eff, skipped, res = release.resolve(rows_for(set(ids), self.reg), self.reg, **kw)
        return eff, skipped, res

    def test_harte_abhaengigkeiten_je_zeile(self):
        for fid, f in self.feats.items():
            for need in f.get("needs", []):
                if need == "IDE-APP":
                    continue                       # Studios ohne IDE werden uebersprungen, kein Fehler (eigener Test)
                with self.subTest(fid=fid, need=need):
                    _, _, res = self.run_sel([fid])
                    self.assertTrue(any(fid in e and need in e for e in res.errors), res.errors)
                    others = [n for n in f["needs"] if n != need]
                    _, _, ok = self.run_sel([fid] + f["needs"])
                    self.assertFalse([e for e in ok.errors if "braucht" in e], ok.errors)

    def test_mindestens_eines_je_zeile(self):
        for fid, f in self.feats.items():
            anyn = f.get("needs_any")
            if not anyn:
                continue
            with self.subTest(fid=fid):
                base = ["IDE-APP"] if "IDE-APP" in f.get("needs", []) else []
                _, _, bad = self.run_sel([fid] + f.get("needs", []) + base)
                self.assertTrue(any("mindestens eines" in e for e in bad.errors), bad.errors)
                for a in anyn:
                    _, _, good = self.run_sel([fid, a] + f.get("needs", []) + base)
                    self.assertFalse([e for e in good.errors if "mindestens eines" in e], (a, good.errors))

    def test_studio_ohne_ide_wird_uebersprungen(self):
        eff, skipped, res = self.run_sel(["STU-SPRITE", "SYS-C64"])
        self.assertIn("STU-SPRITE", skipped)
        self.assertNotIn("STU-SPRITE", eff)
        self.assertEqual(res.errors, [])
        self.assertTrue(any("uebersprungen" in n for n in res.notes))

    def test_hinweis_nennt_was_anzukreuzen_ist(self):
        _, _, res = self.run_sel(["HW-MEGA65"], allow_unfinished=True)
        self.assertTrue(any("HW-MEGA65 braucht SYS-MEGA65: bitte SYS-MEGA65" in e for e in res.errors), res.errors)

    def test_auto_deps(self):
        eff, _, res = self.run_sel(["EMU-XEMU"], auto_deps=True)
        self.assertIn("SYS-MEGA65", eff)
        self.assertTrue(any("--auto-deps" in n for n in res.notes))
        self.assertEqual(res.errors, [])

    def test_stand_unter_beta(self):
        reg = dict(self.reg, preview_ok=[])
        rows = rows_for({"SYS-PET"}, reg, {"SYS-PET": "Spike"})
        _, _, res = release.resolve(rows, reg)
        self.assertTrue(any("SYS-PET" in e and "Spike" in e for e in res.errors))
        _, _, res = release.resolve(rows, reg, allow_unfinished=True)
        self.assertEqual(res.errors, [])
        self.assertTrue(any("allow-unfinished" in w for w in res.warnings))

    def test_vorschau_freigegeben(self):
        # preview_ok: bewusst freigegebene Zeilen unter Beta werden als Vorschau ausgeliefert, ohne --allow-unfinished
        reg = dict(self.reg, preview_ok=["SYS-PET"])
        rows = rows_for({"SYS-PET"}, reg, {"SYS-PET": "Spike"})
        _, _, res = release.resolve(rows, reg)
        self.assertEqual(res.errors, [])
        self.assertTrue(any("Vorschau" in w for w in res.warnings))

    def test_beispiel_nur_mit_target(self):
        eff, _, _ = self.run_sel(["EX-HELLO", "SYS-C64"])
        ex = release.examples_to_ship(eff, self.reg)
        self.assertTrue(ex)
        self.assertTrue(all(p.startswith("c64/") for p, _ in ex), ex)
        eff, _, _ = self.run_sel(["EX-HELLO"])
        self.assertEqual(release.examples_to_ship(eff, self.reg), [])

    def test_system_ohne_anbindung_warnt(self):
        _, _, res = self.run_sel(["SYS-C64"])
        self.assertTrue(any("keine Anbindung" in w for w in res.warnings))
        _, _, res = self.run_sel(["SYS-C64", "EMU-VICE"])
        self.assertFalse(any("keine Anbindung" in w for w in res.warnings))


class StageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.mkdtemp(prefix="test-release-")
        cls.reg = release.load_registry()

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmp, ignore_errors=True)

    def stage(self, name, os_name="macos", extra=(), out=None):
        out = out or os.path.join(self.tmp, "%s-%s" % (name, os_name))
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = release.main(["stage", "--os", os_name, "--out", out, "--no-bin", "--force", "--features", os.path.join(TESTDATA, name + ".md")] + list(extra))
        return rc, out, buf.getvalue()

    def files(self, out):
        return {os.path.relpath(os.path.join(d, f), out).replace(os.sep, "/") for d, _, fs in os.walk(out) for f in fs}

    def test_nur_kommandozeile_nur_c64(self):
        rc, out, log = self.stage("nur-cli-c64")
        self.assertEqual(rc, 0, log)
        files = self.files(out)
        self.assertFalse([f for f in files if "vintagecoder" in f.lower().replace("data/", "") and f.startswith(("bin/", "vintagecoder"))])
        self.assertEqual(sorted(f for f in files if f.startswith("data/targets/")), ["data/targets/c64.target.json"])
        self.assertEqual({f.split("/")[1] for f in files if f.startswith("examples/")}, {"c64"})
        self.assertFalse([f for f in files if f.startswith(("licenses/", "macos/Info", "data/sys/atari8bit", "data/sys/mega65", "data/sys/vcs2600"))])
        rj = json.load(open(os.path.join(out, "data", "release.json"), encoding="utf-8"))
        self.assertEqual((rj["ide"], rj["studios"], rj["links"], rj["targets"], rj["examples"]), (False, [], [], ["c64"], ["EX-HELLO"]))
        self.assertIn("data/cli/build.json", files)
        self.assertIn("release-report.md", files)

    def test_programme_vcc_und_lsp_immer_ide_nur_mit_ide(self):
        bindir = os.path.join(self.tmp, "fakebin")
        os.makedirs(bindir, exist_ok=True)
        for n in ("vcc", "vc-lsp", "vintagecoder"):
            with open(os.path.join(bindir, n), "w") as f:
                f.write("x")
        for name, ide in (("nur-cli-c64", False), ("nur-ide-c64-ohne-emulator", True)):
            out = os.path.join(self.tmp, "bin-" + name)
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = release.main(["stage", "--os", "macos", "--out", out, "--bin-dir", bindir, "--force", "--features", os.path.join(TESTDATA, name + ".md")])
            self.assertEqual(rc, 0, buf.getvalue())
            self.assertEqual(sorted(os.listdir(os.path.join(out, "bin"))), ["vc-lsp", "vcc"] + (["vintagecoder"] if ide else []))

    def test_fehlendes_programm_ist_fehler(self):
        out = os.path.join(self.tmp, "kein-bin")
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = release.main(["stage", "--os", "linux", "--out", out, "--bin-dir", os.path.join(self.tmp, "gibtsnicht"), "--features", os.path.join(TESTDATA, "nur-cli-c64.md")])
        self.assertEqual(rc, 1)
        self.assertIn("nicht gefunden", buf.getvalue())

    def test_atari_ohne_andere_systeme(self):
        rc, out, log = self.stage("nur-cli-atari")
        self.assertEqual(rc, 0, log)
        files = self.files(out)
        self.assertEqual(sorted(os.path.basename(f) for f in files if f.startswith("data/targets/")), ["atari8.target.json"])
        self.assertTrue(any(f.startswith("data/sys/atari8bit/") for f in files))
        self.assertFalse([f for f in files if f.startswith(("data/sys/c64", "data/sys/mega65", "data/sys/plus4"))])
        self.assertIn("examples/common/jumprun/core.asm", files)
        self.assertTrue(all(f.split("/")[1] in ("atari", "common") for f in files if f.startswith("examples/")))
        self.assertIn("data/args/atari800.json", files)
        self.assertNotIn("data/args/altirra.json", files)

    def test_voll_und_betriebssysteme(self):
        for os_name, marker in (("windows", "installer.inc"), ("macos", "macos/Kommandozeile installieren.command"), ("linux", "usr/share/man/man1/vcc.1")):
            rc, out, log = self.stage("voll", os_name)
            self.assertEqual(rc, 0, log)
            self.assertIn(marker, self.files(out), os_name)
        inc = open(os.path.join(self.tmp, "voll-windows", "installer.inc"), encoding="utf-8").read()
        self.assertIn("#define HasIde", inc)
        self.assertIn("#define HasExamples", inc)
        rc, out, _ = self.stage("nur-cli-c64", "windows")
        inc = open(os.path.join(out, "installer.inc"), encoding="utf-8").read()
        self.assertNotIn("HasIde", inc)
        self.assertIn("HasExamples", inc)

    def test_reproduzierbar(self):
        def digest(out):
            h = hashlib.sha256()
            for f in sorted(self.files(out)):
                h.update(f.encode()); h.update(open(os.path.join(out, f), "rb").read())
            return h.hexdigest()
        _, a, _ = self.stage("nur-cli-atari", extra=["--reproducible"], out=os.path.join(self.tmp, "r1"))
        _, b, _ = self.stage("nur-cli-atari", extra=["--reproducible"], out=os.path.join(self.tmp, "r2"))
        self.assertEqual(digest(a), digest(b))

    def test_linux_archiv(self):
        rc, out, log = self.stage("nur-cli-c64", "linux", extra=["--archive", "--reproducible"], out=os.path.join(self.tmp, "lin"))
        self.assertEqual(rc, 0, log)
        self.assertTrue(os.path.exists(out + ".tar.gz"))
        import tarfile
        with tarfile.open(out + ".tar.gz") as tf:
            names = tf.getnames()
        self.assertTrue(any(n.endswith("/data/release.json") for n in names))
        self.assertFalse(any(n.endswith("vintagecoder.desktop") for n in names))

    def test_nicht_leeres_ziel_ohne_force(self):
        out = os.path.join(self.tmp, "voll-vorhanden")
        os.makedirs(out)
        open(os.path.join(out, "x"), "w").close()
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = release.main(["stage", "--os", "macos", "--out", out, "--no-bin", "--features", os.path.join(TESTDATA, "nur-cli-c64.md")])
        self.assertEqual(rc, 1)

    def test_abhaengigkeit_bricht_stage_ab(self):
        bad = os.path.join(self.tmp, "bad.md")
        with open(bad, "w", encoding="utf-8") as f, open(os.path.join(TESTDATA, "nur-cli-c64.md"), encoding="utf-8") as src:
            f.write(src.read().replace("| `EMU-XEMU` | Xemu (MEGA65) | Beta | [ ]", "| `EMU-XEMU` | Xemu (MEGA65) | Beta | [x]"))
        out = os.path.join(self.tmp, "bad-out")
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = release.main(["stage", "--os", "macos", "--out", out, "--no-bin", "--features", bad])
        self.assertEqual(rc, 1)
        self.assertIn("EMU-XEMU braucht SYS-MEGA65", buf.getvalue())
        self.assertFalse(os.path.exists(out))


class ForbiddenScanTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="test-release-scan-")
        self.reg = release.load_registry()

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def put(self, rel, data):
        p = os.path.join(self.tmp, rel)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, "wb") as f:
            f.write(data)

    def test_emulator_programmdatei(self):
        self.put("bin/x64sc", b"x")
        self.put("Altirra64.exe", b"x")
        errs, _ = release.forbidden_scan(self.tmp, self.reg)
        self.assertEqual(len(errs), 2, errs)

    def test_rom_treffer_ueber_pruefsumme(self):
        roms = release.load_json(os.path.join(release.ROOT, "data", "roms", "known.json"))["roms"]
        # eine Datei gleicher Groesse, aber anderer Inhalt: kein Treffer
        r = roms[0]
        self.put("data/x.bin", b"\x00" * int(r["size"]))
        errs, _ = release.forbidden_scan(self.tmp, self.reg)
        self.assertEqual(errs, [])

    def test_namen_anderer_ides(self):
        w = release.ide_words(self.reg)
        self.put("data/help/a.md", ("Oeffne das Projekt in %s.\n" % w[1]).encode())
        self.put("examples/c64/x/readme.md", (w[0] + "\n").encode())
        self.put("README.md", ("%s Erweiterung\n" % w[9]).encode())
        errs, warns = release.forbidden_scan(self.tmp, self.reg)
        self.assertEqual(len(errs), 2, errs)          # data/ und examples/ sind Fehler
        self.assertEqual(len(warns), 1, warns)        # feste Dateien nur Warnung

    def test_technischer_name_nur_in_der_editor_erweiterung(self):
        tech = release.technical_ide_token()
        w = release.ide_words(self.reg)
        # erlaubt: Kleinschreibung im Quelltext der Erweiterung (Staging-Pfad je System verschieden)
        self.put("editors/ext/extension.js", ('require("%s")\n' % tech).encode())
        self.put("usr/share/doc/vintagecoder/editors/ext/package.json", ('{"engines": {"%s": "^1"}}\n' % tech).encode())
        errs, warns = release.forbidden_scan(self.tmp, self.reg)
        self.assertEqual((errs, warns), ([], []))
        # verboten: überall sonst, und die Schreibweisen der Wortliste auch in der Erweiterung
        self.put("docs/a.md", ("%s\n" % tech).encode())
        self.put("editors/ext/README.md", ("%s\n" % w[9]).encode())
        errs, warns = release.forbidden_scan(self.tmp, self.reg)
        self.assertEqual(len(warns), 2, warns)

    def test_wortliste_steht_nicht_im_klartext(self):
        raw = open(os.path.join(release.ROOT, "release", "registry.json"), encoding="utf-8").read()
        for w in release.ide_words(self.reg):
            self.assertNotIn(w, raw)

    def test_staging_von_data_und_examples_ist_frei_von_ide_namen(self):
        eff = set(self.reg["features"])
        data, _ = release.data_files_to_ship(eff, self.reg)
        for p in data:
            if os.path.splitext(p)[1] in release.TEXT_EXT:
                text = open(os.path.join(release.ROOT, p), encoding="utf-8", errors="replace").read()
                for w in release.ide_words(self.reg):
                    self.assertNotIn(w, text, p)


class CliTests(unittest.TestCase):
    def test_hilfe_aller_unterbefehle(self):
        for args in ([], ["check"], ["plan"], ["stage"], ["bootstrap"]):
            r = subprocess.run([sys.executable, os.path.join(HERE, "release.py")] + args + ["-h"], capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, args)
            self.assertIn("usage", r.stdout)
        top = subprocess.run([sys.executable, os.path.join(HERE, "release.py"), "-h"], capture_output=True, text=True).stdout
        self.assertIn("Beispiele:", top)

    def test_check_und_plan_laufen(self):
        for cmd in ("check", "plan"):
            r = subprocess.run([sys.executable, os.path.join(HERE, "release.py"), cmd], capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    def test_explain(self):
        r = subprocess.run([sys.executable, os.path.join(HERE, "release.py"), "check", "--explain", "SYS-MEGA65"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0)
        self.assertIn("EMU-XEMU", r.stdout)


if __name__ == "__main__":
    if any(a in ("-h", "--help") for a in sys.argv[1:]):
        print(__doc__)
        sys.exit(0)
    unittest.main()
