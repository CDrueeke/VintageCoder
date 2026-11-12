// SPDX-License-Identifier: GPL-3.0-or-later
//! Modulformate: VC-20 Rohabbilder (`.a0`, `.20`, `.40`, `.60`), C64 `.crt` (normal 8K/16K, Ocean, Magic Desk, EasyFlash, Action Replay, Ultimax) und Atari `.car` / `.rom` (8K, 16K, XEGS 32K/64K/128K).
//! Die Bankzuordnung kommt aus `.bank n` und `.org`; ungenutzte Bereiche werden mit $FF aufgefüllt.
use crate::{Output, Segment};
use std::collections::BTreeMap;

const FILL: u8 = 0xFF;

/// Bytes der Bank `bank` im Fenster `lo..=hi` (Länge hi-lo+1); `None`, wenn die Bank dort nichts enthält.
fn window(segs: &[Segment], bank: u32, lo: usize, hi: usize) -> Result<Option<Vec<u8>>, String> {
    let mut img = vec![FILL; hi - lo + 1]; let mut used = false;
    for s in segs.iter().filter(|s| s.bank == bank) {
        let (a, b) = (s.start as usize, s.start as usize + s.bytes.len());
        if s.bytes.is_empty() || b <= lo || a > hi { continue; }
        if a < lo || b - 1 > hi { return Err(format!("Bank {bank}: Segment ${a:04X}..${:04X} liegt nicht im Fenster ${lo:04X}-${hi:04X}", b - 1)); }
        img[a - lo..b - lo].copy_from_slice(&s.bytes); used = true;
    }
    Ok(used.then_some(img))
}
fn banks(out: &Output) -> BTreeMap<u32, ()> { out.segments.iter().filter(|s| !s.bytes.is_empty()).map(|s| (s.bank, ())).collect() }

fn chip(bank: u16, addr: u16, data: &[u8]) -> Vec<u8> { chip_t(0, bank, addr, data) }
/// CHIP-Paket mit Typ (0 ROM, 2 Flash)
fn chip_t(kind: u16, bank: u16, addr: u16, data: &[u8]) -> Vec<u8> {
    let mut v = b"CHIP".to_vec(); v.extend((16 + data.len() as u32).to_be_bytes()); v.extend(kind.to_be_bytes()); v.extend(bank.to_be_bytes()); v.extend(addr.to_be_bytes()); v.extend((data.len() as u16).to_be_bytes()); v.extend_from_slice(data); v
}

