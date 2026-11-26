// SPDX-License-Identifier: GPL-3.0-or-later
// Liste der Kataloge aus catalog/de/*.ftl erzeugen: je Bereich (de, en) mit include_str!.
use std::fmt::Write;

fn main() {
    let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("catalog");
    println!("cargo:rerun-if-changed=catalog");
    let mut names: Vec<String> = std::fs::read_dir(root.join("de")).expect("catalog/de fehlt").flatten()
        .filter_map(|e| e.file_name().to_str().and_then(|n| n.strip_suffix(".ftl").map(str::to_string))).collect();
    names.sort();
    let mut out = String::from("pub const CATALOGS: &[(&str, &str, &str)] = &[\n");
    for n in &names {
        // Fehlt die englische Datei (neuer Bereich, Übersetzung folgt), gilt eine leere: Rückfall auf Deutsch; tools/check_i18n.py meldet es als Fehler
        let en = root.join("en").join(format!("{n}.ftl"));
        let en = if en.exists() { en } else {
            let leer = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join(format!("leer-{n}.ftl"));
            std::fs::write(&leer, "").unwrap();
            println!("cargo:warning=catalog/en/{n}.ftl fehlt (Rückfall auf Deutsch)");
            leer
        };
        writeln!(out, "    ({n:?}, include_str!({:?}), include_str!({:?})),", root.join("de").join(format!("{n}.ftl")).to_str().unwrap(), en.to_str().unwrap()).unwrap();
    }
    out.push_str("];\n");
    std::fs::write(std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("catalogs.rs"), out).unwrap();

    // Englische Überlagerungen der Datendateien (data/i18n/en/**.json), eingebettet; Schlüssel = Pfad der Quelldatei unter data/
    let data_en = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../data/i18n/en");
    println!("cargo:rerun-if-changed=../../data/i18n/en");
    let mut files = vec![];
    fn walk(dir: &std::path::Path, base: &std::path::Path, out: &mut Vec<(String, std::path::PathBuf)>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() { walk(&p, base, out); }
            else if p.extension().is_some_and(|x| x == "json") { out.push((p.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/"), p)); }
        }
    }
    walk(&data_en, &data_en, &mut files);
    files.sort();
    let mut ov = String::from("pub const OVERLAYS: &[(&str, &str)] = &[\n");
    for (rel, p) in &files {
        if let Ok(p) = p.canonicalize() { writeln!(ov, "    ({rel:?}, include_str!({:?})),", p.to_str().unwrap()).unwrap(); }
    }
    ov.push_str("];\n");
    std::fs::write(std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("data_overlays.rs"), ov).unwrap();
}
