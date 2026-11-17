# SPDX-License-Identifier: GPL-3.0-or-later
"""Prüft die Lizenzvermerke der neuen Oberfläche: jede direkte Abhängigkeit von vc-gui, vc-gui-ui und vc-gui-editor steht in THIRD-PARTY-NOTICES.md,
der erzeugte Abschnitt ist aktuell, und alle Lizenzen sind erlaubende, mit der GPL-3.0 vereinbare Lizenzen."""
import re, unittest, sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gui_notices as g

# Lizenzkennungen, die für die Weitergabe unter der GPL-3.0-or-later unbedenklich sind (SPDX-Teile eines Ausdrucks)
ERLAUBT = {"MIT", "Apache-2.0", "Zlib", "Unicode-3.0", "Unlicense", "BSD-3-Clause", "BSD-2-Clause", "BSL-1.0", "0BSD", "ISC", "OFL-1.1", "CC0-1.0", "LGPL-2.1-or-later", "GPL-2.0-only", "MPL-2.0", "LLVM-exception", "Unicode-DFS-2016", "BSD-1-Clause", "Apache-2.0 WITH LLVM-exception"}

class GuiNotices(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fremd, cls.direkt = g.pakete(g.metadata()); cls.text = g.DOC.read_text(encoding="utf-8")

    def test_jede_direkte_abhaengigkeit_steht_im_dokument(self):
        fehlt = [n for n in self.direkt if not re.search(r"\| `" + re.escape(n) + r"` \|", self.text)]
        self.assertEqual(fehlt, [], f"nicht in THIRD-PARTY-NOTICES.md: {fehlt} (python3 tools/gui_notices.py)")

    def test_abschnitt_ist_aktuell(self):
        self.assertEqual(g.aktualisieren(self.text, g.abschnitt(self.fremd, self.direkt)), self.text, "Abschnitt veraltet: python3 tools/gui_notices.py")

    def test_alle_lizenzen_sind_erlaubend_und_bekannt(self):
        unbekannt = {}
        for p in self.fremd:
            lic = p.get("license") or ""
            self.assertTrue(lic, f"{p['name']} {p['version']}: keine Lizenzangabe")
            for tok in re.split(r"\s+(?:OR|AND|WITH)\s+|[()/]", lic):
                tok = tok.strip()
                if tok and tok not in ERLAUBT: unbekannt.setdefault(tok, []).append(p["name"])
        self.assertEqual(unbekannt, {}, f"nicht erlaubte oder unbekannte Lizenzen: {unbekannt}")

    def test_die_gui_crates_gehoeren_zum_arbeitsbereich_der_spike_nicht(self):
        md = g.metadata(); namen = {p["name"] for p in md["packages"] if p["id"] in set(md["workspace_members"])}
        self.assertTrue(set(g.GUI) <= namen); self.assertNotIn("spike-iced", namen)

if __name__ == "__main__": unittest.main()
