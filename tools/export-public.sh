#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Sauberer Export eines Standes ins oeffentliche Repository: ein Commit, ohne Verlauf. Pusht nie und setzt keinen Tag.
set -euo pipefail

usage() {
  cat <<'EOF'
export-public.sh – sauberen Export fuer das oeffentliche Repository erzeugen (ein Commit, kein Verlauf)

Aufruf:
  tools/export-public.sh [Optionen] AUSGABEORDNER

Das Skript exportiert einen festen Commit des privaten Entwicklungs-Repositorys mit `git archive` (nur getrackte Dateien),
entfernt die Dateien aus tools/export-exclude.txt, ersetzt die aus tools/export-replace.txt, prueft den Export mit tools/export_check.py (Verbotsscan) und legt im
Ausgabeordner ein neues Git-Repository mit genau einem Commit an. Es pusht NIE und setzt KEINEN Tag.

Optionen:
  --commit REF                 Stand, der exportiert wird (Standard: main). Muss existieren.
  --verify                     Aus dem Export in einem frischen Ordner bauen und testen: cargo build/test (Arbeitsbereich),
                               Python-Tests, `release.py check`, Release-Build von vcc und `vcc --version`. Eigener Build-Ordner
                               ausserhalb (Standard ~/.cache/vintagecoder-target-export-verify, aenderbar mit CARGO_TARGET_DIR).
  --commit-message-file DATEI  Nachricht des Einstiegs-Commits aus dieser Datei statt des Standardtexts.
  --allow-dirty                Arbeitsbaum nicht auf Sauberkeit pruefen (nur fuer Proben; der Export kommt immer aus dem Commit).
  -h, --help                   diese Hilfe

Der AUSGABEORDNER muss leer sein oder nicht existieren; nichts wird ueberschrieben.
Autor und Committer des Einstiegs-Commits: Christian Drüeke <165030641+CDrueeke@users.noreply.github.com> (nur fuer diesen
Commit ueber GIT_AUTHOR_*/GIT_COMMITTER_*, keine globale Konfiguration, kein Co-Authored-By).

Beispiele:
  tools/export-public.sh /tmp/vc-export
  tools/export-public.sh --commit v0.1.0-rc --verify ~/Development/export-probe
  tools/export-public.sh --commit-message-file msg.txt /tmp/vc-export

Danach (nur der Projektinhaber, von Hand; das Skript tut nichts davon):
  cd AUSGABEORDNER
  git remote add origin https://github.com/CDrueeke/VintageCoder.git
  git tag -a v0.1.0 -m "VintageCoder 0.1.0 (Alpha)"
  git push origin main v0.1.0
Vorher das neue Repository leer und privat anlegen; erst nach der Pruefung mit `git ls-remote` auf oeffentlich stellen.
Rueckgabewert 0 = Export sauber; sonst Funde oder Fehler (der Ausgabeordner bleibt zur Ansicht stehen, ohne Commit bei Funden).
EOF
}

AUTHOR_NAME="Christian Drüeke"
AUTHOR_EMAIL="165030641+CDrueeke@users.noreply.github.com"
read -r -d '' DEFAULT_MESSAGE <<'EOF' || true
VintageCoder 0.1.0 (Alpha): erster öffentlicher Stand

Entwicklungsumgebung für 6502-Systeme (C64, C128, VC-20, Plus/4, PET/CBM, Atari 8 Bit, MEGA65). Lizenz GPL-3.0-or-later. Die frühere Entwicklungsgeschichte ist nicht Teil dieses Repositorys.
EOF

