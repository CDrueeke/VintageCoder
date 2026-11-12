#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Verbotsscan und Ausschlussliste fuer den Export ins oeffentliche Repository (Paket P10, Aufrufer: tools/export-public.sh).

Unterbefehle:
  scan ORDNER          Verbotsscan ueber alle Dateien (Inhalt und Dateinamen) des Exportordners. Rueckgabewert 1 bei Funden.
  exclude ORDNER LISTE Dateien der Ausschlussliste (tools/export-exclude.txt) aus dem Exportordner loeschen.
  replace ORDNER LISTE Dateien ersetzen (tools/export-replace.txt, je Zeile `QUELLE ZIEL`): QUELLE wird nach ZIEL verschoben und
                       ueberschreibt es (z. B. die oeffentliche Fassung des Aenderungsprotokolls).

Der Scan sucht nach: Namen anderer Entwicklungsumgebungen (Wortliste verdeckt in release/registry.json, von tools/release.py
zusammengesetzt), festen Pfaden, privaten Namen, E-Mail-Adressen, internen IP-Adressen, Geheimnismustern, internen Ordnern und
Dateien (assets/, .claude/, target/, build/, Konzepte, Mockups, .DS_Store), zu grossen Dateien, nicht erlaubten Binaerdateien
und Co-Autor-Zeilen. Jeder Fund steht als `Datei:Zeile: [Regel] Text`. Hinweise (Warnungen) lassen den Rueckgabewert bei 0.

Aufruf:
  python3 tools/export_check.py scan  /tmp/export
  python3 tools/export_check.py exclude /tmp/export tools/export-exclude.txt
Nur Standardbibliothek. Keine Namen anderer Entwicklungsumgebungen im Klartext: die Woerter kommen aus der Registry."""
import argparse, fnmatch, json, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import release  # noqa: E402  (ide_words, technical_ide_token, load_json)

MAX_SIZE = 2 * 1024 * 1024
BINARY_EXT = {".png", ".icns", ".ico", ".otf", ".ttf"}
# Bekannte Testdaten (Binaerdateien): Ordnerpraefixe (relativ zum Wurzelordner)
BINARY_DIRS = ("crates/vc-emu/tests/data/", "crates/vc-basic/tests/fixtures/")

# Dateien, die die gesuchten Muster selbst beschreiben (Regeln, Testdaten, Ausschlussbegruendungen): dort keine Funde fuer die Musterregeln.
SELF_FILES = {"tools/export_check.py", "tools/test_export_check.py", "tools/export-public.sh", "tools/export-exclude.txt", "tools/export-replace.txt"}
# Wortlisten-Dateien: enthalten die Namen nur in Teilen, aber zusammengesetzt koennten Tests sie nennen
WORDLIST_FILES = {"release/registry.json", "tools/release.py", "tools/test_release.py"} | SELF_FILES
# Ordner, in denen der technische Name der Erweiterungsschnittstelle (Kleinschreibung) vorkommen darf
TECH_ALLOW_DIRS = ("editors/ext/",)

ALLOWED_EMAILS = {"vintagecoder@drueeke.de"}
ALLOWED_EMAIL_SUFFIXES = ("@users.noreply.github.com",)
# Beispieladressen aus Handbuch und Dokumentation (192.168.1.64: Beispielgeraet in den `vcc streams`-Beispielen)
ALLOWED_IPS = {"192.168.0.10", "192.168.1.50", "192.168.1.64"}
# Namen des Autors/Herausgebers in GitHub-Adressen sind gewollt
ALLOWED_PRIVATE_NAME_RX = re.compile(r"github\.com/CDrueeke/|CDrueeke/VintageCoder|\+CDrueeke@users\.noreply\.github\.com", re.I)
PRIVATE_NAME = "cdrueeke"

EMAIL_RX = re.compile(r"(?<![A-Za-z0-9._%+-])[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}\b")
IP_RX = re.compile(r"(?<![\d.])(192\.168\.\d{1,3}\.\d{1,3}|10\.\d{1,3}\.\d{1,3}\.\d{1,3}|172\.(?:1[6-9]|2\d|3[01])\.\d{1,3}\.\d{1,3})(?![\d.]*\d)")
# Feste Pfade: `/Users/<Name>`, `/Volumes/<Name>`, `C:\Users\<Name>`. Ein blosses `/Users/` (Text, Pruefung `starts_with`) oder
# ein Platzhalter (`/Users/<name>`) ist kein Fund; erfundene Beispielnamen aus Tests und Hilfetexten sind erlaubt.
USER_PATH_RX = re.compile(r"(?:/Users/|/Volumes/|[A-Za-z]:[\\/]+Users[\\/]+|\\+Users\\+)([A-Za-z0-9._-]+)", re.I)
EXAMPLE_NAMES = {"anna", "annabelle", "ich", "a", "x", "bob", "chris", "name", "user", "demo"}
TEST_RX = re.compile(r"(^|/)(tests?|fixtures)(/|\.rs$)|(^|/)tests\.rs$")
SECRET_RXS = [
    ("geheimnis", re.compile(r"""(?:password|passwd|token|secret|api[_-]?key)["']?\s*[:=]\s*["'][^"'\s]{4,}["']""", re.I)),
    ("geheimnis", re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----")),
]
COAUTHOR_RX = re.compile(r"Co-Authored", re.I)
# Hinweise (kein Fehler): Namen von Werkzeugen der Entwicklung, schwache Woerter (viele Fehlalarme)
HINT_WORDS_CASE = ["Claude", "Anthropic"]
WEAK_WORD_PARTS = [["v", "im"], ["Ze", "d"], ["Hel", "ix"], ["Notepad", "++"]]

