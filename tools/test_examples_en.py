#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Tests für tools/examples_en.py (englischer Beispielspiegel, i18n I6). Nur Standardbibliothek."""
import os, sys, unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import examples_en as ex  # noqa: E402


class ZerlegungTests(unittest.TestCase):
    def test_assembler_kommentar_ausserhalb_von_anfuehrungszeichen(self):
        self.assertEqual(ex.split_comment('lda #1 ; Zaehler'), ("lda #1 ", "; Zaehler"))
        self.assertEqual(ex.split_comment('.byte "a;b" // text'), ('.byte "a;b" ', "// text"))
        self.assertEqual(ex.split_comment("lda #1"), ("lda #1", ""))

    def test_zeichenketten_werden_maskiert(self):
        self.assertEqual(ex.mask_strings('.byte "HALLO", 0'), '.byte "", 0')

    def test_basic_kommentar_rem_und_semikolon(self):
        self.assertEqual(ex.bas_split("; Titel"), ("", "; Titel"))
        self.assertEqual(ex.bas_split("REM Titel"), ("", "REM Titel"))
        self.assertEqual(ex.bas_split('poke 1,2 : rem stern')[1], "rem stern")
        self.assertEqual(ex.bas_split('print "rem"'), ('print "rem"', ""))
        self.assertEqual(ex.bas_split("remainder = 1"), ("remainder = 1", ""))

    def test_deutschen_text_erkennen(self):
        self.assertTrue(ex.deutsch("Rahmenfarbe ändern"))
        self.assertTrue(ex.deutsch("das ist ein Test und mehr"))
        self.assertFalse(ex.deutsch("Change the border color"))


class RepoTests(unittest.TestCase):
    def test_spiegel_im_repository_ist_in_ordnung(self):
        self.assertEqual(ex.funde(), [])


if __name__ == "__main__":
    unittest.main()
