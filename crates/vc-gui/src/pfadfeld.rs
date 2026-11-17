// SPDX-License-Identifier: GPL-3.0-or-later
//! Pfadfeld: Textfeld plus „Wählen …“ (Dialog des Systems) für jede Stelle, an der ein Pfad steht (UX-Sammelpaket). Das Feld bleibt von Hand beschreibbar; der Dialog trägt
//! den gewählten Pfad über denselben Setzer ein wie das Tippen (ein Weg, ein Rückgängig-Schritt). Pfade im Projekt (`Projekt…`) werden relativ zum Projektordner eingetragen.
use super::*;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PfadArt { Ordner, Datei, /** Speichern unter, mit Endung der Art (leer = keine ergänzen) */ Speichern(&'static str), ProjektOrdner, ProjektSpeichern(&'static str) }

/// Setzt den Wert des Feldes (dieselbe Nachricht wie beim Tippen); `Arc`, weil das Ergebnis des Dialogs später ankommt
#[derive(Clone)]
pub struct Setzer(pub Arc<dyn Fn(String) -> Msg + Send + Sync>);
impl std::fmt::Debug for Setzer { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("Setzer") } }

/// Das Feld: `wert` als Text, rechts „Wählen …“
pub fn pfadfeld<'a>(pal: Pal, hinweis: &str, wert: &str, art: PfadArt, breite: Length, setzer: impl Fn(String) -> Msg + Send + Sync + 'static) -> Element<'a, Msg> {
    let s = Setzer(Arc::new(setzer)); let s2 = s.clone();
    row![text_input(hinweis, wert).on_input(move |v| (s2.0)(v)).size(13).padding(5).width(breite).font(vc_gui_ui::fonts::MONO), vc_gui_ui::muster::knopf(pal, "Wählen …", match art { PfadArt::Ordner | PfadArt::ProjektOrdner => "Ordner im Dialog des Systems wählen", PfadArt::Datei => "Datei im Dialog des Systems wählen", _ => "Dateinamen im Dialog des Systems wählen (die Endung wird ergänzt)" }, Some(Msg::PfadWaehlen(art, wert.to_string(), s)), false, false)].spacing(6).align_y(Alignment::Center).wrap().vertical_spacing(4).into()
}

/// Dateiname mit Endung: fehlt dem Namen jede Endung, wird `endung` (mit Punkt) ergänzt; eine vorhandene, auch eine andere, bleibt (die Person hat sie gewählt). Leere Endung: nichts ergänzen
pub fn mit_endung(name: &str, endung: &str) -> String {
    let datei = name.rsplit(['/', '\\']).next().unwrap_or(name);
    if endung.is_empty() || name.is_empty() || datei.is_empty() || datei.trim_start_matches('.').contains('.') { name.to_string() } else { format!("{name}{}{}", if endung.starts_with('.') { "" } else { "." }, endung) }
}

/// Pfad für das Feld: im Projekt relativ (mit `/`), sonst unverändert; beim Speichern mit ergänzter Endung
pub fn feldwert(art: PfadArt, root: &std::path::Path, p: &std::path::Path) -> String {
    let roh = if matches!(art, PfadArt::ProjektOrdner | PfadArt::ProjektSpeichern(_)) { match p.strip_prefix(root) { Ok(r) => r.to_string_lossy().replace('\\', "/"), Err(_) => p.to_string_lossy().into_owned() } } else { p.to_string_lossy().into_owned() };
    match art { PfadArt::Speichern(e) | PfadArt::ProjektSpeichern(e) => mit_endung(&roh, e), _ => roh }
}

impl App {
    /// Dialog öffnen (Start im Ordner des bisherigen Werts, sonst im Projektordner oder Heimordner)
    pub(crate) fn pfad_waehlen(&self, art: PfadArt, wert: String, s: Setzer) -> Task<Msg> {
        let start = { let roh = wert.trim(); let p = if roh.is_empty() { self.root.clone() } else if std::path::Path::new(roh).is_absolute() { std::path::PathBuf::from(roh) } else { self.root.join(roh) }; if p.is_dir() { p } else { p.parent().map(|x| x.to_path_buf()).unwrap_or_else(|| self.root.clone()) } };
        let name = std::path::Path::new(wert.trim()).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        Task::perform(async move {
            let d = rfd::AsyncFileDialog::new().set_directory(start);
            match art { PfadArt::Ordner | PfadArt::ProjektOrdner => d.set_title("Ordner wählen").pick_folder().await, PfadArt::Datei => d.set_title("Datei wählen").pick_file().await, PfadArt::Speichern(_) | PfadArt::ProjektSpeichern(_) => d.set_title("Speichern unter").set_file_name(name).save_file().await }.map(|f| f.path().to_path_buf())
        }, move |p| Msg::PfadGewaehlt(art, s.clone(), p))
    }
    pub(crate) fn pfad_gewaehlt(&mut self, art: PfadArt, s: Setzer, p: Option<std::path::PathBuf>) -> Task<Msg> {
        let Some(p) = p else { return Task::none() }; let v = feldwert(art, &self.root, &p); self.update_innen((s.0)(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn pfade_im_projekt_werden_relativ_eingetragen() {
        let root = std::path::Path::new("/p/spiel");
        assert_eq!(feldwert(PfadArt::ProjektOrdner, root, std::path::Path::new("/p/spiel/import/bilder")), "import/bilder");
        assert_eq!(feldwert(PfadArt::ProjektOrdner, root, std::path::Path::new("/anderswo/x")), "/anderswo/x", "außerhalb bleibt absolut (die Prüfung meldet es)");
        assert_eq!(feldwert(PfadArt::Ordner, root, std::path::Path::new("/p/spiel/x")), "/p/spiel/x", "Systempfade bleiben absolut");
    }
    #[test] fn endung_wird_ergaenzt_wenn_sie_fehlt_und_nie_ersetzt() {
        assert_eq!(mit_endung("build/spiel", "d64"), "build/spiel.d64"); assert_eq!(mit_endung("build/spiel", ".asm"), "build/spiel.asm"); assert_eq!(mit_endung("spiel.d81", "d64"), "spiel.d81", "gewählte Endung bleibt");
        assert_eq!(mit_endung("ordner.v2/spiel", "d64"), "ordner.v2/spiel.d64", "Punkt im Ordnernamen zählt nicht"); assert_eq!(mit_endung(".versteckt", "d64"), ".versteckt.d64"); assert_eq!(mit_endung("x", ""), "x"); assert_eq!(mit_endung("", "d64"), "");
        assert_eq!(feldwert(PfadArt::ProjektSpeichern("d64"), std::path::Path::new("/p"), std::path::Path::new("/p/build/disk")), "build/disk.d64");
    }
}
