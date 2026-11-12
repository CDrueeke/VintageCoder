// SPDX-License-Identifier: GPL-3.0-or-later
//! Werkzeug für `data/roms/known.json`: Einträge aus echten Dateien erzeugen (Prüfsummen werden nie von Hand getippt).
use crate::hash;
use crate::known::{KnownRom, Use};
use crate::systems;
use std::path::Path;

/// Standardrevisionen (System, Rolle, Revision) – die jeweils verbreitetste Fassung
const DEFAULTS: &[(&str, &str, &str)] = &[
    ("c64", "kernal", "901227-03"), ("c64", "basic", "901226-01"), ("c64", "chargen", "901225-01"),
    ("c128", "kernal", "318020-05"), ("c128", "basiclo", "318018-04"), ("c128", "basichi", "318019-04"), ("c128", "chargen", "315079-01"),
    ("vic20", "kernal", "901486-06"), ("vic20", "basic", "901486-01"), ("vic20", "chargen", "901460-03"),
    ("plus4", "kernal", "318005-05"), ("plus4", "basic", "318006-01"),
    ("c64", "dos1541", "325302-01+901229-05"), ("c64", "dos1571", "310654-05"), ("c64", "dos1581", "318045-02"),
];

pub fn entry(path: &Path, system: &str, role: &str, rev: &str, free: bool, default: bool, note: &str) -> Result<KnownRom, String> { entry_with_sets(path, system, role, rev, free, default, note, &[]) }

/// Wie [`entry`], mit Zeichensätzen abweichend von den Standardorten: `name:offset:size:profile[:Anzeigetext]` (Offset dezimal oder 0x…)
#[allow(clippy::too_many_arguments)]
pub fn entry_with_sets(path: &Path, system: &str, role: &str, rev: &str, free: bool, default: bool, note: &str, sets: &[String]) -> Result<KnownRom, String> {
    let data = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let sd = systems::system(system).ok_or_else(|| format!("unbekanntes System {system}"))?;
    let rd = systems::role(system, role).ok_or_else(|| format!("unbekannte Rolle {system}/{role}"))?;
    if !rd.sizes.contains(&data.len()) { return Err(format!("{}: {} Byte, erwartet {:?} für {system}/{role}", path.display(), data.len(), rd.sizes)); }
    let default = default || DEFAULTS.contains(&(system, role, rev));
    Ok(KnownRom {
        id: format!("{system}.{role}.{}", rev.to_lowercase().replace(' ', "-")),
        label: format!("{} {} {rev}", sd.label, rd.label),
        size: data.len(), crc32: hash::crc32_hex(&data), sha1: hash::sha1_hex(&data),
        uses: vec![Use { system: system.into(), role: role.into() }], default, free, note: note.into(), sets: parse_sets(sets, data.len())?,
    })
}

fn parse_sets(specs: &[String], len: usize) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let num = |t: &str| -> Result<usize, String> { match t.strip_prefix("0x") { Some(h) => usize::from_str_radix(h, 16), None => t.parse() }.map_err(|_| format!("Zahl '{t}' ungültig")) };
    let mut m = serde_json::Map::new();
    for spec in specs {
        let p: Vec<&str> = spec.splitn(5, ':').collect();
        if p.len() < 4 { return Err(format!("--set {spec}: name:offset:size:profile[:Text] erwartet")); }
        let (off, size) = (num(p[1])?, num(p[2])?);
        if off + size > len { return Err(format!("--set {spec}: liegt hinter dem Dateiende ({len} Byte)")); }
        m.insert(p[0].into(), serde_json::json!({ "offset": off, "size": size, "profile": p[3], "label": p.get(4).copied().unwrap_or(p[0]) }));
    }
    Ok(m)
}

/// VICE-Datenordner (`C64`, `C128`, `VIC20`, `PLUS4`, `DRIVES`): Rolle und Revision stammen aus dem Dateinamen
/// (`kernal-901227-03.bin`, `kernal.901486-02.bin`). Dateien mit unbekanntem Namensanfang (Landesvarianten, Prototypen) werden übergangen.
pub fn from_vice(dir: &Path) -> Vec<Result<KnownRom, String>> {
    let mut out = vec![];
    for (folder, system) in [("C64", "c64"), ("C128", "c128"), ("VIC20", "vic20"), ("PLUS4", "plus4"), ("DRIVES", "c64")] {
        let Ok(rd) = std::fs::read_dir(dir.join(folder)) else { continue };
        let mut files: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect();
        files.sort();
        for p in files {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            let Some(stem) = name.strip_suffix(".bin") else { continue };
            let Some(i) = stem.find(['-', '.']) else { continue };
            let (prefix, rev) = (&stem[..i], &stem[i + 1..]);
            if rev.contains("364") && system == "plus4" { continue; }   // Prototyp-ROMs des 364
            let role = match (folder, prefix) {
                ("PLUS4", "3plus1") => if rev.starts_with("317053") { "functionlo" } else if rev.starts_with("317054") { "functionhi" } else { continue },
                ("DRIVES", p) if matches!(p, "dos1541" | "dos1571" | "dos1581") => p,
                ("DRIVES", _) => continue,
                (_, p) if systems::role(system, p).is_some() => p,
                _ => continue,
            };
            out.push(entry(&p, system, role, rev, false, false, "Revision aus dem Dateinamen der Quelle (VICE)"));
            // der C128 nutzt dieselben Laufwerks-ROMs wie der C64
            if folder == "DRIVES" && role == "dos1571" || folder == "DRIVES" && role == "dos1581" {
                if let Some(Ok(e)) = out.last_mut() { e.uses.push(Use { system: "c128".into(), role: role.into() }); }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vice_namen_ohne_echte_roms() {
        let d = std::env::temp_dir().join(format!("vc-roms-emit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        for (f, n) in [("C64/kernal-901227-03.bin", 8192), ("C64/chargen-901225-01.bin", 4096), ("VIC20/kernal.901486-06.bin", 8192), ("PLUS4/3plus1-317053-01.bin", 16384), ("PLUS4/kernal-364.bin", 16384), ("C128/kernalde", 16384), ("DRIVES/dos2031-901484-03+05.bin", 16384)] {
            let p = d.join(f);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, vec![7u8; n]).unwrap();
        }
        let v: Vec<KnownRom> = from_vice(&d).into_iter().map(|r| r.unwrap()).collect();
        let ids: Vec<&str> = v.iter().map(|k| k.id.as_str()).collect();
        assert_eq!(ids, ["c64.chargen.901225-01", "c64.kernal.901227-03", "vic20.kernal.901486-06", "plus4.functionlo.317053-01"]);
        assert!(v.iter().all(|k| k.default || k.id.contains("317053")));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn falsche_groesse_wird_gemeldet() {
        let p = std::env::temp_dir().join(format!("vc-roms-emit-size-{}", std::process::id()));
        std::fs::write(&p, [0u8; 10]).unwrap();
        assert!(entry(&p, "c64", "kernal", "x", false, false, "").is_err());
        let _ = std::fs::remove_file(&p);
    }
}
