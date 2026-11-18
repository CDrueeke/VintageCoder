// SPDX-License-Identifier: GPL-3.0-or-later
//! Lizenzhinweis beim Übersetzen: Auswertung der Lizenzkennungen der eingebundenen Quelltexte ([`vc_asm::license`]).
//! Copyleft (GPL, LGPL, AGPL …) ergibt einen sachlichen Hinweis `L001` (kein Fehler); `deny` in der Projektdatei
//! (`"licenses": {"deny": [...], "allow": [...]}`) oder `--deny-license` lässt den Bau mit `L002` scheitern, `allow` unterdrückt den Hinweis.
//! Das Ergebnis ist nur so gut wie die Kopfzeilen der Dateien und kein Rechtsurteil.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Mutex;
use vc_asm::license::{is_copyleft, LicenseEntry, Origin};

/// Erlaubnis und Sperre (Abschnitt `licenses` der `.vcprj`, ergänzt um `--deny-license`)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub deny: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub allow: Vec<String>,
}

/// Kurzer Lizenzhinweis für `vcc --license` und den Über-Dialog (Wortlaut: LICENSING.md)
pub const NOTICE_SHORT: &str = "VintageCoder steht unter der GNU General Public License, Version 3 oder später (Datei LICENSE). Was du mit VintageCoder erzeugst, gehört dir und ist frei nutzbar; die Sprache VCASM und die Dateiformate sind frei, vcc und vc-lsp darfst du aus beliebiger Software aufrufen. Systemdateien, Vorlagen und Beispiele stehen unter MIT-0 (LICENSES/MIT-0.txt). Die Einzelheiten stehen in LICENSING.md.";

static CLI: Mutex<Policy> = Mutex::new(Policy { deny: vec![], allow: vec![] });

/// Von der Kommandozeile (`vcc build --deny-license …`); gilt für alle folgenden Bauläufe des Prozesses
pub fn set_cli_policy(p: Policy) { if let Ok(mut g) = CLI.lock() { *g = p; } }

/// Richtlinie eines Projekts: Abschnitt `licenses` der `.vcprj` im Ordner plus die Angaben der Kommandozeile
pub fn policy_for(root: &Path) -> Policy {
    let mut p = crate::projfile::find(root).and_then(|f| crate::projfile::ProjectFile::load(&f).ok()).map(|pf| pf.licenses).unwrap_or_default();
    if let Ok(c) = CLI.lock() { p.deny.extend(c.deny.iter().cloned()); p.allow.extend(c.allow.iter().cloned()); }
    p
}

/// Passt `pattern` (ohne Beachtung der Schreibweise, am Ende mit `*` als Platzhalter, z. B. `GPL-*`) auf die Kennung?
fn matches(pattern: &str, license: &str) -> bool {
    let p = pattern.trim().to_ascii_lowercase();
    license.split(|c: char| c.is_whitespace() || c == '(' || c == ')').filter(|t| !t.is_empty() && !matches!(*t, "AND" | "OR" | "WITH")).any(|t| {
        let t = t.to_ascii_lowercase();
        match p.strip_suffix('*') { Some(pre) => t.starts_with(pre), None => t == p }
    })
}

impl Policy {
    pub fn denies(&self, license: &str) -> bool { self.deny.iter().any(|d| matches(d, license)) }
    pub fn allows(&self, license: &str) -> bool { self.allow.iter().any(|a| matches(a, license)) }
    pub fn is_empty(&self) -> bool { self.deny.is_empty() && self.allow.is_empty() }
}

/// Anzeigepfad: im Projekt relativ, in den Systemdateien `data/…`, sonst unverändert
pub fn show_path(root: &Path, data: &Path, file: &str) -> String {
    let p = Path::new(file);
    if let Ok(r) = p.strip_prefix(root) { return vc_core::paths::slashes(&r.to_string_lossy()); }
    if let Ok(r) = p.strip_prefix(data) { return format!("data/{}", vc_core::paths::slashes(&r.to_string_lossy())); }
    vc_core::paths::slashes(file)
}

fn holder_text(h: &str) -> String { if h.is_empty() { "Urheber unbekannt".into() } else { h.to_string() } }

/// Hinweise `L001`, je Lizenz einer (nur Copyleft, nicht in `allow`). Texte ohne Vorspann „Hinweis (L001):“.
pub fn copyleft_notes(entries: &[LicenseEntry], policy: &Policy, show: &dyn Fn(&str) -> String) -> Vec<String> {
    let mut groups: Vec<(String, Vec<&LicenseEntry>)> = vec![];
    for e in entries.iter().filter(|e| is_copyleft(&e.license) && !policy.allows(&e.license)) {
        match groups.iter_mut().find(|g| g.0 == e.license) { Some(g) => g.1.push(e), None => groups.push((e.license.clone(), vec![e])) }
    }
    groups.into_iter().map(|(lic, es)| {
        let mut files: Vec<String> = es.iter().map(|e| show(&e.file)).collect();
        let n = files.len();
        files.truncate(3);
        let mut holders: Vec<String> = vec![];
        for e in &es { let h = holder_text(&e.holder); if !holders.contains(&h) { holders.push(h); } }
        format!("Das Programm enthält Quelltext unter {lic} ({}{}, {}). Wer das Programm weitergibt, muss die Bedingungen dieser Lizenz einhalten (u. a. den Quelltext bereitstellen). Mit --license-report erhältst du die Liste.",
            files.join(", "), if n > 3 { format!(" u. a., {n} Dateien") } else { String::new() }, holders.join("; "))
    }).collect()
}

