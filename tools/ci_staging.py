#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Staging-Pruefung je Testauswahl (Konzept Release-Auswahl, Abschnitt 9, Etappe 5).

Fuer jede Datei unter tools/testdata/*.md (Testauswahlen, keine benannten Voreinstellungen): `release.py stage` fuer das eigene
Betriebssystem mit --verify-build (jedes auszuliefernde Beispiel wird gebaut, Verbotsscan), dann mit dem Staging-`vcc`:
  * data/release.json nennt genau die gewaehlten Anbindungen und Zielsysteme,
  * `vcc emulator list` zeigt genau die gewaehlten Emulatoren, abgewaehlte melden „nicht im Paket enthalten“ (Rueckgabewert 1),
  * `vcc new` + `vcc build` je gewaehltem Zielsystem gelingt, ein abgewaehltes Zielsystem wird mit Rueckgabewert 2 abgelehnt,
  * `vcc help` laeuft.
Aufruf:  python3 tools/ci_staging.py [--bin-dir target/debug] [--only nur-cli-atari] [-v]
Kein Fenster, keine Emulatoren, kein Netz noetig. Rueckgabewert 0 = alles gut."""
import argparse, glob, json, os, shutil, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import release  # noqa: E402  (stage_layout: Linux hat einen FHS-Baum, sonst liegen data/ und bin/ im Staging-Ordner)
EMULATORS = ["vice", "atari800", "altirra", "fujisan", "xemu"]
TARGETS = ["c64", "atari8", "mega65", "plus4", "c128", "vic20", "pet", "cbm2"]
HOST = {"darwin": "macos", "win32": "windows"}.get(sys.platform, "linux")


def run(cmd, env, cwd, timeout=600):
    return subprocess.run(cmd, env=env, cwd=cwd, capture_output=True, text=True, timeout=timeout, stdin=subprocess.DEVNULL)


def check_selection(path, bin_dir, verbose):
    name = os.path.splitext(os.path.basename(path))[0]
    errs = []
    tmp = tempfile.mkdtemp(prefix="vc-ci-stage-")
    try:
        out = os.path.join(tmp, "stage")
        r = run([sys.executable, os.path.join(ROOT, "tools", "release.py"), "stage", "--os", HOST, "--features", path, "--bin-dir", bin_dir, "--out", out, "--force", "--verify-build"], os.environ.copy(), ROOT, 1800)
        if verbose or r.returncode:
            print(r.stdout + r.stderr)
        if r.returncode:
            return ["%s: stage fehlgeschlagen (Rueckgabewert %d)" % (name, r.returncode)]
        lay = release.stage_layout(HOST, out)
        with open(os.path.join(lay["share"], "data", "release.json"), encoding="utf-8") as f:
            sel = json.load(f)
        links, targets = set(sel["links"]), set(sel["targets"])
        vcc = os.path.join(lay["bin"], release.exe_name("vcc", HOST))
        env = {k: v for k, v in os.environ.items() if not k.startswith("VC_")}
        env.update(HOME=tmp, USERPROFILE=tmp, APPDATA=os.path.join(tmp, "appdata"), XDG_CONFIG_HOME=os.path.join(tmp, "cfg"), XDG_DATA_HOME=os.path.join(tmp, "data"))

        def vcc_run(*a):
            return run([vcc] + list(a), env, tmp)

        # Emulatoren: genau die gewaehlten
        r = vcc_run("emulator", "list")
        if r.returncode:
            errs.append("%s: vcc emulator list: Rueckgabewert %d: %s" % (name, r.returncode, r.stderr.strip()))
        shown = {l.split()[0] for l in r.stdout.splitlines() if l and not l[0].isspace()}
        want = {e for e in EMULATORS if e in links} | {d for d in ("ultimate", "mega65") if d in links}
        if shown != want:
            errs.append("%s: vcc emulator list zeigt %s, erwartet %s" % (name, sorted(shown), sorted(want)))
        for e in EMULATORS:
            if e not in links:
                r = vcc_run("emulator", "status", e)
                if r.returncode != 1 or "nicht im Paket enthalten" not in (r.stdout + r.stderr):
                    errs.append("%s: vcc emulator status %s: erwartet Rueckgabewert 1 und 'nicht im Paket enthalten', war %d: %s" % (name, e, r.returncode, (r.stdout + r.stderr).strip()))
        # Zielsysteme: gewaehlte bauen, abgewaehlte werden abgelehnt
        for t in TARGETS:
            proj = os.path.join(tmp, "p-" + t)
            r = vcc_run("new", proj, "--target", t)
            if t in targets:
                if r.returncode:
                    errs.append("%s: vcc new --target %s: %s" % (name, t, (r.stderr or r.stdout).strip()))
                    continue
                b = vcc_run("build", proj)
                if b.returncode:
                    errs.append("%s: vcc build (neues %s-Projekt): %s" % (name, t, (b.stderr or b.stdout).strip()[-300:]))
            elif r.returncode == 0:
                errs.append("%s: vcc new --target %s haette abgelehnt werden muessen (Ziel nicht gewaehlt)" % (name, t))
        r = vcc_run("help")
        if r.returncode:
            errs.append("%s: vcc help: Rueckgabewert %d" % (name, r.returncode))
        print("%-30s %s  (Anbindungen: %s; Ziele: %s)" % (name, "FEHLER" if errs else "ok", ", ".join(sorted(links)) or "-", ", ".join(sorted(targets)) or "-"))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return errs


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--bin-dir", default=os.path.join(ROOT, "target", "debug"), help="Ordner mit vcc, vc-lsp (und vintagecoder fuer Auswahlen mit IDE)")
    ap.add_argument("--only", help="nur diese Testauswahl (Dateiname ohne .md)")
    ap.add_argument("-v", "--verbose", action="store_true")
    a = ap.parse_args(argv)
    files = sorted(glob.glob(os.path.join(ROOT, "tools", "testdata", "*.md")))
    if a.only:
        files = [f for f in files if os.path.splitext(os.path.basename(f))[0] == a.only]
    if not files:
        print("keine Testauswahl gefunden")
        return 2
    errs = []
    for f in files:
        errs += check_selection(f, os.path.abspath(a.bin_dir), a.verbose)
    for e in errs:
        print("FEHLER:  " + e)
    print("%d Testauswahl(en), %d Fehler" % (len(files), len(errs)))
    return 1 if errs else 0


if __name__ == "__main__":
    sys.exit(main())
