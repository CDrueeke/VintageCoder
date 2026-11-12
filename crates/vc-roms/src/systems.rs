// SPDX-License-Identifier: GPL-3.0-or-later
//! Systeme und ROM-Rollen: welche Dateien ein System kennt, wie groß sie sind und wie sie heißen sollten.
//! Größen: VICE 3.10 bzw. lokal gemessen (siehe `Konzept-ROM-Verzeichnis.md`, Abschnitt 3.3).

/// Eine Rolle eines Systems (z. B. C64 `kernal`)
pub struct RoleDef {
    pub id: &'static str,
    pub label: &'static str,
    /// erlaubte Dateigrößen in Byte
    pub sizes: &'static [usize],
    /// bequemer Standardname im ROM-Ordner (Konvention, nicht Pflicht)
    pub file: &'static str,
    /// ohne diese Rolle läuft das System nicht (sonst optional)
    pub required: bool,
    /// Wörter im Dateinamen, die auf die Rolle hindeuten (Kleinbuchstaben); nur für unbekannte Revisionen
    pub keywords: &'static [&'static str],
}

pub struct SystemDef {
    /// Kennung = Zielsystem (`data/targets/<id>.target.json`) = Ordnername unter `roms/`
    pub id: &'static str,
    pub label: &'static str,
    pub roles: &'static [RoleDef],
}

const fn r(id: &'static str, label: &'static str, sizes: &'static [usize], file: &'static str, required: bool, keywords: &'static [&'static str]) -> RoleDef {
    RoleDef { id, label, sizes, file, required, keywords }
}

pub static SYSTEMS: &[SystemDef] = &[
    SystemDef { id: "c64", label: "C64", roles: &[
        r("kernal", "KERNAL", &[8192], "kernal.bin", true, &["kernal", "kernel"]),
        r("basic", "BASIC", &[8192], "basic.bin", true, &["basic"]),
        r("chargen", "Zeichensatz", &[4096], "chargen.bin", true, &["chargen", "char"]),
        r("dos1541", "Laufwerk 1541", &[16384], "dos1541.bin", false, &["dos1541", "1541"]),
        r("dos1571", "Laufwerk 1571", &[32768], "dos1571.bin", false, &["dos1571", "1571"]),
        r("dos1581", "Laufwerk 1581", &[32768], "dos1581.bin", false, &["dos1581", "1581"]),
    ] },
    SystemDef { id: "c128", label: "C128", roles: &[
        r("kernal", "KERNAL", &[16384], "kernal.bin", true, &["kernal", "kernel"]),
        r("basiclo", "BASIC (niedrig)", &[16384], "basiclo.bin", true, &["basiclo"]),
        r("basichi", "BASIC (hoch)", &[16384], "basichi.bin", true, &["basichi"]),
        r("chargen", "Zeichensatz", &[8192], "chargen.bin", true, &["chargen", "char"]),
        r("kernal64", "KERNAL (C64-Betrieb)", &[8192], "kernal64.bin", false, &["kernal64"]),
        r("basic64", "BASIC (C64-Betrieb)", &[8192], "basic64.bin", false, &["basic64"]),
    ] },
    SystemDef { id: "vic20", label: "VC-20", roles: &[
        r("kernal", "KERNAL", &[8192], "kernal.bin", true, &["kernal", "kernel"]),
        r("basic", "BASIC", &[8192], "basic.bin", true, &["basic"]),
        r("chargen", "Zeichensatz", &[4096], "chargen.bin", true, &["chargen", "char"]),
    ] },
    SystemDef { id: "plus4", label: "Plus/4, C16", roles: &[
        r("kernal", "KERNAL", &[16384], "kernal.bin", true, &["kernal", "kernel"]),
        r("basic", "BASIC", &[16384], "basic.bin", true, &["basic"]),
        r("functionlo", "Funktions-ROM (niedrig)", &[16384], "functionlo.bin", false, &["functionlo", "3plus1-317053", "c2lo"]),
        r("functionhi", "Funktions-ROM (hoch)", &[16384], "functionhi.bin", false, &["functionhi", "3plus1-317054", "c2hi"]),
    ] },
    SystemDef { id: "atari8", label: "Atari 8 Bit", roles: &[
        // mindestens eines von os-xl / os-b / os-a nötig; "required" bleibt deshalb false, siehe `OS_ALTERNATIVES`
        r("os-xl", "Betriebssystem XL/XE", &[16384], "os-xl.rom", false, &["osxl", "os-xl", "atarixl", "xlxe", "xl-xe"]),
        r("os-b", "Betriebssystem 400/800 (B)", &[10240], "os-b.rom", false, &["osb", "os-b", "ataribos", "atariosb"]),
        r("os-a", "Betriebssystem 400/800 (A)", &[10240], "os-a.rom", false, &["osa", "os-a", "atariosa"]),
        r("basic", "BASIC", &[8192], "basic.rom", false, &["basic", "ataribas"]),
    ] },
    SystemDef { id: "mega65", label: "MEGA65", roles: &[
        r("rom", "System-ROM", &[131072], "mega65.rom", true, &["mega65"]),
    ] },
];

