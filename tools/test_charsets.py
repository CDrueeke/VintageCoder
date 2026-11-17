# SPDX-License-Identifier: CC0-1.0
"""Prüft die eigenen Standardzeichensätze unter data/charsets/ (Konzept „Eigene Standardzeichensätze“, Abschnitt 5, Etappen E0 bis E5):
Format, Größe, Zeichenanzahl, Lizenzkopf CC0, Herkunftsblock, Prüfstempel, Index, Hinweise zu Dritten, Invertierung, Eindeutigkeit, Ausnahmeliste,
kein Original (Gesamt-Prüfsummen), Erzeugnis aktuell."""
import hashlib, json, os, pathlib, re, subprocess, sys, unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent
CH = ROOT / "data" / "charsets"
sys.path.insert(0, str(ROOT / "tools"))
import gen_vc_standard as gen  # noqa: E402
import charset_distance as cdist  # noqa: E402

PFLICHT = ["id", "rev", "source", "author", "created", "tool", "style", "license", "check"]
ERLAUBT = {"CC0-1.0"}


def profile(pid):
    d = json.loads((ROOT / "data" / "profile" / "profiles.json").read_text(encoding="utf-8"))
    ps = {p["id"]: p for p in d["profiles"]}
    p = dict(ps[pid])
    q = p
    while "extends" in q:
        q = ps[q["extends"]]
        for k, v in q.items():
            p.setdefault(k, v)
    return p


