// SPDX-License-Identifier: GPL-3.0-or-later
//! Dienste der Entwicklungsumgebung als Bibliothek: Bauen, Debug-Sitzung mit Emulator-Adaptern, Start im Emulator und Hilfe.
//! (Die Oberfläche ist `vc-app`; die frühere Web-Oberfläche steht nur noch in der Git-Historie.)
use std::path::{Path, PathBuf};

pub mod args;
pub mod b2a;
pub mod blob;
pub mod build;
pub mod buildlog;
pub mod cond;
pub mod debug;
pub mod debuglog;
pub mod diskimage;
pub mod emulators;
pub mod features;
pub mod feedback;
pub mod help;
pub mod help_images;
pub mod importfiles;
pub mod launch;
pub mod livebild;
pub mod listen;
pub mod mega65net;
pub mod platform;
pub mod projfile;
pub mod reorganize;
pub mod roms;
pub mod settings;
pub mod setup;
pub mod streams;
pub mod structure;
pub mod sysmap;
pub mod targets;
pub mod tests;
pub mod trace;

pub fn find_data_dir(start: &Path) -> Option<PathBuf> {
    let mut d = start.to_path_buf();
    loop {
        if d.join("data/cpus").is_dir() { return Some(d.join("data")); }
        // macOS-App-Bundle: Contents/MacOS/<programm>, Daten in Contents/Resources/data
        if d.join("Resources/data/cpus").is_dir() { return Some(d.join("Resources/data")); }
        // Linux-Pakete (FHS): <prefix>/bin/<programm>, Daten in <prefix>/share/vintagecoder/data; ein näher liegendes data/ gewinnt weiterhin
        if d.join("share/vintagecoder/data/cpus").is_dir() { return Some(d.join("share/vintagecoder/data")); }
        if !d.pop() { return None; }
    }
}

#[cfg(test)]
mod find_data_dir_tests {
    use super::find_data_dir;
    use std::fs;

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("vc-fdd-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn linux_paket_usr_share() {
        let root = tmp("fhs");
        fs::create_dir_all(root.join("usr/bin")).unwrap();
        fs::create_dir_all(root.join("usr/share/vintagecoder/data/cpus")).unwrap();
        assert_eq!(find_data_dir(&root.join("usr/bin/vcc")), Some(root.join("usr/share/vintagecoder/data")));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn data_neben_dem_programm_gewinnt_vor_usr_share() {
        let root = tmp("vorrang");
        fs::create_dir_all(root.join("usr/bin/data/cpus")).unwrap();
        fs::create_dir_all(root.join("usr/share/vintagecoder/data/cpus")).unwrap();
        assert_eq!(find_data_dir(&root.join("usr/bin/vcc")), Some(root.join("usr/bin/data")));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn macos_bundle_und_entwicklung_unveraendert() {
        let root = tmp("mac");
        fs::create_dir_all(root.join("App.app/Contents/Resources/data/cpus")).unwrap();
        fs::create_dir_all(root.join("App.app/Contents/MacOS")).unwrap();
        assert_eq!(find_data_dir(&root.join("App.app/Contents/MacOS/vintagecoder")), Some(root.join("App.app/Contents/Resources/data")));
        let _ = fs::remove_dir_all(&root);
        // Quellbaum: data/ liegt drei Ebenen über dem Cargo-Zielordner
        let dev = find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        assert!(dev.ends_with("data") && dev.join("cpus").is_dir());
    }

    #[test]
    fn ohne_daten_nichts() {
        let root = tmp("leer");
        fs::create_dir_all(root.join("a/b")).unwrap();
        let r = find_data_dir(&root.join("a/b"));
        assert!(r.map_or(true, |p| !p.starts_with(&root)));
        let _ = fs::remove_dir_all(&root);
    }
}
