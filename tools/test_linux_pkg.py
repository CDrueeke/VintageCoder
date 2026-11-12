#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Tests fuer tools/linux_pkg.py und `release.py package` (Metadaten, Dateilisten, Abhaengigkeiten, Versionsabbildung). Ohne Docker.

dpkg-deb und rpmbuild werden nur benutzt, wenn sie auf diesem Rechner vorhanden sind (sonst werden diese Tests uebersprungen); die
verbindlichen Laeufe machen die Bau-Container (tools/docker/). Nur Standardbibliothek.

Aufruf (Arbeitsordner egal):
  python3 tools/test_linux_pkg.py          alle Tests
  python3 tools/test_linux_pkg.py -v       ausfuehrlich
Beispiel:
  python3 tools/test_linux_pkg.py VersionTests
"""
import io, json, os, shutil, sys, tempfile, unittest
from contextlib import redirect_stdout

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import linux_pkg as lp  # noqa: E402
import release  # noqa: E402

TESTDATA = os.path.join(HERE, "testdata")


class VersionTests(unittest.TestCase):
    def test_deb(self):
        for v, want in (("0.1.0", "0.1.0"), ("0.1.0-alpha", "0.1.0~alpha"), ("0.2.0-beta.1", "0.2.0~beta.1"), ("1.0.0", "1.0.0"),
                        ("1.2.3-rc-1", "1.2.3~rc.1"), ("1.0.0+b5", "1.0.0+b5")):
            self.assertEqual(lp.deb_version(v), want, v)

    def test_rpm(self):
        for v, want in (("0.1.0", ("0.1.0", "1")), ("0.1.0-alpha", ("0.1.0", "0.1.alpha")), ("0.2.0-beta.1", ("0.2.0", "0.1.beta.1")),
                        ("1.0.0", ("1.0.0", "1")), ("1.0.0-rc-1", ("1.0.0", "0.1.rc.1"))):
            self.assertEqual(lp.rpm_version_release(v), want, v)

    def test_sortierung_vorab_vor_stabil(self):
        # Debian: '~' sortiert vor allem, auch vor dem Ende; RPM: Release 0.1.x < 1 (numerischer Vergleich der ersten Teile)
        self.assertEqual(lp.deb_version("0.1.0-alpha").split("~")[0], lp.deb_version("0.1.0"))      # dpkg sortiert 0.1.0~alpha vor 0.1.0
        self.assertEqual(lp.rpm_version_release("0.1.0-alpha")[1].split(".")[0], "0")
        self.assertEqual(lp.rpm_version_release("0.1.0")[1], "1")

    def test_tar(self):
        self.assertEqual(lp.tar_version("0.1.0-alpha"), "0.1.0-alpha")
        self.assertEqual(lp.tar_version("0.1.0"), "0.1.0")

    def test_keine_semver(self):
        for v in ("0.1", "v0.1.0", "0.1.0-", "x"):
            with self.assertRaises(lp.PackageError):
                lp.deb_version(v)

    def test_architekturen(self):
        self.assertEqual((lp.deb_arch("x86_64"), lp.deb_arch("aarch64")), ("amd64", "arm64"))
        self.assertEqual((lp.rpm_arch("amd64"), lp.rpm_arch("arm64"), lp.rpm_arch("x86_64")), ("x86_64", "aarch64", "x86_64"))

    def test_version_aus_dem_workspace(self):
        with open(os.path.join(release.ROOT, "Cargo.toml"), encoding="utf-8") as f:
            self.assertIn('version = "%s"' % release.cargo_version(), f.read())


class DistroTests(unittest.TestCase):
    def osr(self, **kv):
        return "\n".join('%s="%s"' % (k, v) for k, v in kv.items())

    def test_erkennung(self):
        for kv, want in (({"ID": "debian", "VERSION_ID": "13"}, "debian"), ({"ID": "ubuntu", "ID_LIKE": "debian"}, "ubuntu"),
                         ({"ID": "fedora"}, "fedora"), ({"ID": "opensuse-tumbleweed", "ID_LIKE": "opensuse suse"}, "opensuse"),
                         ({"ID": "opensuse-leap"}, "opensuse"), ({"ID": "linuxmint", "ID_LIKE": "ubuntu debian"}, "ubuntu"),
                         ({"ID": "rocky", "ID_LIKE": "rhel centos fedora"}, "fedora"), ({"ID": "arch"}, None)):
            self.assertEqual(lp.detect_distro(self.osr(**kv)), want, kv)

    def test_konfiguration(self):
        cfg = lp.load_config()
        self.assertEqual(sorted(cfg["distros"]), sorted(lp.DISTROS))
        for d, c in cfg["distros"].items():
            self.assertIn(c["format"], ("deb", "rpm"))
            self.assertTrue(c["ide"]["depends"], d)
            self.assertTrue(c["image"], d)
            for link, spec in c["links"].items():
                self.assertIn(spec["level"], ("recommends", "suggests"), (d, link))
        # t64-Namen nur bei Debian und Ubuntu; rpm-Distributionen kennen keine Alternativen mit '|'
        for d in ("debian", "ubuntu"):
            self.assertIn("libasound2t64 | libasound2", cfg["distros"][d]["ide"]["depends"])
        for d in ("fedora", "opensuse"):
            self.assertFalse([x for x in cfg["distros"][d]["ide"]["depends"] + cfg["distros"][d]["ide"]["recommends"] if "|" in x], d)
        self.assertEqual(cfg["distros"]["fedora"]["links"]["vice"]["level"], "suggests")       # VICE nur ueber RPM Fusion


def make_fake_bins(tmp):
    d = os.path.join(tmp, "fakebin")
    os.makedirs(d, exist_ok=True)
    for n in ("vcc", "vc-lsp", "vintagecoder"):
        with open(os.path.join(d, n), "w") as f:
            f.write("#!/bin/sh\necho %s\n" % n)
        os.chmod(os.path.join(d, n), 0o755)
    return d


class StagedTests(unittest.TestCase):
    """Arbeitet auf echten Linux-Stagings der Testauswahlen (mit Platzhalter-Programmen)."""

    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.mkdtemp(prefix="test-linux-pkg-")
        cls.bins = make_fake_bins(cls.tmp)
        cls.cfg = lp.load_config()
        cls.stages = {}

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmp, ignore_errors=True)

    @classmethod
    def stage(cls, name):
        if name not in cls.stages:
            out = os.path.join(cls.tmp, "st-" + name)
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = release.main(["stage", "--os", "linux", "--out", out, "--bin-dir", cls.bins, "--force", "--reproducible", "--features", os.path.join(TESTDATA, name + ".md")])
            assert rc == 0, buf.getvalue()
            cls.stages[name] = out
        return cls.stages[name]

    # --- Staging-Baum

    def test_fhs_baum_ohne_ide(self):
        st = self.stage("nur-cli-c64")
        files = lp.stage_files(st)
        self.assertIn("usr/bin/vcc", files)
        self.assertIn("usr/bin/vc-lsp", files)
        self.assertIn("usr/share/vintagecoder/data/release.json", files)
        self.assertIn("usr/share/man/man1/vcc.1", files)
        self.assertIn("usr/share/man/man1/vc-lsp.1", files)
        self.assertFalse([f for f in files if is_ide(f)], "IDE-Dateien ohne IDE-APP")
        self.assertFalse([f for f in files if not f.startswith("usr/")])
        self.assertTrue(all(f.startswith("usr/share/vintagecoder/examples/c64/") or f.startswith("usr/share/vintagecoder/examples/common/") for f in files if "/examples/" in f))
        self.assertEqual(lp.read_selection(st)[0], False)

    def test_fhs_baum_mit_ide(self):
        st = self.stage("nur-ide-c64-ohne-emulator")
        files = lp.stage_files(st)
        for f in ("usr/bin/vintagecoder", "usr/share/applications/vintagecoder.desktop", "usr/share/icons/hicolor/512x512/apps/vintagecoder.png",
                  "usr/share/man/man1/vintagecoder.1", "usr/share/doc/vintagecoder/licenses/Inter-OFL.txt"):
            self.assertIn(f, files)
        self.assertNotIn("usr/bin/data", files)
        with open(os.path.join(st, "usr/share/applications/vintagecoder.desktop"), encoding="utf-8") as fh:
            self.assertIn("Exec=vintagecoder %f", fh.read())

    def test_komponenten(self):
        ohne = lp.split_components(lp.stage_files(self.stage("nur-cli-c64")), False)
        self.assertEqual(list(ohne), ["cli"])
        mit = lp.split_components(lp.stage_files(self.stage("voll")), True)
        self.assertEqual(sorted(mit), ["cli", "ide"])
        self.assertEqual(set(mit["cli"]) & set(mit["ide"]), set())
        self.assertIn("usr/bin/vcc", mit["cli"])
        self.assertIn("usr/bin/vc-lsp", mit["cli"])
        self.assertIn("usr/bin/vintagecoder", mit["ide"])
        self.assertNotIn("usr/bin/vintagecoder", mit["cli"])
        self.assertEqual(sorted(mit["cli"] + mit["ide"]), lp.stage_files(self.stage("voll")))

    def test_ide_dateien_ohne_ide_auswahl_sind_fehler(self):
        files = lp.stage_files(self.stage("voll"))
        with self.assertRaises(lp.PackageError):
            lp.split_components(files, False)
        with self.assertRaises(lp.PackageError):
            lp.split_components([f for f in files if not is_ide(f)], True)

    def test_fremdes_im_staging(self):
        st = os.path.join(self.tmp, "fremd")
        shutil.copytree(self.stage("nur-cli-c64"), st)
        open(os.path.join(st, "vintagecoder.exe"), "w").close()
        with self.assertRaises(lp.PackageError):
            lp.stage_files(st)

    def test_manpages_werden_gz(self):
        self.assertEqual(lp.installed_path("usr/share/man/man1/vcc.1"), "/usr/share/man/man1/vcc.1.gz")
        self.assertEqual(lp.installed_path("usr/bin/vcc"), "/usr/bin/vcc")
        self.assertEqual(lp.installed_path("usr/share/doc/vintagecoder/README.md"), "/usr/share/doc/vintagecoder/README.md")

    def test_verbotene_namen(self):
        self.assertEqual(lp.forbidden_in(["/usr/bin/vcc", "/usr/lib/x64sc", "/opt/atari800"]), ["/usr/lib/x64sc", "/opt/atari800"])
        files = lp.installed_paths(lp.stage_files(self.stage("voll")))
        self.assertEqual(lp.forbidden_in(files), [])
        self.assertFalse([f for f in files if f.lower().endswith((".rom", ".crt"))])

    # --- Abhaengigkeiten

    def rel(self, distro, which, links, fmt=None, version="0.1.0", libc=None):
        fmt = fmt or self.cfg["distros"][distro]["format"]
        return lp.dependencies(self.cfg, distro, which, links, version, fmt, libc)

    def test_cli_ohne_gui_abhaengigkeiten(self):
        for d in lp.DISTROS:
            r = self.rel(d, "cli", [])
            text = " ".join(r["depends"] + r["recommends"] + r["suggests"])
            for gui in ("libxkbcommon", "libgl", "mesa", "wayland", "asound", "alsa", "xcursor"):
                self.assertNotIn(gui, text.lower(), d)
        self.assertEqual(self.rel("debian", "cli", [])["depends"], ["libc6"])
        self.assertEqual(self.rel("debian", "cli", [], libc="2.41")["depends"], ["libc6 (>= 2.41)"])
        self.assertEqual(self.rel("fedora", "cli", [])["depends"], [])             # rpm findet die Bibliotheken selbst

    def test_anbindungen_nur_mit_auswahl(self):
        self.assertEqual(self.rel("debian", "cli", ["vice", "atari800"])["recommends"], ["vice", "atari800"])
        self.assertEqual(self.rel("debian", "cli", ["atari800"])["recommends"], ["atari800"])
        self.assertEqual(self.rel("debian", "cli", ["altirra", "xemu", "ultimate"])["recommends"], [])      # keine Distributionspakete
        self.assertEqual(self.rel("debian", "cli", [])["recommends"], [])
        f = self.rel("fedora", "cli", ["vice", "atari800"])
        self.assertEqual((f["recommends"], f["suggests"]), (["atari800"], ["vice"]))
        o = self.rel("opensuse", "cli", ["vice", "atari800"])
        self.assertEqual((o["recommends"], o["suggests"]), ([], ["vice", "atari800"]))

    def test_ide_haengt_vom_cli_gleicher_version_ab(self):
        r = self.rel("ubuntu", "ide", [])
        self.assertEqual(r["depends"][0], "vintagecoder-cli (= 0.1.0)")
        self.assertIn("libasound2t64 | libasound2", r["depends"])
        self.assertIn("libxkbcommon-x11-0", r["depends"])
        r = self.rel("ubuntu", "ide", [], version="0.1.0-alpha")
        self.assertEqual(r["depends"][0], "vintagecoder-cli (= 0.1.0~alpha)")
        r = self.rel("fedora", "ide", [])
        self.assertEqual(r["depends"][0], "vintagecoder-cli = 0.1.0-1")
        self.assertIn("alsa-lib", r["depends"])
        r = self.rel("fedora", "ide", [], version="0.1.0-alpha")
        self.assertEqual(r["depends"][0], "vintagecoder-cli = 0.1.0-0.1.alpha")
        r = self.rel("opensuse", "ide", [])
        self.assertIn("Mesa-libGL1", r["depends"])

    def test_anbindungen_aus_der_auswahl_des_staging(self):
        links = lp.read_selection(self.stage("nur-cli-atari"))[1]
        self.assertIn("atari800", links)
        self.assertNotIn("vice", links)
        r = self.rel("debian", "cli", links)
        self.assertEqual(r["recommends"], ["atari800"])
        links = lp.read_selection(self.stage("nur-ide-c64-ohne-emulator"))[1]
        self.assertEqual(self.rel("debian", "cli", links)["recommends"], [])

    # --- control und spec

    def test_control_cli(self):
        rel = self.rel("debian", "cli", ["atari800"])
        c = lp.render_control(self.cfg, "cli", "0.1.0", "amd64", rel, 1234)
        lines = c.splitlines()
        self.assertEqual(lines[0], "Package: vintagecoder-cli")
        for want in ("Version: 0.1.0", "Architecture: amd64", "Installed-Size: 1234", "Depends: libc6", "Recommends: atari800"):
            self.assertIn(want, lines)
        self.assertFalse([l for l in lines if l.startswith("Suggests")])
        self.assertTrue(c.endswith("\n"))
        i = [n for n, l in enumerate(lines) if l.startswith("Description: ")][0]
        self.assertTrue(all(l.startswith(" ") for l in lines[i + 1:]))

    def test_control_ide_vorabversion(self):
        rel = self.rel("ubuntu", "ide", [], version="0.1.0-alpha")
        c = lp.render_control(self.cfg, "ide", "0.1.0-alpha", "amd64", rel, 5)
        self.assertIn("Package: vintagecoder\n", c)
        self.assertIn("Version: 0.1.0~alpha\n", c)
        self.assertIn("Depends: vintagecoder-cli (= 0.1.0~alpha), libasound2t64 | libasound2,", c)

    def test_spec(self):
        files = lp.stage_files(self.stage("voll"))
        comps = lp.split_components(files, True)
        paths = lp.installed_paths(comps["cli"])
        spec = lp.render_spec(self.cfg, "cli", "0.1.0", self.rel("fedora", "cli", ["vice", "atari800"]), paths, 1767225600)
        self.assertIn("Name:           vintagecoder-cli\n", spec)
        self.assertIn("Version:        0.1.0\n", spec)
        self.assertIn("Release:        1\n", spec)
        self.assertIn("Recommends:     atari800\n", spec)
        self.assertIn("Suggests:       vice\n", spec)
        self.assertIn("/usr/bin/vcc\n", spec)
        self.assertIn("%dir /usr/share/vintagecoder\n", spec)
        self.assertNotIn("%dir /usr/share/man", spec)
        self.assertNotIn("/usr/bin/vintagecoder\n", spec)
        self.assertIn("* Thu Jan 01 2026 ", spec)
        ide = lp.render_spec(self.cfg, "ide", "0.1.0-alpha", self.rel("fedora", "ide", [], version="0.1.0-alpha"), lp.installed_paths(comps["ide"]), 1767225600)
        self.assertIn("Release:        0.1.alpha\n", ide)
        self.assertIn("Requires:       vintagecoder-cli = 0.1.0-0.1.alpha\n", ide)
        self.assertIn("/usr/bin/vintagecoder\n", ide)
        self.assertIn("/usr/share/applications/vintagecoder.desktop\n", ide)
        self.assertNotIn("/usr/bin/vcc\n", ide)

    def test_spec_leerzeichen_und_prozent(self):
        self.assertEqual(lp.rpm_quote("/usr/share/a b"), '"/usr/share/a b"')
        self.assertEqual(lp.rpm_quote("/usr/share/100%"), "/usr/share/100%%")
        with self.assertRaises(lp.PackageError):
            lp.rpm_quote("/usr/share/a*b")

    def test_besessene_verzeichnisse(self):
        d = lp.owned_dirs(["/usr/bin/vcc", "/usr/share/vintagecoder/data/a.json", "/usr/share/doc/vintagecoder/README.md", "/usr/share/man/man1/vcc.1.gz"])
        self.assertEqual(d, ["/usr/share/doc/vintagecoder", "/usr/share/vintagecoder", "/usr/share/vintagecoder/data"])

    # --- Dateibaum und Listen

    def test_payload_modi_und_gz(self):
        st = self.stage("nur-ide-c64-ohne-emulator")
        files = lp.stage_files(st)
        dest = os.path.join(self.tmp, "payload")
        paths = lp.build_payload(st, files, dest, lp.REPRODUCIBLE_EPOCH)
        self.assertEqual(paths, lp.installed_paths(files))
        self.assertEqual(os.stat(os.path.join(dest, "usr/bin/vcc")).st_mode & 0o777, 0o755)
        self.assertEqual(os.stat(os.path.join(dest, "usr/share/vintagecoder/data/release.json")).st_mode & 0o777, 0o644)
        import gzip
        with gzip.open(os.path.join(dest, "usr/share/man/man1/vcc.1.gz"), "rb") as g:
            self.assertTrue(g.read().startswith(b".\\\""))
        self.assertIn("usr/share/man/man1/vcc.1.gz", md5sums_paths(lp.md5sums(dest, paths)))

    def test_listen_vergleich(self):
        dpkg = ("drwxr-xr-x root/root 0 2026-01-01 00:00 ./\n-rwxr-xr-x root/root 10 2026-01-01 00:00 ./usr/bin/vcc\n"
                "lrwxrwxrwx root/root 0 2026-01-01 00:00 ./usr/bin/x -> vcc\n-rw-r--r-- root/root 5 2026-01-01 00:00 ./usr/share/a b.txt\n")
        self.assertEqual(lp.parse_dpkg_contents(dpkg), ["/usr/bin/vcc", "/usr/share/a b.txt"])
        rpm = "/usr/bin/vcc\n/usr/share/vintagecoder\n/usr/share/vintagecoder/data/x.json\n"
        self.assertEqual(lp.parse_rpm_list(rpm, ["/usr/share/vintagecoder"]), ["/usr/bin/vcc", "/usr/share/vintagecoder/data/x.json"])
        self.assertEqual(lp.verify_listing(["/a", "/b"], ["/a", "/b"], "p"), [])
        e = lp.verify_listing(["/a", "/b"], ["/a", "/c"], "p")
        self.assertEqual(e, ["p: fehlt im Paket: /b", "p: fremd im Paket: /c"])

    # --- package()

    def test_package_tar(self):
        st = self.stage("nur-cli-c64")
        out = os.path.join(self.tmp, "pk-tar")
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = release.main(["package", "--os", "linux", "--format", "tar", "--stage", st, "--out", out, "--reproducible"])
        self.assertEqual(rc, 0, buf.getvalue())
        names = [n for n in os.listdir(out) if n.endswith(".tar.gz")]
        self.assertEqual(len(names), 1)
        self.assertRegex(names[0], r"^vintagecoder-0\.1\.0-linux-[a-z0-9_]+\.tar\.gz$")
        import tarfile
        with tarfile.open(os.path.join(out, names[0])) as tf:
            tn = tf.getnames()
        top = names[0][:-len(".tar.gz")]
        self.assertIn(top + "/usr/bin/vcc", tn)
        self.assertIn(top + "/INSTALL.txt", tn)
        self.assertFalse([n for n in tn if "vintagecoder.desktop" in n])
        with open(os.path.join(out, "SHA256SUMS"), encoding="utf-8") as f:
            self.assertIn(names[0], f.read())

    def test_package_tar_reproduzierbar(self):
        st = self.stage("nur-cli-c64")
        sums = []
        for i in range(2):
            out = os.path.join(self.tmp, "pk-rep%d" % i)
            lp.package(st, out, "tar", None, "0.1.0", machine="x86_64", cfg=self.cfg, reproducible=True)
            sums.append(lp.sha256_file(os.path.join(out, "vintagecoder-0.1.0-linux-x86_64.tar.gz")))
        self.assertEqual(sums[0], sums[1])

    def test_package_fehler(self):
        st = self.stage("nur-cli-c64")
        out = os.path.join(self.tmp, "pk-err")
        with self.assertRaises(lp.PackageError):                                  # deb ohne Distribution
            lp.package(st, out, "deb", None, "0.1.0", cfg=self.cfg)
        with self.assertRaises(lp.PackageError):                                  # Format passt nicht zur Distribution
            lp.package(st, out, "rpm", "debian", "0.1.0", cfg=self.cfg)
        with self.assertRaises(lp.PackageError):                                  # kein Staging
            lp.package(self.tmp, out, "tar", None, "0.1.0", cfg=self.cfg)
        with self.assertRaises(lp.PackageError):                                  # keine SemVer-Version
            lp.package(st, out, "tar", None, "0.1", cfg=self.cfg)
        nobin = os.path.join(self.tmp, "st-nobin")
        buf = io.StringIO()
        with redirect_stdout(buf):
            release.main(["stage", "--os", "linux", "--out", nobin, "--no-bin", "--force", "--features", os.path.join(TESTDATA, "nur-cli-c64.md")])
        with self.assertRaisesRegex(lp.PackageError, "usr/bin/vcc fehlt"):
            lp.package(nobin, out, "tar", None, "0.1.0", cfg=self.cfg)

    @unittest.skipUnless(shutil.which("dpkg-deb"), "dpkg-deb nicht vorhanden (Bau-Container Debian/Ubuntu)")
    def test_deb_bauen(self):
        st = self.stage("voll")
        out = os.path.join(self.tmp, "pk-deb")
        res = lp.package(st, out, "deb", "debian", "0.1.0", arch="amd64", cfg=self.cfg, reproducible=True)
        self.assertEqual(sorted(n for n, _ in res), ["vintagecoder-cli_0.1.0_amd64.deb", "vintagecoder_0.1.0_amd64.deb"])
        cli = lp.parse_dpkg_contents(lp.run(["dpkg-deb", "-c", os.path.join(out, "vintagecoder-cli_0.1.0_amd64.deb")]))
        with open(os.path.join(out, "vintagecoder-cli_0.1.0_amd64.deb.files"), encoding="utf-8") as f:
            self.assertEqual(lp.verify_listing([l.strip() for l in f if l.strip()], cli, "cli"), [])
        info = lp.run(["dpkg-deb", "-f", os.path.join(out, "vintagecoder_0.1.0_amd64.deb"), "Depends"])
        self.assertIn("vintagecoder-cli (= 0.1.0)", info)

    @unittest.skipUnless(shutil.which("rpmbuild"), "rpmbuild nicht vorhanden (Bau-Container Fedora/openSUSE)")
    def test_rpm_bauen(self):
        st = self.stage("voll")
        out = os.path.join(self.tmp, "pk-rpm")
        res = lp.package(st, out, "rpm", "fedora", "0.1.0", arch="x86_64", cfg=self.cfg, reproducible=True)
        names = sorted(n for n, k in res if k == "rpm")
        self.assertEqual(names, ["vintagecoder-0.1.0-1.x86_64.rpm", "vintagecoder-cli-0.1.0-1.x86_64.rpm"])


def is_ide(rel):
    return lp.is_ide_file(rel)


def md5sums_paths(text):
    return [l.split("  ", 1)[1] for l in text.splitlines()]


if __name__ == "__main__":
    unittest.main()