# Pfadbestandteile bzw. Namen, die nie in den Export gehoeren
BAD_TOP_DIRS = {"assets", ".claude", "target", "build"}
BAD_ANY_DIRS = {"build", "target", ".claude"}
BAD_NAME_RX = re.compile(r"konzept|mockup|^\.DS_Store$", re.I)
TEXT_SNIFF = 8192


class Finding:
    def __init__(self, path, line, rule, text, level="Fehler"):
        self.path, self.line, self.rule, self.text, self.level = path, line, rule, text, level

    def __str__(self):
        loc = self.path if not self.line else "%s:%d" % (self.path, self.line)
        return "%s: [%s%s] %s" % (loc, self.rule, "" if self.level == "Fehler" else ", Hinweis", self.text)


def user_path(text):
    """Erster fester Pfad mit echtem Namen (kein Platzhalter, kein erfundener Beispielname) oder None."""
    for m in USER_PATH_RX.finditer(text):
        if m.group(1).lower().split(".")[0] not in EXAMPLE_NAMES:
            return m.group(0)
    return None


def short(s, n=90):
    s = s.strip()
    return s if len(s) <= n else s[:n - 1] + "…"


def build_word_rx(words):
    words = [w for w in words if w]
    return re.compile("|".join(re.escape(w) for w in words), re.I) if words else None


def under(rel, prefixes):
    return any(rel.startswith(p) for p in prefixes)


def is_binary(head):
    return b"\0" in head


def scan_name(rel, word_rx):
    out = []
    parts = rel.split("/")
    if parts[0] in BAD_TOP_DIRS:
        out.append(Finding(rel, 0, "verbotener-ordner", "Ordner '%s/' gehoert nicht in den Export" % parts[0]))
    for d in parts[1:-1]:
        if d in BAD_ANY_DIRS and parts[0] not in BAD_TOP_DIRS:
            out.append(Finding(rel, 0, "verbotener-ordner", "Ordner '%s/' gehoert nicht in den Export" % d))
            break
    if BAD_NAME_RX.search(parts[-1]):
        out.append(Finding(rel, 0, "datei-name", "Konzept-/Mockup-Datei oder .DS_Store"))
    if word_rx and rel not in WORDLIST_FILES:
        m = word_rx.search(rel)
        if m:
            out.append(Finding(rel, 0, "ide-wort", "Dateiname nennt '%s'" % m.group(0)))
    if user_path(rel):
        out.append(Finding(rel, 0, "fester-pfad", "Dateiname enthaelt festen Pfad"))
    if PRIVATE_NAME in rel.lower():
        out.append(Finding(rel, 0, "privater-name", "Dateiname enthaelt '%s'" % PRIVATE_NAME))
    return out