commit_ref="main"; verify=0; msg_file=""; allow_dirty=0; out=""
while [ $# -gt 0 ]; do
  case "$1" in
    -h|--help) usage; exit 0 ;;
    --commit) commit_ref="${2:?--commit braucht einen Wert}"; shift 2 ;;
    --verify) verify=1; shift ;;
    --commit-message-file) msg_file="${2:?--commit-message-file braucht eine Datei}"; shift 2 ;;
    --allow-dirty) allow_dirty=1; shift ;;
    -*) echo "Unbekannte Option: $1 (siehe --help)" >&2; exit 2 ;;
    *) [ -z "$out" ] || { echo "Nur ein Ausgabeordner erlaubt (siehe --help)" >&2; exit 2; }; out="$1"; shift ;;
  esac
done
[ -n "$out" ] || { echo "Ausgabeordner fehlt (siehe --help)" >&2; exit 2; }
[ -z "$msg_file" ] || [ -f "$msg_file" ] || { echo "Nachrichtendatei fehlt: $msg_file" >&2; exit 2; }

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
fail() { echo "FEHLER: $*" >&2; exit 1; }

# 1. Voraussetzungen: Commit vorhanden, Arbeitsbaum sauber, Ausgabeordner leer
commit="$(git rev-parse --verify --quiet "${commit_ref}^{commit}")" || fail "Commit '$commit_ref' existiert nicht"
if [ "$allow_dirty" -eq 0 ] && [ -n "$(git status --short)" ]; then
  git status --short | head -20 >&2
  fail "Arbeitsbaum nicht sauber (git status --short ist nicht leer)"
fi
if [ -e "$out" ]; then
  [ -d "$out" ] || fail "$out ist kein Ordner"
  [ -z "$(ls -A "$out")" ] || fail "Ausgabeordner $out ist nicht leer (nichts wird ueberschrieben)"
