# SPDX-License-Identifier: GPL-3.0-or-later
"""Handbuch und Hilfe nennen nur Tastenkürzel, die es in der Befehlstabelle gibt, und die Kürzeltabellen enthalten alle Kürzel der Befehlstabelle."""
import pathlib, subprocess, sys, unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent


class Kuerzel(unittest.TestCase):
    def test_handbuch_und_hilfe_stimmen_mit_der_befehlstabelle(self):
        r = subprocess.run([sys.executable, str(ROOT / "tools" / "check_kuerzel.py")], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stdout)

    def test_tabelle_listet_die_kuerzel(self):
        r = subprocess.run([sys.executable, str(ROOT / "tools" / "check_kuerzel.py"), "--tabelle"], capture_output=True, text=True)
        self.assertIn("Cmd+Alt+N", r.stdout)
        self.assertIn("Cmd+Umschalt+O", r.stdout)

    def test_menuewege_kennen_untermenues(self):
        sys.path.insert(0, str(ROOT / "tools")); import check_kuerzel as c
        menues, pfade = c.menuepfade()
        self.assertEqual(menues, ["Datei", "Bearbeiten", "Ansicht", "Projekt", "Hilfe"])
        for p in [("Bearbeiten", "Markdown", "Fett"), ("Bearbeiten", "Markdown"), ("Projekt", "Debuggen", "Weiter"), ("Ansicht", "Bereiche", "Layout zurücksetzen"), ("Projekt", "Projekt bauen")]:
            self.assertIn(p, pfade)
        self.assertNotIn(("Bearbeiten", "Fett"), pfade)


if __name__ == "__main__":
    unittest.main()
