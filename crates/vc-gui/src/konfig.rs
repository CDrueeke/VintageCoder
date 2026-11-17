// SPDX-License-Identifier: GPL-3.0-or-later
//! Konfiguration sichern, übernehmen und zurücksetzen (Einstellungen → Allgemein → „Konfiguration“). Exportiert wird die Einstellungsdatei der IDE (`settings.json`:
//! Zielsysteme, Emulatoren, Maschinen, Hilfe, Update, Befehlsparameter) **ohne Passwörter** der Maschinen; die Oberflächeneinstellungen (Farbschema, Zoom …) bleiben, wie sie sind.
//! Importieren fragt vorher mit einer Zusammenfassung; Zurücksetzen fragt ebenso. Passwörter bleiben beim Importieren erhalten, wenn die Maschine schon bekannt ist.
use super::*;
use vc_ide::settings::Settings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KArt { Export, Import }

/// Einstellungen für die Weitergabe: Passwörter der Maschinen leer
pub fn fuer_export(s: &Settings) -> Settings { let mut c = s.clone(); for m in &mut c.machines { m.password.clear() } c }

/// Zusammenfassung für die Rückfrage vor dem Übernehmen
pub fn zusammenfassung(s: &Settings) -> String {
    format!("{} Zielsysteme, {} Maschine(n){}", s.targets.len(), s.machines.len(), if s.projects_dir.trim().is_empty() { String::new() } else { format!(", Projektordner {}", s.projects_dir) })
}

/// Übernommene Einstellungen: Passwörter gleicher Maschinen bleiben, wenn die Datei keine enthält
pub fn uebernehmen(aktuell: &Settings, neu: Settings) -> Settings {
    let mut n = neu; for m in &mut n.machines { if m.password.is_empty() { if let Some(a) = aktuell.machine(&m.id) { m.password = a.password.clone() } } } n
}

impl App {
    /// Dialog des Systems; das Ergebnis kommt als `Msg::KonfigPfad`
    pub(crate) fn konfig_dialog(&self, art: KArt) -> Task<Msg> {
        let start = self.root.clone();
        Task::perform(async move {
            let d = rfd::AsyncFileDialog::new().set_directory(start).add_filter("Einstellungen (JSON)", &["json"]);
            match art { KArt::Export => d.set_title("Konfiguration exportieren").set_file_name("vintagecoder-einstellungen.json").save_file().await, KArt::Import => d.set_title("Konfiguration importieren").pick_file().await }.map(|f| f.path().to_path_buf())
        }, move |p| Msg::KonfigPfad(art, p))
    }
    pub(crate) fn konfig_pfad(&mut self, art: KArt, p: Option<PathBuf>) {
        let Some(p) = p else { return };
        match art {
            KArt::Export => { let p = PathBuf::from(crate::pfadfeld::mit_endung(&p.to_string_lossy(), "json")); self.satz.meldung = Some(match fuer_export(&self.satz.entwurf).save_to(&p) { Ok(()) => (true, format!("Konfiguration exportiert nach {} (ohne Passwörter).", p.display())), Err(e) => (false, format!("Export fehlgeschlagen: {e}")) }) }
            KArt::Import => match Settings::load_from(&p) { Ok(mut s) => { s.migrate_legacy(); self.rueckfrage = Some(Rueckfrage::KonfigImport(p.clone(), zusammenfassung(&s))) } Err(e) => self.satz.meldung = Some((false, format!("Import: {e}"))) },
        }
    }
    pub(crate) fn konfig_import_ja(&mut self, p: &std::path::Path) {
        match Settings::load_from(p) { Ok(mut s) => { s.migrate_legacy(); let neu = uebernehmen(&self.satz.entwurf, s); self.satz.entwurf = neu; self.satz.speichern(); self.log.push(format!("Konfiguration übernommen aus {}", p.display())) } Err(e) => self.satz.meldung = Some((false, format!("Import: {e}"))) }
    }
    pub(crate) fn konfig_reset_ja(&mut self) { self.satz.entwurf = Settings::default(); self.satz.speichern(); self.log.push("Einstellungen auf die Vorgabe zurückgesetzt (Passwörter und Zielsysteme eingeschlossen).".into()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn export_ohne_passwoerter_import_behaelt_bekannte_passwoerter() {
        let mut a = Settings::default(); a.machines.push(vc_ide::settings::Machine { id: "m1".into(), password: "pw1".into(), host: "h".into(), ..Default::default() });
        let e = fuer_export(&a); assert!(e.machines[0].password.is_empty() && e.machines[0].host == "h", "Passwort leer, Rest bleibt"); assert_eq!(a.machines[0].password, "pw1", "das Original bleibt unverändert");
        let n = uebernehmen(&a, e.clone()); assert_eq!(n.machines[0].password, "pw1", "bekannte Maschine behält ihr Passwort");
        let mut fremd = e; fremd.machines[0].id = "neu".into(); assert!(uebernehmen(&a, fremd).machines[0].password.is_empty(), "unbekannte Maschine: kein Passwort erfunden");
        assert!(zusammenfassung(&a).contains("1 Maschine"));
    }
}