/// Gesperrte Lizenzen: Fehlertexte `L002`, je beitragender Datei einer
pub fn denied(entries: &[LicenseEntry], policy: &Policy, show: &dyn Fn(&str) -> String) -> Vec<String> {
    entries.iter().filter(|e| policy.denies(&e.license)).map(|e| format!("Die Lizenz {} ist für dieses Projekt gesperrt (--deny-license oder licenses.deny in der Projektdatei): {} ({}).", e.license, show(&e.file), holder_text(&e.holder))).collect()
}

pub fn origin_text(o: &Origin) -> &'static str { match o { Origin::Header => "Kopfzeile", Origin::Folder(_) => "Ordnervermerk" } }

pub fn entry_json(e: &LicenseEntry, show: &dyn Fn(&str) -> String) -> Value {
    json!({ "file": show(&e.file), "license": e.license, "holder": e.holder, "source": origin_text(&e.origin),
            "notice": match &e.origin { Origin::Folder(f) => Some(show(f)), Origin::Header => None }, "copyleft": is_copyleft(&e.license) })
}

/// Bericht (`build/LICENSES.txt`, `--license-report`): `programs` = (Programm, Ziel, Einträge, Anzahl Dateien ohne Kennung)
pub fn report_text(programs: &[(String, String, Vec<LicenseEntry>, usize)], show: &dyn Fn(&str) -> String) -> String {
    let mut s = String::from("Lizenzen der übersetzten Quelltexte (erzeugt von VintageCoder)\n\
        Gelesen werden die Zeilen SPDX-License-Identifier in den Kopfzeilen der Dateien (sonst ein Ordnervermerk LICENSE).\n\
        Das ist nur so gut wie diese Kopfzeilen; Binärdaten und Dateien ohne Kennung werden nicht erkannt. Kein Rechtsurteil.\n");
    for (prog, target, es, none) in programs {
        s.push_str(&format!("\n== {prog} (Ziel {target}) ==\n"));
        for e in es {
            s.push_str(&format!("{:<18} {}  [{}; {}]{}\n", e.license, show(&e.file), holder_text(&e.holder), origin_text(&e.origin), if is_copyleft(&e.license) { "  Copyleft" } else { "" }));
        }
        if es.is_empty() { s.push_str("(keine Lizenzkennung gefunden)\n"); }
        if *none > 0 { s.push_str(&format!("{none} Datei(en) ohne Kennung (neutral)\n")); }
    }
    s
}

pub fn report_json(programs: &[(String, String, Vec<LicenseEntry>, usize)], show: &dyn Fn(&str) -> String) -> Value {
    json!({ "programs": programs.iter().map(|(p, t, es, none)| json!({ "program": p, "target": t, "files": es.iter().map(|e| entry_json(e, show)).collect::<Vec<_>>(), "without_license": none })).collect::<Vec<_>>() })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn e(file: &str, lic: &str, h: &str) -> LicenseEntry { LicenseEntry { file: file.into(), license: lic.into(), holder: h.into(), origin: Origin::Header } }

    #[test]
    fn patterns_match_case_insensitive_and_with_wildcard() {
        assert!(matches("GPL-3.0-only", "gpl-3.0-only"));
        assert!(matches("GPL-*", "GPL-2.0-or-later"));
        assert!(!matches("GPL-*", "LGPL-2.1-only"));
        assert!(matches("GPL-3.0-only", "MIT OR GPL-3.0-only"));
        assert!(!matches("MIT", "MIT-0"));
    }

    #[test]
    fn note_names_license_files_and_holder_but_only_for_copyleft() {
        let es = vec![e("a/rmtplayr.asm", "GPL-3.0-only", "Radek Sterba"), e("a/rmt.inc", "GPL-3.0-only", "Radek Sterba"), e("sys/c64.inc", "MIT-0", "")];
        let n = copyleft_notes(&es, &Policy::default(), &|f| f.to_string());
        assert_eq!(n.len(), 1);
        assert!(n[0].contains("GPL-3.0-only (a/rmtplayr.asm, a/rmt.inc, Radek Sterba)"), "{}", n[0]);
        assert!(n[0].contains("--license-report"));
        let p = Policy { allow: vec!["GPL-3.0-only".into()], deny: vec![] };
        assert!(copyleft_notes(&es, &p, &|f| f.to_string()).is_empty());
    }

    #[test]
    fn deny_lists_the_offending_files() {
        let es = vec![e("a/rmtplayr.asm", "GPL-3.0-only", "R"), e("b.asm", "MIT-0", "")];
        let p = Policy { deny: vec!["GPL-3.0-only".into(), "GPL-3.0-or-later".into()], allow: vec![] };
        let d = denied(&es, &p, &|f| f.to_string());
        assert_eq!(d.len(), 1);
        assert!(d[0].contains("a/rmtplayr.asm"));
    }
}