/// Ein Zeichensatz innerhalb eines ROM-Abbilds (`rom:c64/chargen#upper`)
#[derive(Debug, Clone, PartialEq)]
pub struct SetDef {
    /// Name im Verweis (`upper`, `lower`, `standard`, `international`, `vdc`)
    pub name: String,
    /// Anzeigetext
    pub label: String,
    pub offset: usize,
    /// Länge in Byte
    pub size: usize,
    /// Profil, zu dem die Bytes gehören (`c64.char.hires`, …)
    pub profile: String,
}

fn sd(name: &str, label: &str, offset: usize, size: usize, profile: &str) -> SetDef { SetDef { name: name.into(), label: label.into(), offset, size, profile: profile.into() } }

/// Zeichensätze, die in jedem Abbild einer Rolle am selben Ort liegen (gemessen an allen VICE-3.10-Abbildern bzw. dem Xformer-Satz).
/// Einträge in `known.json` (`sets`) haben Vorrang, z. B. beim MEGA65.
pub fn default_sets(system: &str, role: &str) -> Vec<SetDef> {
    let two = |profile: &str| vec![sd("upper", "Großbuchstaben und Grafik", 0, 2048, profile), sd("lower", "Groß-/Kleinschreibung", 2048, 2048, profile)];
    match (system, role) {
        ("c64", "chargen") => two("c64.char.hires"),
        ("vic20", "chargen") => two("vic20.char.hires"),
        ("c128", "chargen") => { let mut v = two("c128.char.hires"); v.push(sd("vdc", "80 Spalten (VDC, 8×16)", 0x1000, 4096, "c128.char.vdc")); v }
        ("plus4", "kernal") => vec![sd("standard", "Standard", 0x1000, 2048, "plus4.char.hires")],
        ("atari8", "os-xl") => vec![sd("standard", "Standard", 0x2000, 1024, "atari.char.antic2"), sd("international", "International", 0xC00, 1024, "atari.char.antic2")],
        // OS A und OS B (NTSC): Zeichensatz bei $E000 = Versatz $800, byte-gleich (gemessen)
        ("atari8", "os-a" | "os-b") => vec![sd("standard", "Standard", 0x800, 1024, "atari.char.antic2")],
        _ => vec![],
    }
}

/// Atari: Rolle `os` steht für die erste vorhandene dieser Rollen (Vorzugsfolge)
pub const OS_ALTERNATIVES: &[&str] = &["os-xl", "os-b", "os-a"];

pub fn system(id: &str) -> Option<&'static SystemDef> { SYSTEMS.iter().find(|s| s.id == id) }

pub fn role(system_id: &str, role_id: &str) -> Option<&'static RoleDef> { system(system_id)?.roles.iter().find(|r| r.id == role_id) }

/// Zielsystem-Kennung → ROM-Systemkennung (heute identisch; Zielvarianten wie `c16` zeigen auf `plus4`)
pub fn system_for_target(target: &str) -> Option<&'static SystemDef> {
    let t = match target { "c16" => "plus4", "atari" | "atari800" => "atari8", o => o };
    system(t)
}

/// Alle Größen, die irgendeine Rolle kennt (Dateien anderer Größe werden gar nicht erst eingelesen)
pub fn all_sizes() -> Vec<usize> {
    let mut v: Vec<usize> = SYSTEMS.iter().flat_map(|s| s.roles.iter().flat_map(|r| r.sizes.iter().copied())).collect();
    v.sort_unstable();
    v.dedup();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mega65_rom_hat_131072_byte() { assert_eq!(role("mega65", "rom").unwrap().sizes, &[131072]); }

    #[test]
    fn atari_rollen() {
        for id in OS_ALTERNATIVES { assert!(role("atari8", id).is_some(), "{id}"); }
        assert_eq!(system_for_target("atari").unwrap().id, "atari8");
        assert_eq!(system_for_target("c16").unwrap().id, "plus4");
        assert!(system_for_target("zx").is_none());
    }
}
