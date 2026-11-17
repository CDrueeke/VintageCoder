#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Export als neuer Commit auf dem bestehenden oeffentlichen Stand (Update nach dem ersten Release). Pusht nie und setzt keinen Tag.
set -euo pipefail

PUBLIC_URL="https://github.com/CDrueeke/VintageCoder.git"
AUTHOR_NAME="Christian Drüeke"
AUTHOR_EMAIL="165030641+CDrueeke@users.noreply.github.com"

usage() {
  cat <<'EOF'
export-update.sh – einen Export als EINEN neuen Commit auf den oeffentlichen Verlauf setzen (ohne zu pushen)

Aufruf:
  tools/export-update.sh [Optionen] AUSGABEORDNER

Ablauf: klont den bestehenden oeffentlichen Stand (nur lesend, ohne Anmeldung) in den AUSGABEORDNER, erzeugt mit tools/export-public.sh
einen sauberen Export des gewaehlten Commits (git archive, Ausschlussliste, Ersetzungen, Verbotsscan), ersetzt im Klon den ganzen Dateibaum
durch den Export und legt daraus genau einen neuen Commit an (Autor und Committer: die noreply-Adresse). Der Verlauf des oeffentlichen
Repositorys bleibt erhalten, die Dateien, die im Export fehlen, werden im Commit geloescht. Es pusht NIE und setzt KEINEN Tag.
Gibt es gegenueber dem oeffentlichen Stand keine Aenderung, entsteht kein Commit.

Optionen:
  --commit REF                 Stand im privaten Repository, der exportiert wird (Standard: main).
  --base URL_ODER_PFAD         Oeffentlicher Stand, auf den aufgesetzt wird (Standard: das oeffentliche Repository auf GitHub).
  --message TEXT               Nachricht des neuen Commits (Standard: "VintageCoder <Version aus Cargo.toml>").
  --message-file DATEI         Nachricht aus einer Datei.
  --verify                     Wie bei export-public.sh: Bauen und Testen aus dem Export in einem frischen Ordner (dauert mehrere Minuten).
  --allow-dirty                Arbeitsbaum nicht auf Sauberkeit pruefen (nur fuer Proben).
  -h, --help                   diese Hilfe

Der AUSGABEORDNER muss leer sein oder nicht existieren; nichts wird ueberschrieben.

Beispiele:
  tools/export-update.sh /tmp/vc-update
  tools/export-update.sh --verify --message "VintageCoder 0.1.1" ~/Development/export-update
  tools/export-update.sh --base /tmp/vc-export /tmp/vc-update     (Probe gegen einen lokalen Stand)

Danach (nur der Projektinhaber, von Hand; das Skript tut nichts davon):
  cd AUSGABEORDNER
  git log --oneline -3 && git show --stat HEAD       (Aenderungen ansehen)
  git tag -a v0.1.1 -m "VintageCoder 0.1.1"          (Tag, falls es ein Release ist)
  git push origin main v0.1.1                        (origin ist der oeffentliche Stand; bei --base mit Pfad vorher:
                                                      git remote set-url origin https://github.com/CDrueeke/VintageCoder.git)
Rueckgabewert 0 = Commit angelegt oder nichts zu tun; sonst Funde oder Fehler.
EOF
}

commit_ref="main"; base="$PUBLIC_URL"; message=""; msg_file=""; verify=0; allow_dirty=0; out=""
while [ $# -gt 0 ]; do
  case "$1" in
    -h|--help) usage; exit 0 ;;
    --commit) commit_ref="${2:?--commit braucht einen Wert}"; shift 2 ;;
    --base) base="${2:?--base braucht einen Wert}"; shift 2 ;;
    --message) message="${2:?--message braucht einen Text}"; shift 2 ;;
    --message-file) msg_file="${2:?--message-file braucht eine Datei}"; shift 2 ;;
    --verify) verify=1; shift ;;
    --allow-dirty) allow_dirty=1; shift ;;
    -*) echo "Unbekannte Option: $1 (siehe --help)" >&2; exit 2 ;;
    *) [ -z "$out" ] || { echo "Nur ein Ausgabeordner erlaubt (siehe --help)" >&2; exit 2; }; out="$1"; shift ;;
  esac
done
[ -n "$out" ] || { echo "Ausgabeordner fehlt (siehe --help)" >&2; exit 2; }
[ -z "$msg_file" ] || [ -f "$msg_file" ] || { echo "Nachrichtendatei fehlt: $msg_file" >&2; exit 2; }
[ -z "$message" ] || [ -z "$msg_file" ] || { echo "--message und --message-file schliessen sich aus" >&2; exit 2; }

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
fail() { echo "FEHLER: $*" >&2; exit 1; }

if [ -e "$out" ]; then
  [ -d "$out" ] || fail "$out ist kein Ordner"
  [ -z "$(ls -A "$out")" ] || fail "Ausgabeordner $out ist nicht leer (nichts wird ueberschrieben)"
