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

impl RoleDef {
    /// Anzeigename der Rolle in der aktuellen Sprache (Katalog); `label` bleibt der deutsche Text
    pub fn label_l(&self) -> String {
        match self.id {
            "kernal" => vc_i18n::t!("roms-role-kernal"), "basic" => vc_i18n::t!("roms-role-basic"), "chargen" => vc_i18n::t!("roms-role-chargen"),
            "dos1541" => vc_i18n::t!("roms-role-dos1541"), "dos1571" => vc_i18n::t!("roms-role-dos1571"), "dos1581" => vc_i18n::t!("roms-role-dos1581"),
            "basiclo" => vc_i18n::t!("roms-role-basiclo"), "basichi" => vc_i18n::t!("roms-role-basichi"), "kernal64" => vc_i18n::t!("roms-role-kernal64"), "basic64" => vc_i18n::t!("roms-role-basic64"),
            "functionlo" => vc_i18n::t!("roms-role-functionlo"), "functionhi" => vc_i18n::t!("roms-role-functionhi"),
            "os-xl" => vc_i18n::t!("roms-role-os-xl"), "os-b" => vc_i18n::t!("roms-role-os-b"), "os-a" => vc_i18n::t!("roms-role-os-a"), "rom" => vc_i18n::t!("roms-role-rom"),
            "basic-b000" => vc_i18n::t!("roms-role-basic-b000"), "basic-c000" => vc_i18n::t!("roms-role-basic-c000"), "basic-d000" => vc_i18n::t!("roms-role-basic-d000"), "editor" => vc_i18n::t!("roms-role-editor"),
            "chip-c000" => vc_i18n::t!("roms-role-chip-c000"), "chip-c800" => vc_i18n::t!("roms-role-chip-c800"), "chip-d000" => vc_i18n::t!("roms-role-chip-d000"),
            "chip-d800" => vc_i18n::t!("roms-role-chip-d800"), "chip-f000" => vc_i18n::t!("roms-role-chip-f000"), "chip-f800" => vc_i18n::t!("roms-role-chip-f800"),
            _ => self.label.to_string(),
        }
    }
}

pub struct SystemDef {
    /// Kennung = Zielsystem (`data/targets/<id>.target.json`) = Ordnername unter `roms/`
    pub id: &'static str,
    pub label: &'static str,
    pub roles: &'static [RoleDef],
}

