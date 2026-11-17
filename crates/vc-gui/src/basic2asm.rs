// SPDX-License-Identifier: GPL-3.0-or-later
//! „BASIC in Assembler umwandeln“: die geöffnete BASIC-Datei wird mit dem Umsetzer (`vc_basic::b2a`, derselbe wie `vcc basic2asm`) in eine neue Assembler-Datei
//! neben der Quelle übersetzt (`<name>_asm.asm`, Abbildung `<name>_asm.b2a.json`); eine vorhandene Datei wird nie überschrieben (der Name bekommt eine Nummer).
//! Warnungen und Fehler des Umsetzers kommen in die Probleme; die neue Datei öffnet im Editor.
use super::*;

/// Ziel für den Umsetzer: das Systemkürzel des Projektziels, sonst `c64`
fn umsetzer_ziel(system: &str) -> &'static str { vc_basic::b2a::TARGETS.iter().map(|t| t.id).find(|id| *id == system).unwrap_or("c64") }

/// Freier Dateiname `<stamm>_asm.asm`, bei Bedarf `<stamm>_asm-2.asm` …
pub fn freier_name(root: &std::path::Path, ordner: &str, stamm: &str) -> String {
    let pfad = |n: &str| if ordner.is_empty() { n.to_string() } else { format!("{ordner}/{n}") };
    let mut i = 1; loop { let n = pfad(&if i == 1 { format!("{stamm}_asm.asm") } else { format!("{stamm}_asm-{i}.asm") }); if !root.join(&n).exists() { return n } i += 1 }
}

impl App {
    /// Befehl „BASIC in Assembler umwandeln“ für die aktive Datei
    pub(crate) fn basic_nach_asm(&mut self) {
        let Some(rel) = self.name().filter(|n| n.to_lowercase().ends_with(".bas") && self.editoren.contains_key(n)) else { self.log.push("BASIC in Assembler umwandeln: erst eine BASIC-Datei (.bas) öffnen.".into()); return };
        let system = self.proj.pf.as_ref().and_then(|pf| pf.targets.first()).map(|t| if t.system.is_empty() { t.id.clone() } else { t.system.clone() }).unwrap_or_else(|| "c64".into());
        let ziel = umsetzer_ziel(&system);
        let Some(ti) = vc_basic::b2a::target_info(ziel) else { self.log.push(format!("BASIC in Assembler: Ziel „{ziel}“ ist noch nicht umgesetzt.")); return };
        let d = match vc_basic::Dialect::load(&self.data, ti.dialect) { Ok(d) => d, Err(e) => { self.log.push(format!("BASIC in Assembler: {e}")); return } };
        let src = self.editoren[&rel].text();
        let mut o = vc_basic::b2a::B2aOptions::new(""); o.file = rel.clone(); o.target = ziel.to_string(); o.data_dir = Some(self.data.clone());
        match vc_basic::b2a::convert(&src, &d, &o) {
            Ok(c) => {
                for w in &c.warnings { self.probleme.push(Problem { datei: w.file.clone(), zeile: w.line, text: w.message.clone(), fehler: false }) }
                let (ordner, name) = rel.rsplit_once('/').map_or(("", rel.as_str()), |(o, n)| (o, n)); let stamm = name.rsplit_once('.').map_or(name, |(s, _)| s);
                let aus = freier_name(&self.root, ordner, stamm);
                if let Err(e) = std::fs::write(project::pfad(&self.root, &aus), &c.asm) { self.log.push(format!("{aus}: {e}")); return }
                let karte = aus.trim_end_matches(".asm").to_string() + ".b2a.json"; let _ = std::fs::write(project::pfad(&self.root, &karte), c.map_json(&rel));
                self.log.push(format!("{rel} → {aus}: Stufe {}, {} Zeilen, Module: {}{}", c.stage, c.map.len(), c.modules.join(", "), if c.warnings.is_empty() { String::new() } else { format!(" · {} Hinweis(e) in den Problemen", c.warnings.len()) }));
                self.projekt_neu_laden(); self.oeffne(&aus);
            }
            Err(e) => { for x in &e { self.probleme.push(Problem { datei: x.file.clone(), zeile: x.line, text: x.message.clone(), fehler: true }) } self.log.push(format!("BASIC in Assembler: {} Fehler, siehe Probleme", e.len())); self.schublade_zeigen() }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn freier_name_zaehlt_hoch_und_ueberschreibt_nichts() {
        let t = std::env::temp_dir().join(format!("vcgui-b2a-{}", std::process::id())); let _ = std::fs::remove_dir_all(&t); std::fs::create_dir_all(t.join("o")).unwrap();
        assert_eq!(freier_name(&t, "o", "spiel"), "o/spiel_asm.asm"); std::fs::write(t.join("o/spiel_asm.asm"), "x").unwrap(); assert_eq!(freier_name(&t, "o", "spiel"), "o/spiel_asm-2.asm"); assert_eq!(freier_name(&t, "", "a"), "a_asm.asm");
        let _ = std::fs::remove_dir_all(&t);
    }
}