fi
mkdir -p "$out"
out="$(cd "$out" && pwd)"
case "$out" in "$root"|"$root"/*) fail "Der Ausgabeordner darf nicht im Repository liegen" ;; esac

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# 1. Oeffentlichen Stand holen (ohne Tags, ohne Anmeldung noetig)
echo "Hole den oeffentlichen Stand von $base"
git -c gc.auto=0 clone -q --no-tags "$base" "$out" || fail "Klonen von $base fehlgeschlagen"
[ "$(git -C "$out" rev-parse --abbrev-ref HEAD)" = "main" ] || fail "Der oeffentliche Stand hat keinen Branch main"
prev_head="$(git -C "$out" rev-parse HEAD)"
prev_count="$(git -C "$out" rev-list --count HEAD)"
echo "Oeffentlicher Stand: $(git -C "$out" rev-parse --short HEAD), $prev_count Commit(s)"

# 2. Sauberen Export des privaten Standes erzeugen (mit Verbotsscan)
args=(--commit "$commit_ref")
[ "$verify" -eq 0 ] || args+=(--verify)
[ "$allow_dirty" -eq 0 ] || args+=(--allow-dirty)
"$root/tools/export-public.sh" "${args[@]}" "$tmp/export" > "$tmp/export.log" 2>&1 || { cat "$tmp/export.log" >&2; fail "Export fehlgeschlagen (siehe oben)"; }
grep -E "^(Export von|Export fertig|Pruefung)" "$tmp/export.log" || true
version="$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$tmp/export/Cargo.toml" | head -1)"

# 3. Dateibaum des Klons durch den Export ersetzen (geloeschte Dateien verschwinden, Dateirechte bleiben)
git -C "$out" rm -r -q -f . > /dev/null
tar -C "$tmp/export" --exclude=./.git -cf - . | tar -C "$out" -xf -
git -C "$out" add -A
if git -C "$out" diff --cached --quiet; then
  echo "Keine Aenderungen gegenueber dem oeffentlichen Stand: kein Commit angelegt. Ordner: $out"
  exit 0
fi

# 4. Neuer Commit: Autor = Committer = noreply-Adresse, nur fuer diesen Commit gesetzt
if [ -n "$msg_file" ]; then msgpath="$msg_file"; else msgpath="$tmp/msg"; printf '%s\n' "${message:-VintageCoder ${version:-?}}" > "$msgpath"; fi
GIT_AUTHOR_NAME="$AUTHOR_NAME" GIT_AUTHOR_EMAIL="$AUTHOR_EMAIL" GIT_COMMITTER_NAME="$AUTHOR_NAME" GIT_COMMITTER_EMAIL="$AUTHOR_EMAIL" \
  git -C "$out" -c commit.gpgsign=false -c core.hooksPath=/dev/null -c gc.auto=0 -c maintenance.auto=false commit -q -F "$msgpath"

# 5. Kontrolle
[ "$(git -C "$out" rev-list --count HEAD)" = "$((prev_count + 1))" ] || fail "Es ist nicht genau ein neuer Commit"
[ "$(git -C "$out" rev-parse HEAD^)" = "$prev_head" ] || fail "Der neue Commit sitzt nicht auf dem oeffentlichen Stand"
who="$(git -C "$out" log -1 --format='%an <%ae> | %cn <%ce>')"
[ "$who" = "$AUTHOR_NAME <$AUTHOR_EMAIL> | $AUTHOR_NAME <$AUTHOR_EMAIL>" ] || fail "Autor/Committer falsch: $who"
[ -z "$(git -C "$out" tag)" ] || fail "Es gibt lokale Tags"
[ "$(git -C "$out" branch --format='%(refname:short)')" = "main" ] || fail "Es gibt weitere Branches"
git -C "$out" log -1 --format=%B | grep -qi 'co-authored' && fail "Co-Authored-Zeile in der Nachricht"
[ -z "$(git -C "$out" status --short)" ] || fail "Arbeitsbaum nicht sauber"
python3 -I "$root/tools/export_check.py" scan "$out" > "$tmp/scan.log" 2>&1 || { cat "$tmp/scan.log" >&2; fail "Verbotsscan des fertigen Stands hat Funde"; }

# 6. Zusammenfassung
changes="$(git -C "$out" diff --name-status "$prev_head" HEAD | cut -c1 | sort | uniq -c | tr '\n' ' ')"
echo
echo "Neuer Commit $(git -C "$out" rev-parse --short HEAD) auf $(git -C "$out" rev-parse --short "$prev_head"): $(git -C "$out" show -s --format=%s HEAD)"
echo "Dateien (Art: Anzahl): ${changes:-keine}   [A neu, M geaendert, D geloescht]"
git -C "$out" show --stat --format= HEAD | tail -1
echo "Funde: 0. Ordner: $out"
echo "Nichts gepusht, kein Tag gesetzt. Naechste Schritte stehen in: tools/export-update.sh --help"
