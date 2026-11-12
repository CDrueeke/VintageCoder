// SPDX-License-Identifier: GPL-3.0-or-later
//! Hardware-Datenbank: lädt CPU- und Target-Definitionen aus JSON.
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

mod bank;
pub use bank::{BankBit, BankPreset, BankRegion, BankRegister, Banking, Page, PageCategory, PageKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode { Imp, Acc, Imm, Zp, Zpx, Zpy, Abs, Abx, Aby, Ind, Izx, Izy, Rel,
    /// 45GS02: `#$nnnn`, `$zp,$rel` (BBR/BBS), 16-Bit-Sprung, `($zp),z`, `($zp,sp),y`, `($nnnn,x)`
    Imw, Zpr, Rl, Izz, Isy, Iax }

impl Mode {
    pub fn parse(s: &str) -> Option<Mode> {
        Some(match s {
            "imp" => Mode::Imp, "acc" => Mode::Acc, "imm" => Mode::Imm, "zp" => Mode::Zp, "zpx" => Mode::Zpx,
            "zpy" => Mode::Zpy, "abs" => Mode::Abs, "abx" => Mode::Abx, "aby" => Mode::Aby, "ind" => Mode::Ind,
            "izx" => Mode::Izx, "izy" => Mode::Izy, "rel" => Mode::Rel,
            "imw" => Mode::Imw, "zpr" => Mode::Zpr, "rl" => Mode::Rl, "izz" => Mode::Izz, "isy" => Mode::Isy, "iax" => Mode::Iax,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Op {
    pub mnemonic: String,
    pub mode: Mode,
    pub opcode: u8,
    pub bytes: u8,
    pub cycles: u8,
    pub page_cross: bool,
    pub branch: bool,
    pub affects: String,
    pub group: String,
}

#[derive(Debug, Clone)]
pub struct Cpu {
    pub id: String,
    pub features: Vec<String>,
    pub ops: Vec<Op>,
    /// 45GS02: Präfix für 32-Bit-Zeiger (`[$nn],z`, Standard $EA) und Q-Register-Präfix (`NEG NEG`)
    pub nop32: Option<u8>,
    pub quad_prefix: Vec<u8>,
    /// Q-Mnemonic → Grundbefehl (LDQ → LDA)
    pub quad_ops: Vec<(String, String)>,
    by_name: HashMap<(String, Mode), usize>,
}

impl Cpu {
    /// Kanonischer Opcode für (Mnemonic, Modus): offiziell vor undokumentiert, dann niedrigster Opcode.
    pub fn lookup(&self, mnemonic: &str, mode: Mode) -> Option<&Op> {
        self.by_name.get(&(mnemonic.to_string(), mode)).map(|&i| &self.ops[i])
    }
    pub fn by_opcode(&self, code: u8) -> Option<&Op> {
        self.ops.iter().find(|o| o.opcode == code)
    }
    /// Grundbefehl eines Q-Mnemonics (`LDQ` → `LDA`), falls die CPU Q-Befehle hat
    pub fn quad_base(&self, mnemonic: &str) -> Option<&str> { self.quad_ops.iter().find(|(q, _)| q == mnemonic).map(|(_, b)| b.as_str()) }
    pub fn has_mnemonic(&self, mnemonic: &str) -> bool {
        self.ops.iter().any(|o| o.mnemonic == mnemonic)
    }
    /// Ist die Gruppe (`official`, `illegal.stable`, …) für diese CPU vorhanden?
    pub fn has_group(&self, group: &str) -> bool {
        group == "official" || self.features.iter().any(|f| f == group)
    }
    pub fn canonical_ops(&self) -> impl Iterator<Item = &Op> {
        let mut idx: Vec<usize> = self.by_name.values().copied().collect();
        idx.sort();
        idx.into_iter().map(move |i| &self.ops[i])
    }
    fn index(&mut self) {
        fn rank(g: &str) -> u8 { match g { "official" => 0, "illegal.stable" => 1, _ => 2 } }
        self.by_name.clear();
        for (i, op) in self.ops.iter().enumerate() {
            let key = (op.mnemonic.clone(), op.mode);
            match self.by_name.get(&key) {
                Some(&j) if rank(&self.ops[j].group) <= rank(&op.group) => {}
                _ => { self.by_name.insert(key, i); }
            }
        }
    }
}

#[derive(Deserialize)]
struct CpuFile {
    id: String,
    #[serde(default)]
    extends: Option<String>,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    ops: Vec<Vec<Value>>,
    #[serde(default, rename = "overrideOps")]
    override_ops: Vec<Vec<Value>>,
    #[serde(default)]
    prefixes: HashMap<String, String>,
    #[serde(default, rename = "quadOps")]
    quad_ops: HashMap<String, String>,
}

pub fn parse_num(s: &str) -> Option<i64> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix('$').or_else(|| s.strip_prefix("0x")) { i64::from_str_radix(h, 16).ok() }
    else if let Some(b) = s.strip_prefix('%') { i64::from_str_radix(b, 2).ok() }
    else { s.parse().ok() }
}

fn parse_row(row: &[Value]) -> Result<Op, String> {
    let s = |i: usize| row.get(i).and_then(Value::as_str).ok_or_else(|| format!("Spalte {i} fehlt"));
    let n = |i: usize| row.get(i).and_then(Value::as_u64).ok_or_else(|| format!("Spalte {i} fehlt"));
    let mode = Mode::parse(s(1)?).ok_or_else(|| format!("unbekannter Modus {}", s(1).unwrap_or("?")))?;
    let flags = s(5)?;
    Ok(Op {
        mnemonic: s(0)?.to_string(), mode,
        opcode: parse_num(s(2)?).ok_or("Opcode ungültig")? as u8,
        bytes: n(3)? as u8, cycles: n(4)? as u8,
        page_cross: flags.contains('p'), branch: flags.contains('b'), affects: s(6)?.to_string(),
        group: s(7)?.to_string(),
    })
}

pub fn load_cpu(data_dir: &Path, id: &str) -> Result<Cpu, String> {
    let path = data_dir.join("cpus").join(format!("{id}.cpu.json"));
    let text = vc_core::read_static(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file: CpuFile = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut cpu = match &file.extends {
        Some(parent) => load_cpu(data_dir, parent)?,
        None => Cpu { id: String::new(), features: vec![], ops: vec![], nop32: None, quad_prefix: vec![], quad_ops: vec![], by_name: HashMap::new() },
    };
    cpu.id = file.id;
    let bytes = |s: &str| -> Vec<u8> { s.split_whitespace().filter_map(|b| parse_num(b).map(|v| v as u8)).collect() };
    if let Some(p) = file.prefixes.get("nop32") { cpu.nop32 = bytes(p).first().copied(); }
    if let Some(p) = file.prefixes.get("quad") { cpu.quad_prefix = bytes(p); }
    let mut q: Vec<(String, String)> = file.quad_ops.into_iter().collect(); q.sort(); cpu.quad_ops.extend(q);
    cpu.features.extend(file.features);
    for row in file.ops.iter().chain(file.override_ops.iter()) {
        let op = parse_row(row).map_err(|e| format!("{}: {e}", path.display()))?;
        match cpu.ops.iter().position(|o| o.opcode == op.opcode) {
            Some(i) => cpu.ops[i] = op,
            None => cpu.ops.push(op),
        }
    }
    cpu.index();
    Ok(cpu)
}

#[derive(Debug, Clone)]
pub struct Target {
    pub id: String,
    pub cpu: String,
    pub chips: Vec<String>,
    pub basic_load_addr: Option<u16>,
    pub basic_dialect: Option<String>,
    /// Standard-Ausgabeformat (erster Eintrag unter `outputs`), z. B. `prg` oder `xex`.
    pub default_format: String,
    /// Textcode des Systems: `petscii` (Commodore, Standard) oder `atari`
    pub text_encoding: String,
    /// Speicherausbauten (VC-20): Vorgabe und Liste; leer bei Systemen ohne Ausbaustufen
    pub memory_default: Option<String>,
    pub memory: Vec<MemConfig>,
    /// Gewählter Ausbau (nach [`Target::apply_memory`])
    pub memory_sel: Option<String>,
    /// Bank-Presets (Speicherumschaltung); `None`, wenn das Target keine hat
    pub banking: Option<Banking>,
    /// Adresse des Postfachs der Tests (Postfach-Modus, 8 Byte); `None`, wo keine frei belegbare Stelle festgelegt ist
    pub test_mailbox: Option<u16>,
    /// Adresse des Debug-Registers der Ultimate 64 (`$D7FF`); `test_end` meldet dort das Ergebnis, falls das Ziel es erlaubt (nur C64)
    pub test_debug_reg: Option<u16>,
}

/// Eine Speicherausbaustufe eines Targets (z. B. VC-20: Grundgerät, +3 KB, +8 KB …)
#[derive(Debug, Clone, PartialEq)]
pub struct MemConfig {
    pub id: String,
    pub label: String,
    pub basic_load: Option<u16>,
    pub screen: Option<u16>,
    pub color_ram: Option<u16>,
    /// Argument für VICE `-memory`
    pub vice: String,
}

impl Target {
    /// Seitenplan (256 Seiten) des Presets `preset`; `None` ohne Banking oder bei unbekannter Kennung
    pub fn page_plan(&self, preset: &str) -> Option<[Page; 256]> {
        let b = self.banking.as_ref()?;
        Some(b.page_plan(b.preset(preset)?))
    }
    /// Stellt den Speicherausbau ein (`None` oder leer = Vorgabe); setzt die BASIC-Startadresse. Fehler bei unbekanntem Namen.
    pub fn apply_memory(&mut self, id: Option<&str>) -> Result<(), String> {
        if self.memory.is_empty() { return Ok(()); }
        let want = id.filter(|i| !i.is_empty()).or(self.memory_default.as_deref()).unwrap_or(&self.memory[0].id).to_string();
        let c = self.memory.iter().find(|c| c.id == want)
            .ok_or_else(|| format!("unbekannter Speicherausbau »{want}« (möglich: {})", self.memory.iter().map(|c| c.id.as_str()).collect::<Vec<_>>().join(", ")))?;
        if c.basic_load.is_some() { self.basic_load_addr = c.basic_load; }
        self.memory_sel = Some(c.id.clone());
        Ok(())
    }
    /// Vordefinierte Symbole des Speicherausbaus für den Assembler (`MEM_SCREEN_RAM`, `MEM_COLOR_RAM`, `MEM_BASIC_START`); leer ohne Ausbaustufen
    pub fn memory_symbols(&self) -> Vec<(String, i64)> {
        let Some(m) = self.memory_config() else { return vec![] };
        [("MEM_SCREEN_RAM", m.screen), ("MEM_COLOR_RAM", m.color_ram), ("MEM_BASIC_START", m.basic_load)].into_iter()
            .filter_map(|(n, v)| v.map(|v| (n.to_string(), v as i64))).collect()
    }
    /// Vordefinierte Symbole für Testprogramme: `TEST_MAILBOX` (Adresse des Postfachs), leer ohne Postfach
    pub fn test_symbols(&self) -> Vec<(String, i64)> {
        let mut v: Vec<(String, i64)> = self.test_mailbox.map(|a| vec![("TEST_MAILBOX".to_string(), a as i64)]).unwrap_or_default();
        if let Some(a) = self.test_debug_reg { v.push(("TEST_DEBUGREG".to_string(), a as i64)); }
        v
    }
    /// Der gewählte Speicherausbau (Vorgabe, solange `apply_memory` nicht aufgerufen wurde)
    pub fn memory_config(&self) -> Option<&MemConfig> {
        let id = self.memory_sel.as_deref().or(self.memory_default.as_deref())?;
        self.memory.iter().find(|c| c.id == id).or(self.memory.first())
    }
}

/// Meldung für ein Zielsystem, dessen Datei in diesem Release fehlt (kein Absturz, kein Dateipfad)
pub fn missing_message(id: &str) -> String { format!("Zielsystem „{id}“ ist in diesem Release nicht enthalten") }

/// Ist die Target-Datei des Systems vorhanden (also im Release geliefert)?
pub fn target_available(data_dir: &Path, id: &str) -> bool {
    !id.is_empty() && !id.contains(['/', '\\']) && data_dir.join("targets").join(format!("{id}.target.json")).is_file()
}

/// Kennungen aller gelieferten Targets (Dateien `data/targets/<id>.target.json`), sortiert
pub fn available_targets(data_dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(data_dir.join("targets")).into_iter().flatten().flatten()
        .filter_map(|e| e.file_name().to_string_lossy().strip_suffix(".target.json").map(String::from)).collect();
    v.sort();
    v
}

/// Vorgabe-Target für neue Projekte und Rückfälle: C64, wenn geliefert, sonst das erste gelieferte (Hauptsysteme in fester
/// Reihenfolge, danach alphabetisch); `None`, wenn kein Target vorhanden ist.
pub fn default_target(data_dir: &Path) -> Option<String> {
    let all = available_targets(data_dir);
    ["c64", "c128", "plus4", "c16", "vic20", "atari8", "mega65"].iter().find(|id| all.iter().any(|a| a == *id)).map(|s| s.to_string()).or_else(|| all.into_iter().next())
}

/// Wie [`default_target`]; ohne Datenordner oder ohne Target `"c64"` (die spätere Fehlermeldung nennt dann das fehlende Target)
pub fn default_target_or_c64(data_dir: Option<&Path>) -> String { data_dir.and_then(default_target).unwrap_or_else(|| "c64".into()) }

pub fn load_target(data_dir: &Path, id: &str) -> Result<Target, String> {
    let path = data_dir.join("targets").join(format!("{id}.target.json"));
    if !path.exists() { return Err(missing_message(id)); }
    let text = vc_core::read_static(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let t = &v["target"];
    let banking = match v.get("banking") {
        Some(b) => {
            let b: Banking = serde_json::from_value(b.clone()).map_err(|e| format!("{}: banking: {e}", path.display()))?;
            b.validate().map_err(|e| format!("{}: banking: {e}", path.display()))?;
            Some(b)
        }
        None => None,
    };
    Ok(Target {
        banking,
        test_mailbox: t["testMailbox"].as_str().and_then(parse_num).filter(|a| (0..=0xFFF8).contains(a)).map(|a| a as u16),
        test_debug_reg: t["testDebugReg"].as_str().and_then(parse_num).filter(|a| (0..=0xFFFF).contains(a)).map(|a| a as u16),
        id: t["id"].as_str().ok_or("target.id fehlt")?.into(),
        cpu: t["cpu"].as_str().ok_or("target.cpu fehlt")?.into(),
        chips: t["chips"].as_array().map(|a| a.iter().filter_map(|c| c.as_str().map(String::from)).collect()).unwrap_or_default(),
        basic_load_addr: v["basic"]["loadAddr"].as_str().and_then(parse_num).map(|a| a as u16),
        basic_dialect: v["basic"]["dialect"].as_str().map(String::from),
        default_format: v["outputs"][0]["format"].as_str().unwrap_or("bin").into(),
        text_encoding: t["textEncoding"].as_str().unwrap_or("petscii").into(),
        memory_sel: None,
        memory_default: v["memory"]["default"].as_str().map(String::from),
        memory: v["memory"]["configs"].as_object().map(|m| m.iter().map(|(id, c)| {
            let a = |k: &str| c[k].as_str().and_then(parse_num).map(|n| n as u16);
            MemConfig { id: id.clone(), label: c["label"].as_str().unwrap_or(id).into(), basic_load: a("basicLoad"), screen: a("screen"), color_ram: a("colorRam"), vice: c["vice"].as_str().unwrap_or("").into() }
        }).collect()).unwrap_or_default(),
    })
}

#[derive(Debug, Clone)]
pub struct Reg {
    pub name: String,
    pub addr: u32,
    pub access: String,
    pub doc: String,
}

#[derive(Debug, Clone)]
pub struct Chip {
    pub id: String,
    pub name: String,
    pub regs: Vec<Reg>,
}

pub fn load_chip(data_dir: &Path, id: &str) -> Result<Chip, String> {
    let path = data_dir.join("chips").join(format!("{id}.chip.json"));
    let text = vc_core::read_static(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let regs = v["regs"].as_array().ok_or("regs fehlt")?.iter().map(|r| {
        Ok(Reg {
            name: r["name"].as_str().ok_or("reg.name fehlt")?.into(),
            addr: r["addr"].as_str().and_then(parse_num).ok_or("reg.addr ungültig")? as u32,
            access: r["access"].as_str().unwrap_or("rw").into(),
            doc: r["doc"].as_str().unwrap_or("").into(),
        })
    }).collect::<Result<Vec<_>, String>>().map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Chip { id: v["id"].as_str().unwrap_or(id).into(), name: v["name"].as_str().unwrap_or(id).into(), regs })
}

/// Assembler-Konstanten des Projekt-Targets `tid` aus `"defines": { "NAME": Zahl }` in der `.vcprj` im Ordner `root`
/// (wie `vcc asm -D NAME=Zahl`, z. B. `DEBUG` für `.if DEBUG`); `true`/`false` zählen als 1/0. Ohne Projektdatei oder Eintrag: leer.
pub fn project_defines(root: &Path, tid: &str) -> Vec<(String, i64)> {
    let Some(file) = std::fs::read_dir(root).ok().and_then(|rd| rd.flatten().map(|e| e.path()).find(|p| p.extension().map_or(false, |e| e == "vcprj"))) else { return vec![] };
    let Some(v) = std::fs::read_to_string(file).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) else { return vec![] };
    let Some(t) = v["targets"].as_array().and_then(|a| a.iter().find(|t| t["id"] == tid || t["system"] == tid)) else { return vec![] };
    t["defines"].as_object().map(|m| m.iter().filter_map(|(k, x)| x.as_i64().or_else(|| x.as_bool().map(i64::from)).map(|n| (k.clone(), n))).collect()).unwrap_or_default()
}

/// Alle Chip-Register eines Targets als (`chip.reg`, Adresse) – vordefinierte Symbole für den Assembler.
pub fn chip_symbols(data_dir: &Path, target: &Target) -> Result<Vec<(String, i64)>, String> {
    let mut v = vec![];
    for c in &target.chips {
        let chip = load_chip(data_dir, c)?;
        v.extend(chip.regs.iter().map(|r| (format!("{}.{}", chip.id, r.name), r.addr as i64)));
    }
    Ok(v)
}

/// Erzeugt eine Include-Datei (`.const chip.reg = $addr`) für alle Chips eines Targets.
pub fn generate_include(data_dir: &Path, target: &Target) -> Result<String, String> {
    let mut s = format!("; Automatisch erzeugt aus data/chips/ für Target '{}' – nicht von Hand ändern.\n", target.id);
    for c in &target.chips {
        let chip = load_chip(data_dir, c)?;
        s.push_str(&format!("\n; {} ({})\n", chip.name, chip.id));
        for r in &chip.regs {
            let rw = match r.access.as_str() { "r" => "lesen", "w" => "schreiben", _ => "lesen/schreiben" };
            s.push_str(&format!(".const {}.{} = ${:04x}   ; {} ({})\n", chip.id, r.name, r.addr, r.doc, rw));
        }
    }
    Ok(s)
}

#[cfg(test)]
mod release_tests {
    use super::*;
    fn tmp(ids: &[&str]) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("vc-target-rel-{}-{}", std::process::id(), ids.join("_")));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("targets")).unwrap();
        for id in ids { std::fs::write(d.join("targets").join(format!("{id}.target.json")), "{}").unwrap(); }
        d
    }
    #[test]
    fn default_ist_c64_sonst_erstes_geliefertes() {
        assert_eq!(default_target(&tmp(&["atari8", "c64"])).as_deref(), Some("c64"));
        assert_eq!(default_target(&tmp(&["mega65", "atari8"])).as_deref(), Some("atari8"));
        assert_eq!(default_target(&tmp(&["pet"])).as_deref(), Some("pet"));
        assert_eq!(default_target(&tmp(&[])), None);
        assert_eq!(default_target_or_c64(None), "c64");
    }
    #[test]
    fn fehlendes_target_gibt_meldung_statt_pfad() {
        let d = tmp(&["atari8"]);
        assert!(target_available(&d, "atari8") && !target_available(&d, "c64") && !target_available(&d, "../x"));
        let e = load_target(&d, "c64").err().unwrap();
        assert!(e.contains("in diesem Release nicht enthalten") && !e.contains('/'), "{e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn data() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }
    fn plan(target: &str, preset: &str) -> [Page; 256] {
        load_target(&data(), target).unwrap().page_plan(preset).unwrap_or_else(|| panic!("{target}: Preset {preset} fehlt"))
    }
    fn all(p: &[Page; 256], from: u8, to: u8, kind: PageKind) -> bool { p[from as usize..=to as usize].iter().all(|x| x.kind == kind) }

    #[test]
    fn c64_standard() {
        let p = plan("c64", "37");
        assert!(all(&p, 0x00, 0x9F, PageKind::Ram));
        assert!(all(&p, 0xA0, 0xBF, PageKind::BasicRom));
        assert!(all(&p, 0xC0, 0xCF, PageKind::Ram));
        assert!(all(&p, 0xD0, 0xDF, PageKind::Io));
        assert!(all(&p, 0xE0, 0xFF, PageKind::KernalRom));
        assert_eq!(p[0].category, PageCategory::Zeropage);
        assert_eq!(p[1].category, PageCategory::Stack);
        assert_eq!(p[0xA0].category, PageCategory::Basic);
        assert_eq!(p[0xD0].category, PageCategory::Io);
        assert_eq!(p[0x10].category, PageCategory::Free);
    }

    #[test]
    fn c64_alle_belegungen_nach_pla() {
        let t = load_target(&data(), "c64").unwrap();
        let b = t.banking.as_ref().unwrap();
        assert_eq!(b.register.addr, 1);
        assert_eq!(b.presets.len(), 8);
        for pr in &b.presets {
            let (loram, hiram, charen) = (pr.value & 1 != 0, pr.value & 2 != 0, pr.value & 4 != 0);
            let p = b.page_plan(pr);
            let basic = if loram && hiram { PageKind::BasicRom } else { PageKind::Ram };
            let kernal = if hiram { PageKind::KernalRom } else { PageKind::Ram };
            let d000 = if !loram && !hiram { PageKind::Ram } else if charen { PageKind::Io } else { PageKind::CharRom };
            assert!(all(&p, 0xA0, 0xBF, basic), "{} A000", pr.id);
            assert!(all(&p, 0xD0, 0xDF, d000), "{} D000", pr.id);
            assert!(all(&p, 0xE0, 0xFF, kernal), "{} E000", pr.id);
            assert!(all(&p, 0x00, 0x9F, PageKind::Ram) && all(&p, 0xC0, 0xCF, PageKind::Ram), "{} RAM", pr.id);
        }
    }

    #[test]
    fn c64_alles_ram() {
        for id in ["30", "34"] {
            let p = plan("c64", id);
            assert!(all(&p, 0, 0xFF, PageKind::Ram), "Preset {id}");
        }
    }

    #[test]
    fn c64_zeichen_rom() {
        let p = plan("c64", "31");
        assert!(all(&p, 0xD0, 0xDF, PageKind::CharRom));
        assert!(all(&p, 0xE0, 0xFF, PageKind::Ram));
    }

    #[test]
    fn c128_konfigurationen() {
        let p = plan("c128", "00");
        assert!(all(&p, 0x00, 0x3F, PageKind::Ram));
        assert!(all(&p, 0x40, 0xAF, PageKind::BasicRom));
        assert_eq!(p[0xB0].kind, PageKind::Rom);
        assert!(all(&p, 0xC0, 0xCF, PageKind::KernalRom));
        assert!(all(&p, 0xD0, 0xDF, PageKind::Io));
        assert!(all(&p, 0xE0, 0xFF, PageKind::KernalRom));
        assert_eq!(plan("c128", "01")[0xD0].kind, PageKind::CharRom);
        let p = plan("c128", "3e");
        assert!(all(&p, 0xD0, 0xDF, PageKind::Io));
        assert!(all(&p, 0x00, 0xCF, PageKind::Ram) && all(&p, 0xE0, 0xFF, PageKind::Ram));
        assert!(all(&plan("c128", "3f"), 0, 0xFF, PageKind::Ram));
        assert!(all(&plan("c128", "7f"), 0, 0xFF, PageKind::Ram));
        assert_eq!(plan("c128", "7e")[0xD5].kind, PageKind::Io);
        // die Werte der Konstanten in data/sys/c128/hardware.asm
        let t = load_target(&data(), "c128").unwrap();
        for v in [0x00u8, 0x3E, 0x3F, 0x7E, 0x7F] { assert!(t.banking.as_ref().unwrap().preset_for_value(v).is_some(), "${v:02X}"); }
    }

    #[test]
    fn atari_portb() {
        let p = plan("atari8", "ff");
        assert!(all(&p, 0x00, 0xBF, PageKind::Ram));
        assert!(all(&p, 0xC0, 0xCF, PageKind::KernalRom));
        assert!(all(&p, 0xD0, 0xD7, PageKind::Io));
        assert!(all(&p, 0xD8, 0xFF, PageKind::KernalRom));
        assert!(all(&plan("atari8", "fd"), 0xA0, 0xBF, PageKind::BasicRom));
        assert_eq!(plan("atari8", "7f")[0x50].kind, PageKind::Rom);
        assert_eq!(p[0x50].kind, PageKind::Ram);
        let p = plan("atari8", "fe");
        assert!(all(&p, 0xC0, 0xCF, PageKind::Ram) && all(&p, 0xD8, 0xFF, PageKind::Ram));
        assert_eq!(p[0xD3].kind, PageKind::Io);
        assert!(all(&plan("atari8", "ef"), 0x40, 0x7F, PageKind::Ram));
    }

    #[test]
    fn atari_modellfilter() {
        let t = load_target(&data(), "atari8").unwrap();
        let b = t.banking.as_ref().unwrap();
        assert!(b.presets_for_model(Some("800xl")).all(|p| p.value & 0x10 != 0));
        assert!(b.presets_for_model(Some("130xe")).any(|p| p.value & 0x10 == 0));
        assert!(b.presets_for_model(Some("800")).next().is_none());
        assert_eq!(b.default_preset().unwrap().value, 0xFF);
    }

    #[test]
    fn ohne_banking_bleibt_none() {
        let t = load_target(&data(), "vic20").unwrap();
        assert!(t.banking.is_none() && t.page_plan("37").is_none());
    }

    #[test]
    fn alle_targets_laden_und_presets_stimmig() {
        for id in ["c64", "c128", "atari8", "plus4", "c16", "vic20", "mega65"] {
            let t = load_target(&data(), id).unwrap();
            if let Some(b) = &t.banking {
                assert!(b.default_preset().is_some(), "{id}");
                for pr in &b.presets { assert_eq!(b.preset_for_value(pr.value).map(|q| q.value), Some(pr.value)); }
            }
        }
    }
}
