# SPDX-License-Identifier: GPL-3.0-or-later
# Hilfe: erzeugte Seiten (Emulatoren, Handbuchkopf, Befehle, Register, Nachschlagen)
help-emu-arch-64bit = 64 Bit
help-emu-portable-download = portabel ladbar: { $pakete }
help-emu-col-emulator = Emulator
help-emu-col-systems = Systeme
help-emu-col-role = Rolle
help-emu-col-portable = Portabel installierbar
help-emu-homepage = Homepage
help-emu-download-page = Download-Seite
help-emu-license = Lizenz
help-emu-role-in-vc = Rolle in VintageCoder
help-chapter-meta = Stufe: { $stufe } · Zeit: { $zeit }
help-chapter-see-also = Siehe auch im Handbuch:
help-chapter-reference = Zum Nachschlagen:
help-mn-quad = Q-Befehl der 45GS02: Präfix `NEG NEG` (`$42 $42`) vor dem Opcode von `{ $base }`; wirkt auf das 32-Bit-Register Q (A, X, Y, Z). Mit `[$nn],z` zusätzlich Präfix `$EA` (32-Bit-Zeiger).
help-mn-izz = Mit `[$nn],z` (Präfix `$EA`) wird der Zeiger 32 Bit breit.
help-mn-table-head = | Adressierung | Schreibweise | Opcode | Bytes | Zyklen | Flags |
help-mn-illegal = Einige Formen sind **undokumentiert** (`illegals = allow|warn` im `.cpu`-Kopf nötig).
help-mn-cycles-flags = Zyklenangaben: `+1` bei Seitenüberschreitung, `+1/+2` bei genommenem Sprung. Flags: N=Negativ, V=Überlauf, Z=Null, C=Übertrag.
help-see-basic = Siehe [BASIC](help:basic).
help-see-topic = Siehe [{ $title }](help:{ $rel }).
help-reg-title = Register { $name }
help-reg-line = **{ $chip }.{ $name }** – Adresse `{ $addr }`, Zugriff { $access }
help-reg-use = Im Quelltext als `chip.register` verwendbar (z. B. `lda vic2.border`).
help-none = Zu „{ $word }“ habe ich keinen Eintrag. Die Suche findet Themen, Befehle, Direktiven und Register.
help-sym-address = Adresse `{ $addr }`
help-sym-same-address = Gleiche Adresse: { $names }
help-sym-include = Einbinden: `#include <{ $file }>`