def scan_line(rel, n, line, ctx):
    """Funde einer Textzeile. ctx: word_rx, tech_rx, hint_rx, weak_rx."""
    out = []
    selfdesc = rel in SELF_FILES
    if rel not in WORDLIST_FILES:
        if ctx["word_rx"]:
            for m in ctx["word_rx"].finditer(line):
                # der technische Name der Erweiterungsschnittstelle steht auch in der Wortliste: dort erlaubt
                if under(rel, TECH_ALLOW_DIRS) and m.group(0).lower() == release.technical_ide_token():
                    continue
                out.append(Finding(rel, n, "ide-wort", "nennt '%s'" % m.group(0)))
                break
        if ctx["tech_rx"] and not under(rel, TECH_ALLOW_DIRS):
            m = ctx["tech_rx"].search(line)
            if m:
                out.append(Finding(rel, n, "ide-technisch", "technischer Name '%s' ausserhalb der Editor-Erweiterung" % m.group(0)))
        if ctx["weak_rx"]:
            m = ctx["weak_rx"].search(line)
            if m:
                out.append(Finding(rel, n, "ide-wort", "mögliches Werkzeug '%s'" % m.group(0), "Hinweis"))
    if selfdesc:
        return out
    if user_path(line):
        out.append(Finding(rel, n, "fester-pfad", short(line)))
    if PRIVATE_NAME in line.lower() and not ALLOWED_PRIVATE_NAME_RX.search(line):
        out.append(Finding(rel, n, "privater-name", short(line)))
    for m in EMAIL_RX.finditer(line):
        a = m.group(0)
        if a.lower() in ALLOWED_EMAILS or a.lower().endswith(ALLOWED_EMAIL_SUFFIXES):
            continue
        out.append(Finding(rel, n, "e-mail", a))
    # Testdateien und Pruefzeilen (`assert…`): erfundene Testwerte (Adressen, Kennwoerter) sind nur ein Hinweis
    level = "Hinweis" if TEST_RX.search(rel) or "assert" in line else "Fehler"
    for m in IP_RX.finditer(line):
        if m.group(1) not in ALLOWED_IPS:
            out.append(Finding(rel, n, "interne-ip", m.group(1), level))
    for rule, rx in SECRET_RXS:
        if rx.search(line):
            out.append(Finding(rel, n, rule, short(line), level))
    if COAUTHOR_RX.search(line):
        out.append(Finding(rel, n, "co-autor", short(line)))
    if ctx["hint_rx"] and ctx["hint_rx"].search(line) and rel != "crates/vc-app/src/about.rs":
        out.append(Finding(rel, n, "werkzeug-name", short(line), "Hinweis"))
    return out


def scan_tree(root, reg=None):
    """Alle Funde (Fehler und Hinweise) im Ordner root, sortiert nach Pfad."""
    if reg is None:
        reg = release.load_json(os.path.join(root, "release", "registry.json"))
    words = release.ide_words(reg)
    ctx = {
        "word_rx": build_word_rx(words),
        "tech_rx": re.compile(re.escape(release.technical_ide_token()), re.I),
        "hint_rx": re.compile("|".join(r"\b%s\b" % re.escape(w) for w in HINT_WORDS_CASE)),
        "weak_rx": re.compile("|".join(r"\b%s\b" % re.escape("".join(p)) for p in WEAK_WORD_PARTS)),
    }
    findings = []
    for d, dirs, files in os.walk(root):
        dirs[:] = sorted(x for x in dirs if x != ".git")
        for f in sorted(files):
            p = os.path.join(d, f)
            rel = os.path.relpath(p, root).replace(os.sep, "/")
            findings += scan_name(rel, ctx["word_rx"])
            if os.path.islink(p):
                continue
            size = os.path.getsize(p)
            if size > MAX_SIZE:
                findings.append(Finding(rel, 0, "zu-gross", "%.1f MiB (Grenze 2 MiB)" % (size / 1048576)))
            with open(p, "rb") as fh:
                head = fh.read(TEXT_SNIFF)
                if is_binary(head):
                    ext = os.path.splitext(f)[1].lower()
                    if ext not in BINARY_EXT and not under(rel, BINARY_DIRS):
                        findings.append(Finding(rel, 0, "binaer", "Binaerdatei, Typ nicht in der Allowlist"))
                    continue
                fh.seek(0)
                for n, raw in enumerate(fh, 1):
                    findings += scan_line(rel, n, raw.decode("utf-8", errors="replace"), ctx)
    findings.sort(key=lambda x: (x.path, x.line, x.rule))
    return findings