/// C64-CRT. `kind`: `normal` (Standard), `ocean`, `magicdesk`, `easyflash`, `actionreplay`, `ultimax`.
pub fn to_crt(out: &Output, kind: Option<&str>) -> Result<Vec<u8>, String> {
    let kind = kind.or(out.cart.as_ref().map(|c| c.0.as_str())).unwrap_or("normal").to_ascii_lowercase();
    let name = out.cart.as_ref().map(|c| c.1.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| "VINTAGECODER".into());
    let bs = banks(out); if bs.is_empty() { return Err("keine Ausgabe".into()); }
    let (hw, exrom, game, mut chips): (u16, u8, u8, Vec<u8>) = match kind.as_str() {
        "normal" | "8k" | "16k" => {
            if bs.len() != 1 || !bs.contains_key(&0) { return Err("Standardmodul: nur Bank 0 (für mehrere Bänke ocean oder magicdesk wählen)".into()); }
            let rom = window(&out.segments, 0, 0x8000, 0xBFFF)?.ok_or("Standardmodul: Code muss in $8000-$BFFF liegen")?;
            if out.segments.iter().any(|s| !s.bytes.is_empty() && ((s.start as usize) < 0x8000 || s.start as usize + s.bytes.len() > 0xC000)) { return Err("Standardmodul: Code außerhalb $8000-$BFFF".into()); }
            let big = out.segments.iter().any(|s| s.start as usize + s.bytes.len() > 0xA000) || kind == "16k";
            if big { (0, 0, 0, chip(0, 0x8000, &rom)) } else { (0, 0, 1, chip(0, 0x8000, &rom[..0x2000])) }
        }
        "ocean" | "magicdesk" => {
            let (hw, min, max) = if kind == "ocean" { (5u16, 16usize, 64usize) } else { (19, 4, 16) };
            let top = *bs.keys().next_back().unwrap() as usize + 1;
            let count = top.max(min).next_power_of_two(); if count > max { return Err(format!("{kind}: höchstens {max} Bänke (8 KB)")); }
            let mut v = vec![];
            for b in 0..count as u32 {
                if kind == "ocean" && b >= 16 { // Bänke 16..31 liegen bei $A000 (ROMH)
                    let data = window(&out.segments, b, 0xA000, 0xBFFF)?.unwrap_or(vec![FILL; 0x2000]); v.extend(chip(b as u16, 0xA000, &data));
                } else {
                    let data = window(&out.segments, b, 0x8000, 0x9FFF)?.unwrap_or(vec![FILL; 0x2000]); v.extend(chip(b as u16, 0x8000, &data));
                }
            }
            (hw, 0, if kind == "ocean" { 0 } else { 1 }, v)   // Ocean meldet EXROM=0/GAME=0, Magic Desk 8K-Modus
        }
        "easyflash" => {
            // je Bank ROML $8000-$9FFF und ROMH (im CRT bei $A000; Code für den Ultimax-Start darf bei $E000 stehen), Typ „Flash“
            let top = *bs.keys().next_back().unwrap() as usize + 1;
            if top > 64 { return Err("easyflash: höchstens 64 Bänke (je 8 KB ROML + 8 KB ROMH)".into()); }
            let mut v = vec![];
            for b in 0..top as u32 {
                let lo = window(&out.segments, b, 0x8000, 0x9FFF)?;
                let (ha, he) = (window(&out.segments, b, 0xA000, 0xBFFF)?, window(&out.segments, b, 0xE000, 0xFFFF)?);
                if ha.is_some() && he.is_some() { return Err(format!("easyflash: Bank {b} belegt ROMH sowohl bei $A000 als auch bei $E000")); }
                let hi = ha.or(he);
                if b == 0 && hi.is_none() { return Err("easyflash: Bank 0 braucht ROMH-Code (Startvektoren bei $FFFC im Fenster $E000-$FFFF)".into()); }
                if lo.is_none() && hi.is_none() { continue; }
                v.extend(chip_t(2, b as u16, 0x8000, &lo.unwrap_or(vec![FILL; 0x2000])));
                v.extend(chip_t(2, b as u16, 0xA000, &hi.unwrap_or(vec![FILL; 0x2000])));
            }
            (32, 1, 0, v)
        }
        "actionreplay" | "ar" => {
            let top = *bs.keys().next_back().unwrap() as usize + 1;
            if top > 4 { return Err("actionreplay: genau 4 Bänke (je 8 KB bei $8000)".into()); }
            let mut v = vec![];
            for b in 0..4u32 { let d = window(&out.segments, b, 0x8000, 0x9FFF)?.unwrap_or(vec![FILL; 0x2000]); v.extend(chip(b as u16, 0x8000, &d)); }
            (1, 0, 1, v)
        }
        "ultimax" => {
            // Ultimax-Modus: ROML $8000-$9FFF und/oder ROMH $E000-$FFFF (Reset-Vektor!)
            if bs.len() != 1 || !bs.contains_key(&0) { return Err("ultimax: nur Bank 0".into()); }
            let (lo, hi) = (window(&out.segments, 0, 0x8000, 0x9FFF)?, window(&out.segments, 0, 0xE000, 0xFFFF)?);
            if hi.is_none() { return Err("ultimax: ROMH-Code im Fenster $E000-$FFFF mit Reset-Vektor bei $FFFC fehlt".into()); }
            if out.segments.iter().any(|s| !s.bytes.is_empty() && !((s.start as usize >= 0x8000 && s.start as usize + s.bytes.len() <= 0xA000) || s.start as usize >= 0xE000)) { return Err("ultimax: Code nur in $8000-$9FFF oder $E000-$FFFF".into()); }
            let mut v = vec![]; if let Some(l) = lo { v.extend(chip(0, 0x8000, &l)); } v.extend(chip(0, 0xE000, &hi.unwrap()));
            (0, 1, 0, v)
        }
        k => return Err(format!("unbekannter Modultyp '{k}' (normal, 16k, ocean, magicdesk, easyflash, actionreplay, ultimax)")),
    };
    let mut f = b"C64 CARTRIDGE   ".to_vec(); f.extend(0x40u32.to_be_bytes()); f.extend(0x0100u16.to_be_bytes()); f.extend(hw.to_be_bytes());
    f.extend([exrom, game, 0, 0, 0, 0, 0, 0]);
    let mut nm = name.to_ascii_uppercase().into_bytes(); nm.resize(32, 0); f.extend(&nm[..32]); f.append(&mut chips); Ok(f)
}

