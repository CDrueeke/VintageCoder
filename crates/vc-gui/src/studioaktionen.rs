// SPDX-License-Identifier: GPL-3.0-or-later
//! Aktionen der Studios als Liste: dieselben Einträge speisen die Befehlspalette („Studio: …“) und das Kontextmenü der Fläche. So steht jede Studio-Aktion mit Name und
//! Kürzel an einer Stelle und ist ohne Maus erreichbar, ohne die Befehlstabelle mit Dutzenden Einträgen je Studio aufzublähen.
use super::*;

/// Eine Aktion eines Studios: Name, Kürzel (leer = keins), ob sie gerade geht, ob sie im Kontextmenü steht, Gruppe (Trenner im Menü) und die Nachricht
#[derive(Clone, Debug)]
pub struct SAktion { pub name: String, pub kuerzel: String, pub aktiv: bool, pub menue: bool, pub gruppe: u8, pub msg: Msg }

impl App {
    /// Aktionen des Studios im aktiven Reiter (leer, wenn dort kein Studio liegt)
    pub(crate) fn studio_aktionen(&self) -> Vec<SAktion> {
        let Some(rel) = self.name() else { return vec![] }; let r = rel.clone(); let mut v: Vec<SAktion> = vec![];
        let mut a = |name: &str, kuerzel: &str, aktiv: bool, menue: bool, gruppe: u8, msg: Msg| v.push(SAktion { name: name.to_string(), kuerzel: kuerzel.to_string(), aktiv, menue, gruppe, msg });
        if let Some(s) = self.studios.get(&rel) {
            use crate::studio::Aktion as A; use vc_ide_model::pixel::Tool; let d = &s.doc; let m = |x: A| Msg::StudioAkt(r.clone(), x);
            for (t, n, k) in [(Tool::Pen, "Stift", "D"), (Tool::Erase, "Radierer", "E"), (Tool::Fill, "Füllen", "F"), (Tool::Line, "Linie", "L"), (Tool::Rect, "Rahmen", "R"), (Tool::Select, "Auswahl", "S"), (Tool::Pick, "Pipette", "I")] { a(&format!("Werkzeug: {n}"), k, true, true, 0, m(A::Werkzeug(t))) }
            a("Horizontal spiegeln", "X", true, true, 1, m(A::Spiegeln("mh"))); a("Vertikal spiegeln", "Y", true, true, 1, m(A::Spiegeln("mv"))); a("Frame leeren", "", true, true, 1, m(A::Leeren));
            if !d.fixed_frames && d.kind != "charset" { a("Frame anhängen", "", true, true, 2, m(A::FrameNeu)); a("Frame duplizieren", "", true, true, 2, m(A::FrameDup)); a("Frame löschen", "", d.frames.len() > 1, true, 2, m(A::FrameLoeschen)); }
            if d.kind == "sprite" && d.frame_modes.len() > 1 { a("Modus: Hires", "C", true, false, 2, m(A::Modus(false))); a("Modus: Multicolor", "C", true, false, 2, m(A::Modus(true))); }
            a("Einpassen", "0", true, false, 3, m(A::ZoomAnpassen)); a("Vergrößern", "+", true, false, 3, m(A::ZoomPlus)); a("Verkleinern", "−", true, false, 3, m(A::ZoomMinus));
        } else if let Some(s) = self.screens.get(&rel) {
            use crate::screenstudio::SAkt as A; use vc_ide_model::screendoc::Werkzeug as W; let m = |x: A| Msg::ScreenAkt(r.clone(), x); let _ = s;
            for (t, n, k) in [(W::Zeichen, "Zeichen", "D"), (W::Text, "Text", "T"), (W::Fuellen, "Füllen", "F"), (W::Rahmen, "Rahmen", "R"), (W::Pipette, "Pipette", "I"), (W::Loeschen, "Löschen", "E")] { a(&format!("Werkzeug: {n}"), k, true, true, 0, m(A::Werkzeug(t))) }
            a("In Kacheln zerlegen …", "", true, true, 1, m(A::ZerlegenDialog(Some((2, 2))))); a("Einpassen", "0", true, false, 2, m(A::Einpassen)); a("Vergrößern", "+", true, false, 2, m(A::Zoom(1))); a("Verkleinern", "−", true, false, 2, m(A::Zoom(-1)));
        } else if let Some(s) = self.tilesets.get(&rel) {
            use crate::tilestudio::TAkt as A; let d = &s.doc; let (w, h, hat) = (d.ts.tw, d.ts.th, !d.ts.tiles.is_empty()); let m = |x: A| Msg::TileAkt(r.clone(), x);
            a("Zellen waagerecht spiegeln", "", hat && w > 1, true, 0, m(A::SpiegelH)); a("Zellen senkrecht spiegeln", "", hat && h > 1, true, 0, m(A::SpiegelV)); a("Um 90° drehen", "", hat && w == h && w > 1, true, 0, m(A::Drehen)); a("Alle Zellen leeren", "", hat, true, 0, m(A::Leeren));
            a("Zellen der Kachel kopieren", "", hat, true, 1, m(A::Kopieren)); a("Zellen einfügen", "", d.zwischen.as_ref().is_some_and(|c| c.len() == w * h), true, 1, m(A::Einfuegen));
            a("Neue Kachel", "", d.ts.tiles.len() < vc_media::tiles::MAX_TILES, true, 2, m(A::Neu)); a("Kachel duplizieren", "", hat, true, 2, m(A::Dupl)); a("Kachel löschen", "", hat, true, 2, m(A::Loeschen));
            a("Einpassen", "0", true, false, 3, m(A::Einpassen)); a("Vergrößern", "+", true, false, 3, m(A::Zoom(1))); a("Verkleinern", "−", true, false, 3, m(A::Zoom(-1)));
        } else if let Some(s) = self.banken.get(&rel) {
            use crate::bankstudio::BAkt as A; use vc_ide_model::bankdoc::Modus; let d = &s.doc; let m = |x: A| Msg::BankAkt(r.clone(), x);
            a("Modus: Kacheln", "Tab", true, true, 0, m(A::Modus(Modus::Kachel))); a("Modus: Zeichen", "Tab", true, true, 0, m(A::Modus(Modus::Zelle)));
            for (w, n) in d.werkzeuge() { a(&format!("Werkzeug: {n}"), crate::bankstudio::werkzeug_taste(*w).unwrap_or(""), true, true, 1, m(A::Werkzeug(*w))) }
            a("Neuer Screen", "", true, true, 2, m(A::ScreenNeu)); a("Screen duplizieren", "", true, true, 2, m(A::ScreenDupl)); a("Screen löschen", "", d.bank.screens.len() > 1, true, 2, m(A::ScreenLoeschen));
            a("Auswahl kopieren", "", d.auswahl.is_some(), true, 3, m(A::AuswahlKopieren)); a("Auswahl leeren", "Entf", d.auswahl.is_some(), true, 3, m(A::AuswahlLeeren));
            a("Einpassen", "0", true, false, 4, m(A::Einpassen)); a("Vergrößern", "+", true, false, 4, m(A::Zoom(1))); a("Verkleinern", "−", true, false, 4, m(A::Zoom(-1)));
        } else if self.paletten.contains_key(&rel) {
            use crate::palettestudio::PAkt as A; let m = |x: A| Msg::PaletteAkt(r.clone(), x);
            a("Standardpalette laden", "", true, true, 0, m(A::Standard)); a("Rückgängig", "Cmd+Z", self.paletten.get(&rel).is_some_and(|p| p.kann_undo()), true, 1, m(A::Undo)); a("Wiederholen", "Cmd+Umschalt+Z", self.paletten.get(&rel).is_some_and(|p| p.kann_redo()), true, 1, m(A::Redo));
        } else if let Some(d) = self.disks.get(&rel) {
            use crate::diskstudio::DAkt as A; let m = |x: A| Msg::DiskAkt(r.clone(), x);
            a("Validieren", "", true, true, 0, m(A::Validieren)); a("Einlegen", "", d.path.is_some() && d.disk.is_ok(), true, 0, m(A::Einlegen)); a("Datei hinzufügen …", "", d.disk.as_ref().is_ok_and(|x| x.has_filesystem()), true, 0, m(A::Hinzufuegen));
            a("Dem Folgesektor folgen", "", d.folge().is_some(), true, 1, m(A::Folgen)); a("Zurück zum vorigen Sektor", "", !d.history.is_empty(), true, 1, m(A::Zurueck)); a(if d.kette_an { "Kette ausblenden" } else { "Kette anzeigen" }, "", d.sel_file.is_some(), true, 1, m(A::KetteAn(!d.kette_an)));
        } else if let Some(s) = self.songs.get(&rel) {
            use crate::songstudio::GAkt as A; let d = &s.doc; let m = |x: A| Msg::SongAkt(r.clone(), x); let spielt = self.audio_song.as_deref() == Some(rel.as_str());
            a(if spielt { "Stopp" } else { "Song abspielen" }, "Leertaste", true, true, 0, m(if spielt { A::Stopp } else { A::SpielSong })); a("Muster abspielen", "Umschalt+Leertaste", !spielt, true, 0, m(A::SpielMuster));
            a("Oktave tiefer", ",", d.oktave > 0, true, 1, m(A::Oktave(-1))); a("Oktave höher", ".", d.oktave < 7, true, 1, m(A::Oktave(1)));
            a("Neues Muster", "", true, true, 2, m(A::MusterNeu)); a("Muster duplizieren", "", true, true, 2, m(A::MusterDupl)); a("Muster löschen", "", d.anzahl_muster() > 1, true, 2, m(A::MusterLoeschen));
            a("Neues Instrument", "", d.anzahl_instrumente() < d.max_instrumente(), false, 3, m(A::InsNeu));
            if d.kanaele > 1 { a("Alle Kanäle einblenden", "", !s.aus.is_empty(), true, 4, m(A::AlleEinblenden)) }
        }
        v
    }
}