impl SystemDef {
    /// Anzeigename des Systems in der aktuellen Sprache (Katalog); `label` bleibt der deutsche Text
    pub fn label_l(&self) -> String {
        match self.id {
            "c64" => vc_i18n::t!("roms-system-c64"), "c128" => vc_i18n::t!("roms-system-c128"), "vic20" => vc_i18n::t!("roms-system-vic20"),
            "plus4" => vc_i18n::t!("roms-system-plus4"), "atari8" => vc_i18n::t!("roms-system-atari8"), "mega65" => vc_i18n::t!("roms-system-mega65"), "pet" => vc_i18n::t!("roms-system-pet"),
            _ => self.label.to_string(),
        }
    }
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
    // PET/CBM: die ROMs liegen als einzelne Chips; Rollen nach Speicherplatz. BASIC 2/4 in 4-KB-Chips, die Urfassungen (ROM 1 bis 3)
    // in 2-KB-Chips (`chip-*`). Editor ($E000) und KERNAL ($F000) gibt es je Revision einmal; die Revision wählt der Anwender (Standard: BASIC 4, 40 Spalten).
    SystemDef { id: "pet", label: "PET/CBM", roles: &[
        r("basic-b000", "BASIC ($B000)", &[4096], "basic-b000.bin", false, &["b000"]),
        r("basic-c000", "BASIC ($C000)", &[4096], "basic-c000.bin", false, &["basic-c000"]),
        r("basic-d000", "BASIC ($D000)", &[4096], "basic-d000.bin", false, &["basic-d000"]),
        r("editor", "Bildschirm-Editor", &[2048], "editor.bin", false, &["edit"]),
        r("kernal", "KERNAL", &[4096], "kernal.bin", false, &["kernal", "kernel"]),
        r("chargen", "Zeichensatz", &[2048], "chargen.bin", false, &["chargen", "char"]),
        r("chip-c000", "Chip $C000 (2 KB)", &[2048], "chip-c000.bin", false, &["c000"]),
        r("chip-c800", "Chip $C800 (2 KB)", &[2048], "chip-c800.bin", false, &["c800"]),
        r("chip-d000", "Chip $D000 (2 KB)", &[2048], "chip-d000.bin", false, &["d000"]),
        r("chip-d800", "Chip $D800 (2 KB)", &[2048], "chip-d800.bin", false, &["d800"]),
        r("chip-f000", "Chip $F000 (2 KB)", &[2048], "chip-f000.bin", false, &["f000"]),
        r("chip-f800", "Chip $F800 (2 KB)", &[2048], "chip-f800.bin", false, &["f800"]),
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

impl SetDef {
    /// Größe in Byte, wie sie das Profil erwartet: die ROM-Sätze des TED (Plus/4, C16) haben nur 128 Zeichen, die Invers-Hälfte erzeugt der Chip.
    pub fn expanded_size(&self) -> usize { if self.profile.ends_with(".char.hires") && self.size == 1024 { 2048 } else { self.size } }

    /// ROM-Bytes des Satzes auf die Größe des Profils bringen: bei 128-Zeichen-Sätzen folgt die Invers-Hälfte (alle Bits umgekehrt).
    pub fn expand(&self, mut bytes: Vec<u8>) -> Vec<u8> {
        if self.expanded_size() != self.size { let inv: Vec<u8> = bytes.iter().map(|b| !b).collect(); bytes.extend(inv); }
        bytes
    }
}

fn sd(name: &str, label: &str, offset: usize, size: usize, profile: &str) -> SetDef { SetDef { name: name.into(), label: label.into(), offset, size, profile: profile.into() } }

/// Zeichensätze, die in jedem Abbild einer Rolle am selben Ort liegen (gemessen an allen VICE-3.10-Abbildern bzw. dem Xformer-Satz).
/// Einträge in `known.json` (`sets`) haben Vorrang, z. B. beim MEGA65.
pub fn default_sets(system: &str, role: &str) -> Vec<SetDef> {
    let upper = || vc_i18n::t!("roms-set-upper");
    let lower = || vc_i18n::t!("roms-set-lower");
    let two = |profile: &str| vec![sd("upper", &upper(), 0, 2048, profile), sd("lower", &lower(), 2048, 2048, profile)];
    let two_half = |profile: &str| vec![sd("upper", &upper(), 0, 1024, profile), sd("lower", &lower(), 1024, 1024, profile)];
    match (system, role) {
        ("c64", "chargen") => two("c64.char.hires"),
        ("vic20", "chargen") => two("vic20.char.hires"),
        ("c128", "chargen") => { let mut v = two("c128.char.hires"); v.push(sd("vdc", &vc_i18n::t!("roms-set-vdc"), 0x1000, 4096, "c128.char.vdc")); v }
        // Plus/4, C16: Zeichen-ROM $D000 bis $D7FF = zwei Sätze zu je 128 Zeichen (1 KB); die Invers-Hälfte erzeugt der TED (Umkehrbit), siehe `SetDef::expanded_size`.
        // Reihenfolge gemessen (2026-11-17) an den VICE-3.10-Abbildern 318004-01, 318004-05, 318005-05 und 364: erster Teil Groß/Grafik, zweiter Klein/Groß.
        ("plus4", "kernal") => vec![sd("upper", &upper(), 0x1000, 1024, "plus4.char.hires"), sd("lower", &lower(), 0x1400, 1024, "plus4.char.hires")],
        // PET/CBM: 128 Zeichen je Satz (1 KB), die Invers-Hälfte erzeugt der Chip (wie beim TED); erster Satz Groß/Grafik, zweiter Klein/Groß
        ("pet", "chargen") => two_half("pet.char.hires"),
        ("atari8", "os-xl") => vec![sd("standard", &vc_i18n::t!("roms-set-standard"), 0x2000, 1024, "atari.char.antic2"), sd("international", &vc_i18n::t!("roms-set-international"), 0xC00, 1024, "atari.char.antic2")],
        // OS A und OS B (NTSC): Zeichensatz bei $E000 = Versatz $800, byte-gleich (gemessen)
        ("atari8", "os-a" | "os-b") => vec![sd("standard", &vc_i18n::t!("roms-set-standard"), 0x800, 1024, "atari.char.antic2")],
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
    fn plus4_hat_zwei_saetze_zu_128_zeichen_und_erzeugt_die_invers_haelfte() {
        let s = default_sets("plus4", "kernal");
        assert_eq!(s.iter().map(|x| (x.name.as_str(), x.offset, x.size)).collect::<Vec<_>>(), [("upper", 0x1000, 1024), ("lower", 0x1400, 1024)]);
        assert_eq!(s[0].expanded_size(), 2048);
        let b = s[0].expand(vec![0x18; 1024]);
        assert_eq!((b.len(), b[0], b[1024]), (2048, 0x18, 0xE7));
        // C64-Sätze (2048 Byte) bleiben unverändert
        let c = &default_sets("c64", "chargen")[0];
        assert_eq!(c.expand(vec![1; 2048]).len(), 2048);
    }

    #[test]
    fn pet_hat_chip_rollen_und_zeichensatz_mit_zwei_halben_saetzen() {
        for (id, size) in [("basic-b000", 4096), ("basic-c000", 4096), ("basic-d000", 4096), ("kernal", 4096), ("editor", 2048), ("chargen", 2048), ("chip-c000", 2048), ("chip-f800", 2048)] {
            assert_eq!(role("pet", id).unwrap().sizes, &[size], "{id}");
        }
        assert_eq!(system_for_target("pet").unwrap().id, "pet");
        let s = default_sets("pet", "chargen");
        assert_eq!(s.iter().map(|x| (x.name.as_str(), x.offset, x.size)).collect::<Vec<_>>(), [("upper", 0, 1024), ("lower", 1024, 1024)]);
        assert_eq!(s[1].expand(vec![0x18; 1024]).len(), 2048);
    }

    #[test]
    fn mega65_rom_hat_131072_byte() { assert_eq!(role("mega65", "rom").unwrap().sizes, &[131072]); }

    #[test]
    fn atari_rollen() {
        for id in OS_ALTERNATIVES { assert!(role("atari8", id).is_some(), "{id}"); }
        assert_eq!(system_for_target("atari").unwrap().id, "atari8");
        assert_eq!(system_for_target("c16").unwrap().id, "plus4");
        assert!(system_for_target("zx").is_none());
    }

    #[test]
    fn texte_in_beiden_sprachen() {
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::En));
        assert_eq!(default_sets("c64", "chargen")[0].label, "Uppercase and graphics");
        assert_eq!(role("c64", "dos1541").unwrap().label_l(), "Drive 1541");
        assert_eq!(system("vic20").unwrap().label_l(), "VIC-20");
        assert_eq!(crate::store::Status::Missing.label_l(), "Missing");
        assert_eq!(crate::env::Stage::User.label_l(), "User folder");
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::De));
        assert_eq!(default_sets("c64", "chargen")[0].label, "Großbuchstaben und Grafik");
        assert_eq!(role("c64", "dos1541").unwrap().label_l(), "Laufwerk 1541");
        assert_eq!(system("vic20").unwrap().label_l(), "VC-20");
        assert_eq!(crate::store::Status::Missing.label_l(), "Fehlt");
    }
}