fi
mkdir -p "$out"
out="$(cd "$out" && pwd)"
case "$out" in "$root"|"$root"/*) fail "Der Ausgabeordner darf nicht im Repository liegen" ;; esac
echo "Export von $(git rev-parse --short "$commit") ($commit_ref) nach $out"

# 2. Export: nur getrackte Dateien
git archive --format=tar "$commit" | tar -x -C "$out"
expected="$(git ls-tree -r --name-only "$commit" | wc -l | tr -d ' ')"

# 3. Ausschlussliste: die Fassung des exportierten Standes, sonst die dieses Arbeitsbaums
exclude_list="$out/tools/export-exclude.txt"
[ -f "$exclude_list" ] || { exclude_list="$root/tools/export-exclude.txt"; echo "Hinweis: Ausschlussliste aus dem Arbeitsbaum (der Stand hat keine)"; }
python3 -I "$root/tools/export_check.py" exclude "$out" "$exclude_list"
replace_list="$out/tools/export-replace.txt"
[ -f "$replace_list" ] || replace_list="$root/tools/export-replace.txt"
if [ -f "$replace_list" ]; then python3 -I "$root/tools/export_check.py" replace "$out" "$replace_list"; fi
files_after="$(find "$out" -type f -o -type l | wc -l | tr -d ' ')"

# 4. Verbotsscan (Werkzeug aus diesem Arbeitsbaum, Wortliste aus dem Export)
scan_status=0
python3 -I "$root/tools/export_check.py" scan "$out" || scan_status=$?

# 5. Gegenprobe mit Git: Dateizahl des Exports = getrackte Dateien des Commits abzueglich Ausschluesse
(cd "$out" && git init -q -b main --template= && git add -A)
tracked="$(git -C "$out" ls-files | wc -l | tr -d ' ')"
if [ "$tracked" != "$files_after" ] || [ "$files_after" -gt "$expected" ]; then
  fail "Dateizahl passt nicht: Export $files_after, git ls-files $tracked, Commit $expected"
fi
bad="$(git -C "$out" ls-files | grep -E '^(assets|\.claude|target|build)/|/build/|(^|/)\.DS_Store$' || true)"
[ -z "$bad" ] || { echo "$bad" >&2; fail "Verbotene Pfade in git ls-files"; }

if [ "$scan_status" -ne 0 ]; then
  rm -rf "$out/.git"
  fail "Verbotsscan hat Funde (siehe oben). Der Export in $out bleibt zur Ansicht, hat aber kein Repository und keinen Commit."
fi

# 6. Einstiegs-Commit: Autor = Committer = noreply-Adresse, nur fuer diesen Commit gesetzt
if [ -n "$msg_file" ]; then msgpath="$msg_file"; else msgpath="$(mktemp)"; printf '%s\n' "$DEFAULT_MESSAGE" > "$msgpath"; fi
GIT_AUTHOR_NAME="$AUTHOR_NAME" GIT_AUTHOR_EMAIL="$AUTHOR_EMAIL" GIT_COMMITTER_NAME="$AUTHOR_NAME" GIT_COMMITTER_EMAIL="$AUTHOR_EMAIL" \
  git -C "$out" -c commit.gpgsign=false -c core.hooksPath=/dev/null -c gc.auto=0 -c maintenance.auto=false commit -q -F "$msgpath"
[ -n "$msg_file" ] || rm -f "$msgpath"

# Kontrolle des Commits
[ "$(git -C "$out" rev-list --all --count)" = "1" ] || fail "Mehr als ein Commit"
who="$(git -C "$out" log --format='%an <%ae> | %cn <%ce>')"
[ "$who" = "$AUTHOR_NAME <$AUTHOR_EMAIL> | $AUTHOR_NAME <$AUTHOR_EMAIL>" ] || fail "Autor/Committer falsch: $who"
[ -z "$(git -C "$out" tag)" ] || fail "Es gibt Tags"
[ "$(git -C "$out" branch --format='%(refname:short)')" = "main" ] || fail "Es gibt weitere Branches"
git -C "$out" log -1 --format=%B | grep -qi 'co-authored' && fail "Co-Authored-Zeile in der Nachricht"
[ -z "$(git -C "$out" status --short)" ] || fail "Arbeitsbaum des Exports nicht sauber"

# 7. Optional: Bauen und Testen aus dem Export in einem frischen Ordner
if [ "$verify" -eq 1 ]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  git clone -q --no-hardlinks "$out" "$tmp/export"
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/vintagecoder-target-export-verify}"
  export CARGO_INCREMENTAL=0
  log="$out.verify.log"
  echo "Pruefung aus dem Export (Build-Ordner $CARGO_TARGET_DIR, Protokoll $log)"
  (
    cd "$tmp/export"
    set -x
    cargo build --workspace --all-targets
    cargo test --workspace --no-fail-fast
    python3 tools/release.py check
    python3 -m unittest discover -s tools -p 'test_*.py'
    cargo build --release -p vc-cli
    version="$(sed -n 's/^version *= *"\(.*\)"/\1/p' Cargo.toml | head -1)"
    got="$("$CARGO_TARGET_DIR/release/vcc" --version)"
    echo "vcc --version: $got (Cargo.toml: $version)"
    case "$got" in *"$version"*) ;; *) echo "Versionsabweichung" >&2; exit 1 ;; esac
  ) > "$log" 2>&1 || { tail -40 "$log" >&2; fail "Pruefung aus dem Export fehlgeschlagen (siehe $log)"; }
  echo "Pruefung aus dem Export: bestanden"
fi

# 8. Zusammenfassung
size="$(python3 -I -c 'import os,sys
t=0
for d,ds,fs in os.walk(sys.argv[1]):
    ds[:]=[x for x in ds if x!=".git"]
    t+=sum(os.path.getsize(os.path.join(d,f)) for f in fs if not os.path.islink(os.path.join(d,f)))
print(t//1024)' "$out")"
echo
echo "Export fertig: $tracked Dateien, $((size / 1024)) MiB (ohne .git), 1 Commit $(git -C "$out" rev-parse --short HEAD), Funde: 0"
echo "Ordner: $out"
echo "Nichts gepusht, kein Tag gesetzt. Naechste Schritte stehen in: tools/export-public.sh --help"