def read_excludes(path):
    """Muster der Ausschlussliste: eine Zeile je Datei oder Glob, `#` beginnt einen Kommentar, Leerzeilen werden ignoriert."""
    out = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.split("#", 1)[0].strip()
            if line:
                out.append(line.strip("/"))
    return out


def apply_excludes(root, patterns):
    """Loescht passende Dateien; gibt (geloescht, ohne_treffer) zurueck. Pfade relativ zum Wurzelordner; Muster nach fnmatch auf dem ganzen Pfad (`*` gilt auch ueber Ordnergrenzen)."""
    files = []
    for d, dirs, fs in os.walk(root):
        dirs[:] = [x for x in dirs if x != ".git"]
        for f in fs:
            files.append(os.path.relpath(os.path.join(d, f), root).replace(os.sep, "/"))
    removed, unmatched = [], []
    for pat in patterns:
        hits = [f for f in files if f == pat or fnmatch.fnmatchcase(f, pat)]
        if not hits:
            unmatched.append(pat)
        for h in hits:
            if h not in removed:
                os.remove(os.path.join(root, h))
                removed.append(h)
    # leere Ordner entfernen
    for d, dirs, fs in os.walk(root, topdown=False):
        if d != root and ".git" not in d.split(os.sep) and not os.listdir(d):
            os.rmdir(d)
    return sorted(removed), unmatched


def read_replacements(path):
    """Zeilen `QUELLE ZIEL` (Pfade ab Wurzel); `#` beginnt einen Kommentar."""
    out = []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            parts = line.split()
            if len(parts) != 2:
                raise ValueError("Zeile braucht genau `QUELLE ZIEL`: %r" % line)
            out.append((parts[0], parts[1]))
    return out


def apply_replacements(root, pairs):
    """Verschiebt QUELLE nach ZIEL (ueberschreibt). Gibt (ersetzt, fehlend) zurueck."""
    done, missing = [], []
    for src, dst in pairs:
        s, d = os.path.join(root, src), os.path.join(root, dst)
        if not os.path.isfile(s):
            missing.append(src)
            continue
        os.makedirs(os.path.dirname(d) or root, exist_ok=True)
        os.replace(s, d)
        done.append((src, dst))
    for dd, dirs, fs in os.walk(root, topdown=False):
        if dd != root and ".git" not in dd.split(os.sep) and not os.listdir(dd):
            os.rmdir(dd)
    return done, missing


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("scan", help="Verbotsscan eines Exportordners")
    s.add_argument("ordner")
    e = sub.add_parser("exclude", help="Ausschlussliste anwenden")
    e.add_argument("ordner")
    e.add_argument("liste")
    r = sub.add_parser("replace", help="Dateien ersetzen (QUELLE ZIEL)")
    r.add_argument("ordner")
    r.add_argument("liste")
    a = ap.parse_args(argv)
    if not os.path.isdir(a.ordner):
        print("Ordner fehlt: %s" % a.ordner, file=sys.stderr)
        return 2
    if a.cmd == "exclude":
        removed, unmatched = apply_excludes(a.ordner, read_excludes(a.liste))
        for r in removed:
            print("ausgeschlossen: %s" % r)
        for u in unmatched:
            print("Hinweis: Ausschluss '%s' trifft keine Datei" % u)
        return 0
    if a.cmd == "replace":
        done, missing = apply_replacements(a.ordner, read_replacements(a.liste))
        for s, d in done:
            print("ersetzt: %s <- %s" % (d, s))
        for m in missing:
            print("FEHLER: Quelle '%s' fehlt" % m, file=sys.stderr)
        return 1 if missing else 0
    findings = scan_tree(a.ordner)
    errs = [f for f in findings if f.level == "Fehler"]
    for f in findings:
        print(f)
    print("Verbotsscan: %d Fund(e), %d Hinweis(e)" % (len(errs), len(findings) - len(errs)))
    return 1 if errs else 0


if __name__ == "__main__":
    sys.exit(main())
