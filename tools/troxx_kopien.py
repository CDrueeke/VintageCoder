#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
# troxx_kopien.py - gleicht die Kopien der gemeinsamen Troxx-Logik mit der Referenzfassung (VC-20) ab.
#
# Hintergrund: Jedes Troxx-Projekt (examples/<system>/troxx) ist in sich geschlossen; die systemunabhaengige Logik (Nummerierung, Zaehler,
# Zufall, BCD-Punkte, KI-Entscheidung, Tabellen) wird in der Referenzfassung (examples/vic20/troxx) gepflegt und je Projekt kopiert.  Dieses
# Werkzeug vergleicht die Kopien mit dem Original, damit niemand sie unbemerkt auseinanderlaufen laesst.
#
# Das Projekt fuehrt eine Liste KOPIEN.txt (im Projektordner).  Aufbau, eine Zeile je Routine oder Tabelle (Kommentare mit #):
#     Referenz: examples/vic20/troxx   Stand: 4697d074              (Kopfzeile)
#     <Datei> <Name> kopie                                          Routine ist eine Kopie: muss gleich sein
#     <Datei> <Name> angepasst <Pruefsumme>                         Routine wurde angepasst; die Pruefsumme ist die der Referenz zum Stand
#     <Datei> <Name> kopie <Referenzdatei> [<Referenzname>]         (optional) Referenz liegt in anderer Datei oder anderem Namen
# Verglichen wird der Text von der Marke "<Name>:" bis zur naechsten Marke in Spalte 1 (Namen ohne Punkt) oder dem Ende des Bereichs; vorher werden
# Kommentare (ab ";"), Leerzeilen und ueberzaehlige Leerzeichen entfernt.  Pruefsumme = SHA-256 des so bereinigten Textes (erste 12 Stellen).
#
# Ergebnis je Eintrag:  gleich | ABWEICHUNG (Kopie ungleich Original) | angepasst (Referenz unveraendert) | REFERENZ GEAENDERT (Anpassung pruefen).
# Endestatus 0, wenn keine Abweichung und keine Referenzaenderung vorliegt, sonst 1.
#
# Aufruf (im Repository-Ordner):
#     python3 tools/troxx_kopien.py                      alle Projekte examples/*/troxx mit KOPIEN.txt gegen examples/vic20/troxx
#     python3 tools/troxx_kopien.py examples/c64/troxx   nur dieses Projekt (ebenso examples/plus4/troxx, examples/atari/troxx)
#     python3 tools/troxx_kopien.py --scan examples/c64/troxx   zeigt je Datei alle Routinen, die es in der Referenz gleichen Namens gibt (Vorschlag fuer KOPIEN.txt)
#     python3 tools/troxx_kopien.py --stand examples/c64/troxx  schreibt die Pruefsummen der Referenz in die "angepasst"-Zeilen (nach geprueften Aenderungen)
import hashlib
import os
import re
import sys

TRAIL = ('.assert', '.if', '.ifdef', '.ifndef', '.else', '.const', '.var', '.macro', '.encoding')
LABEL = re.compile(r'^([A-Za-z_][A-Za-z0-9_]*):')


def clean(line):
    line = line.split(';', 1)[0]
    return ' '.join(line.split())


def routines(path):
    """Liefert {Name: bereinigter Text} aller Marken in Spalte 1 der Datei."""
    out = {}
    cur = None
    buf = []
    if not os.path.exists(path):
        return out
    for raw in open(path, encoding='utf-8'):
        line = raw.rstrip('\n')
        m = LABEL.match(line)
        if m:
            if cur is not None:
                out[cur] = '\n'.join(buf)
            cur = m.group(1)
            buf = []
            line = line[m.end():]
        elif re.match(r'^\}', line) or re.match(r'^\.endmacro', line):
            if cur is not None:
                out[cur] = '\n'.join(buf)
            cur = None
            buf = []
            continue
        elif re.match(r'^[A-Za-z_][A-Za-z0-9_]*\s*:\s*\{', line) or re.match(r'^\.macro', line):
            if cur is not None:
                out[cur] = '\n'.join(buf)
            cur = None
            buf = []
            continue
        if cur is not None:
            c = clean(line)
            if c:
                buf.append(c)
    if cur is not None:
        out[cur] = '\n'.join(buf)
    # Anweisungen, die hinter der letzten Zeile der Routine stehen, aber schon zur naechsten Marke gehoeren, abschneiden
    for k in out:
        lines = out[k].split('\n')
        while lines and lines[-1].startswith(TRAIL):
            lines.pop()
        out[k] = '\n'.join(lines)
    return out


def digest(text):
    return hashlib.sha256(text.encode('utf-8')).hexdigest()[:12]