/// Atari-Modulabbild und CAR-Typ. `kind`: `8k`, `16k`, `xegs` (32/64/128 KB je nach Bankzahl); Standard: aus dem Code abgeleitet.
fn car_image(out: &Output, kind: Option<&str>) -> Result<(u32, Vec<u8>), String> {
    let kind = kind.or(out.cart.as_ref().map(|c| c.0.as_str())).unwrap_or("auto").to_ascii_lowercase();
    let bs = banks(out); if bs.is_empty() { return Err("keine Ausgabe".into()); }
    let mut segs = out.segments.clone();
    // Startvektoren ($BFFA..$BFFF) ergänzen, wenn das Programm sie nicht selbst anlegt: Run-Adresse, Flag 0, Option 4 (Modul starten), Init = RTS
    let top_bank = *bs.keys().next_back().unwrap();
    let vec_bank = if kind == "xegs" { xegs_banks(top_bank as usize)? as u32 - 1 } else { 0 };
    let has_vectors = segs.iter().any(|s| s.bank == vec_bank && (s.start as usize) <= 0xBFFA && s.start as usize + s.bytes.len() > 0xBFFA);
    if !has_vectors {
        let run = out.run.ok_or("Atari-Modul: Startadresse fehlt (.run oder eigene Vektoren bei $BFFA)")?;
        segs.push(Segment { start: 0xBFF9, bytes: vec![0x60, run as u8, (run >> 8) as u8, 0, 0x04, 0xF9, 0xBF], bank: vec_bank, name: "vectors".into() });
    }
    match kind.as_str() {
        "auto" | "8k" | "16k" => {
            if bs.len() != 1 { return Err("mehrere Bänke: Modultyp xegs wählen".into()); }
            let big = kind == "16k" || segs.iter().any(|s| !s.bytes.is_empty() && (s.start as usize) < 0xA000);
            let (lo, size, ty) = if big { (0x8000usize, 0x4000usize, 2) } else { (0xA000, 0x2000, 1) };
            if segs.iter().any(|s| !s.bytes.is_empty() && ((s.start as usize) < lo || s.start as usize + s.bytes.len() > 0xC000)) { return Err(format!("Atari-Modul: Code außerhalb ${lo:04X}-$BFFF")); }
            let img = window(&segs, 0, lo, lo + size - 1)?.unwrap();
            Ok((ty, img))
        }
        "xegs" => {
            let n = xegs_banks(top_bank as usize)?;
            let mut img = vec![];
            for b in 0..n as u32 {
                let (lo, hi) = if b as usize == n - 1 { (0xA000, 0xBFFF) } else { (0x8000, 0x9FFF) };
                img.extend(window(&segs, b, lo, hi)?.unwrap_or(vec![FILL; 0x2000]));
            }
            Ok((match n { 4 => 12, 8 => 13, _ => 14 }, img))
        }
        k => Err(format!("unbekannter Modultyp '{k}' (8k, 16k, xegs)")),
    }
}
fn xegs_banks(top: usize) -> Result<usize, String> { let n = (top + 1).max(4).next_power_of_two(); if n > 16 { Err("XEGS: höchstens 16 Bänke (128 KB)".into()) } else { Ok(n) } }

pub fn to_car(out: &Output, kind: Option<&str>) -> Result<Vec<u8>, String> {
    let (ty, img) = car_image(out, kind)?;
    let sum: u32 = img.iter().map(|b| *b as u32).sum();
    let mut f = b"CART".to_vec(); f.extend(ty.to_be_bytes()); f.extend(sum.to_be_bytes()); f.extend(0u32.to_be_bytes()); f.extend(img); Ok(f)
}
/// Rohes Modulabbild ohne CAR-Kopf (`.rom`)
pub fn to_rom(out: &Output, kind: Option<&str>) -> Result<Vec<u8>, String> { car_image(out, kind).map(|x| x.1) }

/// VC-20-Steckmodul: rohes 8-KB-Abbild für den Speicherblock `kind` (`a0` = `$A000`–`$BFFF`, `20`/`40`/`60` = `$2000`/`$4000`/`$6000`); ungenutzte Bytes $FF.
/// Rückgabe: (Dateiendung = Blockname, Abbild). Ein Modul bei `$A000` startet das System selbst, wenn bei `$A004` die Kennung `A0CBM` steht
/// (`.byte $41,$30,$c3,$c2,$cd`) und bei `$A000`/`$A002` die Kalt- und Warmstartadresse.
pub fn to_vic20_cart(out: &Output, kind: Option<&str>) -> Result<(&'static str, Vec<u8>), String> {
    let kind = kind.or(out.cart.as_ref().map(|c| c.0.as_str())).unwrap_or("a0").to_ascii_lowercase();
    let (ext, base): (&'static str, usize) = match kind.trim_start_matches('$').trim_start_matches("0x").trim_end_matches("00") {
        "a0" | "a" | "normal" | "8k" | "" => ("a0", 0xA000), "20" | "2" => ("20", 0x2000), "40" | "4" => ("40", 0x4000), "60" | "6" => ("60", 0x6000),
        k => return Err(format!("unbekannter VC-20-Modultyp '{k}' (a0, 20, 40, 60)")),
    };
    if banks(out).iter().any(|(b, _)| *b != 0) { return Err("VC-20-Modul: keine Bänke (.bank) möglich".into()); }
    let img = window(&out.segments, 0, base, base + 0x1FFF)?.ok_or(format!("VC-20-Modul: Code muss in ${base:04X}-${:04X} liegen", base + 0x1FFF))?;
    if out.segments.iter().any(|s| !s.bytes.is_empty() && ((s.start as usize) < base || s.start as usize + s.bytes.len() > base + 0x2000)) { return Err(format!("VC-20-Modul: Code außerhalb ${base:04X}-${:04X}", base + 0x1FFF)); }
    if base == 0xA000 && img[4..9] != [0x41, 0x30, 0xC3, 0xC2, 0xCD] {
        return Err("VC-20-Modul bei $A000: Kennung A0CBM bei $A004 fehlt (.byte $41,$30,$c3,$c2,$cd; davor die Start-Vektoren .word kalt, warm)".into());
    }
    Ok((ext, img))
}
