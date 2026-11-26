// SPDX-License-Identifier: GPL-3.0-or-later
//! Speichern ohne Datenverlust: erst in eine Nachbardatei schreiben, auf den Datenträger zwingen (`sync_all`), dann umbenennen.
//! Ein Absturz, Stromausfall oder voller Datenträger lässt so entweder die alte oder die neue Datei zurück, nie eine leere oder halbe.
//! Rechte einer vorhandenen Datei bleiben erhalten; ist das Ziel ein Symlink, wird dessen Ziel ersetzt (der Link bleibt).
use std::io::Write;
use std::path::{Path, PathBuf};

/// Schreibt `bytes` atomar nach `path`
pub fn schreiben(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> std::io::Result<()> {
    let path = path.as_ref();
    // Symlink: das Ziel ersetzen, nicht den Link
    let ziel: PathBuf = match std::fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_symlink() => std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()),
        _ => path.to_path_buf(),
    };
    let dir = ziel.parent().filter(|d| !d.as_os_str().is_empty()).map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));
    let name = ziel.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    let r = (|| -> std::io::Result<()> {
        let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(&tmp)?;
        f.write_all(bytes.as_ref())?;
        if let Ok(m) = std::fs::metadata(&ziel) { let _ = f.set_permissions(m.permissions()); }
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, &ziel)
    })();
    if r.is_err() { let _ = std::fs::remove_file(&tmp); }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tmp(tag: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc-atomar-{tag}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }

    #[test]
    fn schreibt_ersetzt_und_hinterlaesst_nichts() {
        let d = tmp("a");
        schreiben(d.join("x.txt"), "eins").unwrap();
        schreiben(d.join("x.txt"), b"zwei").unwrap();
        assert_eq!(std::fs::read_to_string(d.join("x.txt")).unwrap(), "zwei");
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), 1, "keine temporäre Datei übrig");
    }

    #[test]
    fn fehler_laesst_die_alte_datei_und_raeumt_auf() {
        let d = tmp("b");
        schreiben(d.join("x.txt"), "alt").unwrap();
        // Ziel ist ein Ordner: rename scheitert
        std::fs::create_dir_all(d.join("ordner")).unwrap();
        assert!(schreiben(d.join("ordner"), "neu").is_err());
        assert_eq!(std::fs::read_to_string(d.join("x.txt")).unwrap(), "alt");
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), 2);
        // Elternordner fehlt
        assert!(schreiben(d.join("fehlt/x"), "n").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rechte_und_symlink_bleiben() {
        use std::os::unix::fs::PermissionsExt;
        let d = tmp("c");
        std::fs::write(d.join("run.sh"), "a").unwrap();
        std::fs::set_permissions(d.join("run.sh"), std::fs::Permissions::from_mode(0o755)).unwrap();
        schreiben(d.join("run.sh"), "b").unwrap();
        assert_eq!(std::fs::metadata(d.join("run.sh")).unwrap().permissions().mode() & 0o777, 0o755);
        std::os::unix::fs::symlink(d.join("run.sh"), d.join("link")).unwrap();
        schreiben(d.join("link"), "c").unwrap();
        assert!(std::fs::symlink_metadata(d.join("link")).unwrap().file_type().is_symlink(), "Link bleibt Link");
        assert_eq!(std::fs::read_to_string(d.join("run.sh")).unwrap(), "c");
    }
}