def read_list(project):
    path = os.path.join(project, 'KOPIEN.txt')
    ref = None
    stand = ''
    rows = []
    for n, raw in enumerate(open(path, encoding='utf-8'), 1):
        line = raw.split('#', 1)[0].strip()
        if not line:
            continue
        m = re.match(r'^Referenz:\s*(\S+)\s+Stand:\s*(\S+)', line)
        if m:
            ref, stand = m.group(1), m.group(2)
            continue
        f = line.split()
        if len(f) < 3 or f[2] not in ('kopie', 'angepasst'):
            print('%s:%d: Zeile nicht verstanden: %s' % (path, n, line))
            sys.exit(2)
        rows.append(f)
    return ref, stand, rows


def run(project, refdir_default, stand_schreiben=False):
    ref, stand, rows = read_list(project)
    refdir = ref or refdir_default
    cache = {}

    def get(d, f):
        k = (d, f)
        if k not in cache:
            cache[k] = routines(os.path.join(d, f))
        return cache[k]

    print('%s  (Referenz %s, Stand %s)' % (project, refdir, stand))
    gleich = angepasst = bad = 0
    files_seen = set()
    new_rows = []
    for f in rows:
        datei, name, art = f[0], f[1], f[2]
        rest = f[3:]
        rdatei, rname = datei, name
        stored = None
        if art == 'angepasst' and rest and re.fullmatch(r'[0-9a-f]{12}', rest[0]):
            stored = rest[0]
            rest = rest[1:]
        if rest:
            rdatei = rest[0]
        if len(rest) > 1:
            rname = rest[1]
        files_seen.add(datei)
        mine = get(project, datei).get(name)
        orig = get(refdir, rdatei).get(rname)
        if orig is None:
            print('  %-12s %-16s FEHLT in der Referenz (%s: %s)' % (datei, name, rdatei, rname))
            bad += 1
            new_rows.append(f)
            continue
        if mine is None:
            print('  %-12s %-16s FEHLT im Projekt' % (datei, name))
            bad += 1
            new_rows.append(f)
            continue
        if art == 'kopie':
            if mine == orig:
                print('  %-12s %-16s gleich            [%s]' % (datei, name, digest(mine)))
                gleich += 1
            else:
                print('  %-12s %-16s ABWEICHUNG        Projekt %s, Referenz %s' % (datei, name, digest(mine), digest(orig)))
                bad += 1
            new_rows.append(f)
        else:
            d = digest(orig)
            if stand_schreiben:
                new_rows.append(f[:3] + [d] + rest)
                stored = d
            else:
                new_rows.append(f)
            if stored is None or stored == d:
                print('  %-12s %-16s angepasst         [Referenz %s unveraendert]' % (datei, name, d))
                angepasst += 1
            else:
                print('  %-12s %-16s REFERENZ GEAENDERT (war %s, jetzt %s): Anpassung pruefen' % (datei, name, stored, d))
                bad += 1
    for datei in sorted(files_seen):
        p = os.path.join(project, datei)
        head = ''.join(open(p, encoding='utf-8').readlines()[:40])
        if 'Herkunft:' not in head:
            print('  %-12s Kopfkommentar "Herkunft:" fehlt' % datei)
            bad += 1
    print('  Ergebnis: %d gleich, %d angepasst, %d Abweichungen' % (gleich, angepasst, bad))
    if stand_schreiben:
        path = os.path.join(project, 'KOPIEN.txt')
        lines = open(path, encoding='utf-8').read().split('\n')
        out = []
        it = iter(new_rows)
        for raw in lines:
            body = raw.split('#', 1)[0].strip()
            if body and not body.startswith('Referenz:'):
                out.append(' '.join(next(it)))
            else:
                out.append(raw)
        open(path, 'w', encoding='utf-8').write('\n'.join(out))
        print('  Pruefsummen der Referenz in %s eingetragen' % path)
    return bad


def scan(project, refdir):
    for f in sorted(os.listdir(project)):
        if not (f.endswith('.asm') or f.endswith('.inc')):
            continue
        mine = routines(os.path.join(project, f))
        for rf in sorted(os.listdir(refdir)):
            if not (rf.endswith('.asm') or rf.endswith('.inc')):
                continue
            orig = routines(os.path.join(refdir, rf))
            for name, txt in mine.items():
                if name in orig and not name.startswith('t_'):
                    print('%-12s %-16s %-12s %s' % (f, name, rf, 'gleich' if txt == orig[name] else 'verschieden'))


def main(argv):
    refdir = 'examples/vic20/troxx'
    args = [a for a in argv if not a.startswith('--')]
    if '--scan' in argv:
        for p in args:
            scan(p, refdir)
        return 0
    projects = args
    if not projects:
        base = 'examples'
        for sysdir in sorted(os.listdir(base)):
            p = os.path.join(base, sysdir, 'troxx')
            if os.path.exists(os.path.join(p, 'KOPIEN.txt')):
                projects.append(p)
    if not projects:
        print('Keine Troxx-Projekte mit KOPIEN.txt gefunden.')
        return 0
    bad = 0
    for p in projects:
        bad += run(p, refdir, '--stand' in argv)
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