class Charsets(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.index = json.loads((CH / "index.json").read_text(encoding="utf-8"))["saetze"]
        cls.ausn = json.loads((CH / "ausnahmen.json").read_text(encoding="utf-8"))["saetze"]
        cls.sets = {e["id"]: json.loads((CH / e["datei"]).read_text(encoding="utf-8")) for e in cls.index}
        cls.notices = (ROOT / "THIRD-PARTY-NOTICES.md").read_text(encoding="utf-8")

    def test_dateien_und_index_stimmen_ueberein(self):
        auf_platte = sorted(str(p.relative_to(CH)) for p in CH.rglob("*.vcchar"))
        self.assertEqual(auf_platte, sorted(e["datei"] for e in self.index))
        self.assertEqual(len({e["id"] for e in self.index}), len(self.index), "doppelte id")
        self.assertEqual(sorted(self.ausn), sorted(self.sets))

    def test_format_groesse_zeichenanzahl(self):
        for e in self.index:
            d = self.sets[e["id"]]
            p = profile(d["profile"])
            self.assertEqual((d["format"], d["kind"]), (1, "charset"), e["id"])
            self.assertEqual(d["name"], e["id"])
            self.assertEqual(len(d["frames"]), p["count"], f"{e['id']}: Zeichenanzahl")
            self.assertEqual(e["zeichen"], p["count"])
            self.assertEqual(e["profil"], d["profile"])
            digits = p.get("digits", 1)
            for i, f in enumerate(d["frames"]):
                self.assertEqual(len(f["rows"]), p["height"], f"{e['id']} Zeichen {i}: Zeilen")
                for r in f["rows"]:
                    self.assertEqual(len(r), p["width"] * digits, f"{e['id']} Zeichen {i}: Breite")
                    if digits == 1:
                        self.assertRegex(r, r"^[01]+$")
                    else:
                        self.assertRegex(r, r"^(0[01])+$")

    def test_herkunft_lizenz_pflichtfelder(self):
        for e in self.index:
            o = self.sets[e["id"]]["origin"]
            for k in PFLICHT:
                self.assertTrue(o.get(k) not in (None, ""), f"{e['id']}: origin.{k} fehlt")
            self.assertIn(o["license"], ERLAUBT)
            self.assertEqual(e["lizenz"], o["license"])
            self.assertEqual(o["id"], e["id"])
            self.assertEqual(o["source"], "vintagecoder-standard")
            self.assertEqual(o["style"], "STIL.md@1")
            self.assertIn(o["check"]["result"], {"offen", "bestanden"})
            if o["check"]["result"] == "bestanden":
                self.assertTrue(o["check"]["date"] and o["check"]["min_distance"] is not None)
            self.assertIn(o.get("freigabe"), {"offen", "erteilt"})
            if o["freigabe"] == "erteilt":
                self.assertTrue(o.get("freigabe_datum") and o.get("freigabe_vermerk"), f"{e['id']}: Freigabe braucht Datum und Vermerk")
                # Ehrlichkeit der Daten: ohne Stempel „bestanden“ muss der Vermerk sagen, dass die Abstandsprüfung nicht bestanden ist
                if o["check"]["result"] != "bestanden":
                    self.assertIn("nicht bestanden", o["freigabe_vermerk"], e["id"])

    def test_lizenzkopf_cc0(self):
        lic = (CH / "LICENSE.txt").read_text(encoding="utf-8")
        self.assertTrue(lic.startswith("SPDX-License-Identifier: CC0-1.0"))
        for f in ("tools/gen_vc_standard.py", "tools/charset_distance.py", "tools/test_charsets.py"):
            kopf = (ROOT / f).read_text(encoding="utf-8").split("\n", 2)[:2]
            self.assertTrue(any("SPDX-License-Identifier: CC0-1.0" in z for z in kopf), f)
        stil = (CH / "STIL.md").read_text(encoding="utf-8")
        self.assertIn("CC0-1.0", stil)

    def test_pruefstempel_hash_stimmt(self):
        for s in gen.build_sets():
            o = self.sets[s["id"]]["origin"]
            self.assertEqual(o["check"]["set_sha256"], gen.set_hash(s["chars"]), f"{s['id']}: Prüfstempel veraltet")
            self.assertEqual(o["check"]["set_sha256"], next(e for e in self.index if e["id"] == s["id"])["satz_sha256"])

    def test_hinweise_zu_dritten_nennen_jede_id(self):
        a = self.notices.index("Mitgelieferte Zeichensätze (Eigenwerke)")
        abschnitt = self.notices[a:a + 6000]
        for e in self.index:
            self.assertIn(e["id"], abschnitt, f"{e['id']} fehlt in THIRD-PARTY-NOTICES.md")
        self.assertIn("keine Original-ROMs", abschnitt)

    def test_invertierte_haelfte(self):
        mit = {"c64", "c128", "vic20", "plus4", "mega65", "pet"}
        for e in self.index:
            if e["system"] not in mit:
                continue
            fr = self.sets[e["id"]]["frames"]
            self.assertEqual(len(fr), 256)
            for n in range(128):
                a, b = fr[n]["rows"], fr[128 + n]["rows"]
                if e["system"] == "mega65":
                    a = ["".join("1" if c == "01" else "0" for c in (r[i:i + 2] for i in range(0, len(r), 2))) for r in a]
                    b = ["".join("1" if c == "01" else "0" for c in (r[i:i + 2] for i in range(0, len(r), 2))) for r in b]
                neg = ["".join("1" if c == "0" else "0" for c in r) for r in a]
                if e["profil"] == "c128.char.vdc":
                    pass
                self.assertEqual(b, neg, f"{e['id']}: Code {128 + n} ist nicht die Negation von {n}")

    def test_atari_international_aendert_nur_die_belegten_codes(self):
        """Quellen (Wikipedia ATASCII, Altirra Kapitel 9): ersetzt werden die internen Codes 64 bis 90 sowie 96 und 123; 91 bis 95 (Esc, Pfeile) bleiben."""
        a = self.sets["vc-atari-standard"]["frames"]
        b = self.sets["vc-atari-international"]["frames"]
        self.assertEqual(len(a), 128)
        diff = [i for i in range(128) if a[i] != b[i]]
        self.assertEqual(diff, list(range(64, 91)) + [96, 123])

    def test_belegung_ist_quellengepruft_dokumentiert(self):
        for e in self.index:
            o = self.sets[e["id"]]["origin"]
            bel = o["belegung"]
            self.assertIn(bel["status"], ("quellengeprueft", "vorlaeufig"), e["id"])
            self.assertEqual(e["belegung"], bel["status"], e["id"])
            self.assertEqual(bel["datum"], e["belegung_datum"])
            self.assertRegex(bel["datum"], r"^\d{4}-\d{2}-\d{2}$")
            if bel["status"] == "quellengeprueft":
                self.assertTrue(bel["geprueft_gegen"], f"{e['id']}: quellengeprüft ohne Quellenangabe")
            self.assertNotIn("vorläufig", e["name"], e["id"])

    def test_pet_unterscheidet_sich_vom_c64_nur_im_code_28(self):
        c = lambda i: [f["rows"] for f in self.sets[i]["frames"]]
        for pet, c64 in (("vc-pet-upper", "vc-c64-upper"), ("vc-pet-lower", "vc-c64-lower")):
            diff = [i for i, (x, y) in enumerate(zip(c(pet), c(c64))) if x != y]
            self.assertEqual(diff, [28, 156], pet)

    def test_ableitungen_sind_wirklich_abgeleitet(self):
        c = lambda i: [f["rows"] for f in self.sets[i]["frames"]]
        for i in ("vc-plus4-standard", "vc-vic20-upper", "vc-c128-upper"):
            self.assertEqual(c(i), c("vc-c64-upper"), i)
        for i in ("vc-vic20-lower", "vc-c128-lower", "vc-plus4-lower"):
            self.assertEqual(c(i), c("vc-c64-lower"), i)
        for k, basis in (("vc-c128-vdc-upper", "vc-c64-upper"), ("vc-c128-vdc-lower", "vc-c64-lower")):
            self.assertEqual(c(k), [[r for r in z for _ in (0, 1)] for z in c(basis)], k)
        for k, basis in (("vc-mega65-upper", "vc-c64-upper"), ("vc-mega65-lower", "vc-c64-lower")):
            self.assertEqual(c(k), [[("".join("01" if x == "1" else "00" for x in r)) for r in z] for z in c(basis)], k)

    def test_ausnahmeliste_passt_zu_den_bildern(self):
        for s in gen.build_sets():
            ex = self.ausn[s["id"]]
            for code, kl in ex.items():
                ch = s["bild"][int(code)]
                self.assertEqual(gen.forced_kind(ch), kl, f"{s['id']} Code {code}")
                if kl == "geometrisch":
                    self.assertTrue(gen.grafikcode(s["system"], s["satz"], int(code)), f"{s['id']} Code {code}: geometrisch nur bei Grafikzeichen")
            self.assertEqual(self.sets[s["id"]]["origin"]["check"]["exceptions"], len(ex))
            self.assertIn("0" if s["system"] == "atari8" else "32", ex, f"{s['id']}: Leerzeichen muss Ausnahme sein")

    def test_geometrisch_erzwungene_klasse(self):
        """Linien, Rahmen, Ecken, T-Stücke, Kreuze, Dreiecke und Diagonalen sind eine eigene Ausnahmeklasse (auch als Negativ); Buchstaben und Symbole nie."""
        ex = self.ausn["vc-c64-upper"]
        for code in (66, 73, 77, 78, 85, 86, 91, 93, 95, 105, 107, 109, 110, 112, 113, 114, 115, 125, 194, 205, 221):
            self.assertIn(str(code), ex, f"Code {code}")
        self.assertEqual(ex["91"], "geometrisch")
        for code in list(range(1, 27)) + list(range(33, 64)):
            self.assertNotEqual(ex.get(str(code)), "geometrisch", f"Code {code} ist ein Buchstabe, Symbol oder eine Ziffer")
        d = json.loads((CH / "ausnahmen.json").read_text(encoding="utf-8"))
        self.assertIn("geometrisch", d["klassen"])
        self.assertIn("Nachbarzellen", d["klassen"]["geometrisch"])

    def test_abstandswerkzeug_klassenweise_schwelle_und_codes(self):
        idx = {e["id"]: e for e in self.index}
        c64, lo, at = idx["vc-c64-upper"], idx["vc-c64-lower"], idx["vc-atari-standard"]
        self.assertEqual([cdist.klasse(c64, c) for c in (9, 36, 30, 27, 159, 150, 190, 70)], ["Buchstabe", "Symbol/Ziffer", "Symbol/Ziffer", "Symbol/Ziffer", "Symbol/Ziffer", "Buchstabe", "Symbol/Ziffer", "übrig"])
        self.assertEqual([cdist.klasse(lo, c) for c in (73, 90, 91)], ["Buchstabe", "Buchstabe", "übrig"], "Versalien im Satz Klein/Groß liegen an 65 bis 90")
        self.assertEqual([cdist.klasse(at, c) for c in (4, 41, 59, 70, 100, 92, 126)], ["Symbol/Ziffer", "Buchstabe", "Symbol/Ziffer", "übrig", "Buchstabe", "Symbol/Ziffer", "Symbol/Ziffer"])
        ai = idx["vc-atari-international"]
        self.assertEqual([cdist.klasse(ai, c) for c in (65, 72, 96, 123, 70)], ["Buchstabe", "Symbol/Ziffer", "Symbol/Ziffer", "Buchstabe", "Buchstabe"])
        # Selbstvergleich: der eigene Satz als „Fremddatei“ liefert Abstand 0; alle Codes werden mit Klasse genannt, nie Bilder
        z = cdist.eigene_zeichen(c64)
        k, ex, v, zu_nah = cdist.vergleich("vc-c64-upper", z, z, self.ausn["vc-c64-upper"], c64)
        self.assertEqual((k, ex > 0), (0, True))
        self.assertTrue(set(zu_nah) <= set(cdist.KLASSEN))
        codes = {c for lst in zu_nah.values() for c, _ in lst}
        self.assertTrue(all(str(c) not in self.ausn["vc-c64-upper"] for c in codes), "Ausnahmen werden nicht gemeldet")
        self.assertTrue(all(isinstance(c, int) and isinstance(d, int) for lst in zu_nah.values() for c, d in lst))
        # eine Schwelle von 0 meldet nichts
        self.assertEqual(cdist.vergleich("x", z, z, self.ausn["vc-c64-upper"], c64, 0, 0)[3], {})
        # klassenweise: ein Vergleich mit Abstand 3 besteht bei Buchstaben (Schwelle 3), nicht bei Grafik (Schwelle 6)
        self.assertEqual(cdist.SCHWELLE_ZEICHEN, 3)
        self.assertEqual(cdist.SCHWELLE, 6)

    def test_innere_eindeutigkeit(self):
        for s in gen.build_sets():
            ex = self.ausn[s["id"]]
            seen = {}
            for code, ch in enumerate(s["bild"]):
                key = tuple(ch)
                if key in seen and str(code) not in ex:
                    self.fail(f"{s['id']}: Code {code} sieht aus wie Code {seen[key]}")
                seen.setdefault(key, code)

    def test_buchstaben_bleiben_im_raster(self):
        """Stilblatt: Textzeichen nutzen nur die Spalten 1 bis 5 und die Zeilen 0 bis 7; Versalien haben Höhe 6."""
        for k, (top, rows) in gen.ALLTEXT.items():
            self.assertTrue(all(len(r) == 5 for r in rows), k)
            self.assertLessEqual(top + len(rows), 8, k)
        for k, (top, rows) in gen.CAPS.items():
            self.assertEqual((top, len(rows)), (0, 6), k)

    def test_kein_original(self):
        """Ebene C: kein Satz (ganz oder in Hälften) stimmt mit einer Gesamtprüfsumme aus known.json überein; keine Datei heißt chargen*."""
        known = json.loads((ROOT / "data" / "roms" / "known.json").read_text(encoding="utf-8"))["roms"]
        sha1s = {r["sha1"].lower() for r in known}
        crcs = {(r["size"], r["crc32"].upper()) for r in known}
        for p in CH.rglob("*"):
            self.assertFalse(p.name.lower().startswith("chargen"), p)
        for s in gen.build_sets():
            if s["profile"] in ("mega65.char.fcm", "c128.char.vdc"):
                raw_chars = s["bild"] if s["profile"] == "mega65.char.fcm" else [r[::2] for r in s["chars"]]
            else:
                raw_chars = s["chars"]
            roh = bytes(int(r, 2) for c in raw_chars for r in c)
            for teil in (roh, roh[:len(roh) // 2], roh[len(roh) // 2:]):
                self.assertNotIn(hashlib.sha1(teil).hexdigest(), sha1s, s["id"])
                self.assertNotIn((len(teil), "%08X" % (zlib_crc(teil))), crcs, s["id"])

    def test_erzeugnis_ist_aktuell(self):
        r = subprocess.run([sys.executable, str(ROOT / "tools" / "gen_vc_standard.py"), "--check"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, "data/charsets/ veraltet: python3 tools/gen_vc_standard.py\n" + r.stdout)

    def test_verbotene_schreibweisen_fehlen(self):
        for p in list(CH.rglob("*")) + [ROOT / "tools" / "gen_vc_standard.py"]:
            if p.is_file():
                t = p.read_text(encoding="utf-8").lower()
                self.assertNotIn("".join(["kil", "ler"]), t, p)


def zlib_crc(b):
    import zlib
    return zlib.crc32(b) & 0xFFFFFFFF


if __name__ == "__main__":
    unittest.main()
