// SPDX-License-Identifier: GPL-3.0-or-later
//! Disassembler auf Basis der CPU-Tabelle (45GS02: auch Q-Befehle `NEG NEG …` und 32-Bit-Zeiger `$EA …`).
use vc_target::{Cpu, Mode, Op};

#[derive(Debug, Clone)]
pub struct DisLine { pub addr: u16, pub bytes: Vec<u8>, pub text: String, pub illegal: bool }

/// Operandentext; `b` = Opcode + Operandbytes, `addr` = Adresse des Opcodes
fn operand_text(op: &Op, b: &[u8], addr: u16, ptr32: bool) -> String {
    let m = op.mnemonic.to_ascii_lowercase();
    let w = if b.len() == 3 { b[1] as u16 | (b[2] as u16) << 8 } else { 0 };
    match op.mode {
        Mode::Imp => m,
        Mode::Acc => format!("{m} a"),
        Mode::Imm => format!("{m} #${:02x}", b[1]),
        Mode::Imw => format!("{m} #${w:04x}"),
        Mode::Zp => format!("{m} ${:02x}", b[1]),
        Mode::Zpx => format!("{m} ${:02x},x", b[1]),
        Mode::Zpy => format!("{m} ${:02x},y", b[1]),
        Mode::Abs => format!("{m} ${w:04x}"),
        Mode::Abx => format!("{m} ${w:04x},x"),
        Mode::Aby => format!("{m} ${w:04x},y"),
        Mode::Ind => format!("{m} (${w:04x})"),
        Mode::Iax => format!("{m} (${w:04x},x)"),
        Mode::Izx => format!("{m} (${:02x},x)", b[1]),
        Mode::Izy => format!("{m} (${:02x}),y", b[1]),
        Mode::Izz => if ptr32 { format!("{m} [${:02x}],z", b[1]) } else { format!("{m} (${:02x}),z", b[1]) },
        Mode::Isy => format!("{m} (${:02x},sp),y", b[1]),
        Mode::Rel => format!("{m} ${:04x}", (addr as i32 + 2 + b[1] as i8 as i32) as u16),
        Mode::Rl => format!("{m} ${:04x}", (addr as i32 + 2 + w as i16 as i32) as u16),
        Mode::Zpr => format!("{m} ${:02x},${:04x}", b[1], (addr as i32 + 3 + b[2] as i8 as i32) as u16),
    }
}

pub fn disassemble(cpu: &Cpu, data: &[u8], start: u16) -> Vec<DisLine> {
    let mut out = vec![];
    let mut i = 0;
    let byte_line = |out: &mut Vec<DisLine>, addr: u16, b: u8| out.push(DisLine { addr, bytes: vec![b], text: format!(".byte ${b:02x}"), illegal: false });
    while i < data.len() {
        let addr = start.wrapping_add(i as u16);
        // 45GS02-Präfixe: NEG NEG [NOP] → Q-Befehl, NOP → 32-Bit-Zeiger
        let (mut j, mut quad, mut ptr32) = (i, false, false);
        if cpu.quad_prefix.len() > 0 && data[i..].starts_with(&cpu.quad_prefix) { quad = true; j += cpu.quad_prefix.len(); }
        if let Some(n) = cpu.nop32 { if data.get(j) == Some(&n) && (quad || data.get(j + 1).and_then(|c| cpu.by_opcode(*c)).map_or(false, |o| o.mode == Mode::Izz)) { ptr32 = true; j += 1; } }
        let Some(op) = data.get(j).and_then(|c| cpu.by_opcode(*c)) else { byte_line(&mut out, addr, data[i]); i += 1; continue };
        let q_name = if quad { cpu.quad_ops.iter().find(|(_, b)| *b == op.mnemonic).map(|(q, _)| q.clone()) } else { None };
        if (quad && q_name.is_none()) || (ptr32 && op.mode != Mode::Izz) { // kein gültiges Präfix: das erste Byte normal dekodieren
            let Some(op) = cpu.by_opcode(data[i]) else { byte_line(&mut out, addr, data[i]); i += 1; continue };
            let n = op.bytes as usize;
            if i + n > data.len() { byte_line(&mut out, addr, data[i]); i += 1; continue; }
            out.push(DisLine { addr, bytes: data[i..i + n].to_vec(), text: operand_text(op, &data[i..i + n], addr, false), illegal: op.group != "official" });
            i += n; continue;
        }
        let n = op.bytes as usize;
        if j + n > data.len() { byte_line(&mut out, addr, data[i]); i += 1; continue; }
        let mut shown = op.clone();
        if let Some(q) = q_name { shown.mnemonic = q; }
        let mut text = operand_text(&shown, &data[j..j + n], addr.wrapping_add((j - i) as u16), ptr32);
        if quad && shown.mode == Mode::Acc { text = text.replace(" a", " q"); }
        out.push(DisLine { addr, bytes: data[i..j + n].to_vec(), text, illegal: op.group != "official" });
        i = j + n;
    }
    out
}
