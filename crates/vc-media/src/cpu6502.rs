// SPDX-License-Identifier: GPL-3.0-or-later
//! Kleiner 6502-Kern (NMOS, mit den gebräuchlichen undokumentierten Befehlen) für das Abspielen von SID-Dateien ohne Emulator.
//! Die Zyklenzahlen sind Näherungen (für Rasterabfragen und Zeitgrenzen); die Speicherzugriffe laufen über [`Bus`].

pub trait Bus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}

#[derive(Clone, Debug, Default)]
pub struct Cpu { pub a: u8, pub x: u8, pub y: u8, pub sp: u8, pub pc: u16, pub c: bool, pub z: bool, pub i: bool, pub d: bool, pub v: bool, pub n: bool }

#[derive(Clone, Copy, PartialEq)]
enum M { Imm, Zp, Zpx, Zpy, Abs, Abx, Aby, Izx, Izy }

enum Operand { Imm(u8), Addr(u16) }

impl Cpu {
    pub fn new() -> Cpu { Cpu { sp: 0xFD, i: true, ..Default::default() } }

    pub fn status(&self, brk: bool) -> u8 {
        (self.c as u8) | (self.z as u8) << 1 | (self.i as u8) << 2 | (self.d as u8) << 3 | (brk as u8) << 4 | 0x20 | (self.v as u8) << 6 | (self.n as u8) << 7
    }
    fn set_status(&mut self, p: u8) { self.c = p & 1 != 0; self.z = p & 2 != 0; self.i = p & 4 != 0; self.d = p & 8 != 0; self.v = p & 0x40 != 0; self.n = p & 0x80 != 0; }
    pub fn push<B: Bus>(&mut self, b: &mut B, v: u8) { b.write(0x100 | self.sp as u16, v); self.sp = self.sp.wrapping_sub(1); }
    pub fn pull<B: Bus>(&mut self, b: &mut B) -> u8 { self.sp = self.sp.wrapping_add(1); b.read(0x100 | self.sp as u16) }
    fn fetch<B: Bus>(&mut self, b: &mut B) -> u8 { let v = b.read(self.pc); self.pc = self.pc.wrapping_add(1); v }
    fn fetch16<B: Bus>(&mut self, b: &mut B) -> u16 { let lo = self.fetch(b) as u16; lo | (self.fetch(b) as u16) << 8 }
    fn nz(&mut self, v: u8) -> u8 { self.z = v == 0; self.n = v & 0x80 != 0; v }

    /// Operand holen; liefert zusätzlich, ob eine Seitengrenze überschritten wurde
    fn operand<B: Bus>(&mut self, b: &mut B, m: M) -> (Operand, bool) {
        match m {
            M::Imm => (Operand::Imm(self.fetch(b)), false),
            M::Zp => (Operand::Addr(self.fetch(b) as u16), false),
            M::Zpx => { let z = self.fetch(b); (Operand::Addr(z.wrapping_add(self.x) as u16), false) }
            M::Zpy => { let z = self.fetch(b); (Operand::Addr(z.wrapping_add(self.y) as u16), false) }
            M::Abs => (Operand::Addr(self.fetch16(b)), false),
            M::Abx => { let a = self.fetch16(b); let r = a.wrapping_add(self.x as u16); (Operand::Addr(r), (a ^ r) & 0xFF00 != 0) }
            M::Aby => { let a = self.fetch16(b); let r = a.wrapping_add(self.y as u16); (Operand::Addr(r), (a ^ r) & 0xFF00 != 0) }
            M::Izx => { let z = self.fetch(b).wrapping_add(self.x); let lo = b.read(z as u16) as u16; let hi = b.read(z.wrapping_add(1) as u16) as u16; (Operand::Addr(lo | hi << 8), false) }
            M::Izy => { let z = self.fetch(b); let lo = b.read(z as u16) as u16; let hi = b.read(z.wrapping_add(1) as u16) as u16; let a = lo | hi << 8; let r = a.wrapping_add(self.y as u16); (Operand::Addr(r), (a ^ r) & 0xFF00 != 0) }
        }
    }
    fn base_cycles(m: M) -> u32 { match m { M::Imm => 2, M::Zp => 3, M::Zpx | M::Zpy | M::Abs | M::Abx | M::Aby => 4, M::Izx => 6, M::Izy => 5 } }
    fn load<B: Bus>(b: &mut B, o: &Operand) -> u8 { match o { Operand::Imm(v) => *v, Operand::Addr(a) => b.read(*a) } }

    fn adc(&mut self, v: u8) {
        let a = self.a as u16; let c = self.c as u16;
        if self.d {
            let bin = a + v as u16 + c; self.z = bin & 0xFF == 0;
            let mut lo = (a & 0xF) + (v as u16 & 0xF) + c; if lo >= 0xA { lo = ((lo + 6) & 0xF) + 0x10; }
            let mut r = (a & 0xF0) + (v as u16 & 0xF0) + lo;
            self.n = r & 0x80 != 0; self.v = (!(a ^ v as u16) & (a ^ r) & 0x80) != 0;
            if r >= 0xA0 { r += 0x60; }
            self.c = r >= 0x100; self.a = r as u8;
        } else {
            let r = a + v as u16 + c;
            self.v = (!(a ^ v as u16) & (a ^ r) & 0x80) != 0; self.c = r > 0xFF; self.a = r as u8; let a = self.a; self.nz(a);
        }
    }
    fn sbc(&mut self, v: u8) {
        if self.d {
            let (a, c) = (self.a as i32, self.c as i32);
            let bin = a - v as i32 - (1 - c);
            self.c = bin >= 0; self.v = ((a ^ v as i32) & (a ^ bin) & 0x80) != 0; self.z = bin & 0xFF == 0; self.n = bin & 0x80 != 0;
            let mut lo = (a & 0xF) - (v as i32 & 0xF) - (1 - c); if lo < 0 { lo = ((lo - 6) & 0xF) - 0x10; }
            let mut r = (a & 0xF0) - (v as i32 & 0xF0) + lo; if r < 0 { r -= 0x60; }
            self.a = r as u8;
        } else { self.adc_bin(!v); }
    }
    fn adc_bin(&mut self, v: u8) {
        let (a, c) = (self.a as u16, self.c as u16); let r = a + v as u16 + c;
        self.v = (!(a ^ v as u16) & (a ^ r) & 0x80) != 0; self.c = r > 0xFF; self.a = r as u8; let a = self.a; self.nz(a);
    }
    fn cmp(&mut self, r: u8, v: u8) { self.c = r >= v; self.nz(r.wrapping_sub(v)); }
    fn asl(&mut self, v: u8) -> u8 { self.c = v & 0x80 != 0; self.nz(v << 1) }
    fn lsr(&mut self, v: u8) -> u8 { self.c = v & 1 != 0; self.nz(v >> 1) }
    fn rol(&mut self, v: u8) -> u8 { let c = self.c as u8; self.c = v & 0x80 != 0; self.nz(v << 1 | c) }
    fn ror(&mut self, v: u8) -> u8 { let c = (self.c as u8) << 7; self.c = v & 1 != 0; self.nz(v >> 1 | c) }

    /// Springt in die Unterbrechungsroutine (wie ein IRQ): Rückkehradresse und Status auf den Stapel
    pub fn irq<B: Bus>(&mut self, b: &mut B, vector: u16) {
        let pc = self.pc; self.push(b, (pc >> 8) as u8); self.push(b, pc as u8); let p = self.status(false); self.push(b, p); self.i = true; self.pc = vector;
    }

    /// Einen Befehl ausführen; liefert die (genäherten) Zyklen
    pub fn step<B: Bus>(&mut self, b: &mut B) -> Result<u32, String> {
        let at = self.pc; let op = self.fetch(b);
        // --- Befehle ohne Operanden und Sprünge
        match op {
            0x00 => { let pc = self.pc.wrapping_add(1); self.push(b, (pc >> 8) as u8); self.push(b, pc as u8); let p = self.status(true); self.push(b, p); self.i = true; self.pc = b.read(0xFFFE) as u16 | (b.read(0xFFFF) as u16) << 8; return Ok(7); }
            0x20 => { let t = self.fetch16(b); let r = self.pc.wrapping_sub(1); self.push(b, (r >> 8) as u8); self.push(b, r as u8); self.pc = t; return Ok(6); }
            0x40 => { let p = self.pull(b); self.set_status(p); let lo = self.pull(b) as u16; let hi = self.pull(b) as u16; self.pc = lo | hi << 8; return Ok(6); }
            0x60 => { let lo = self.pull(b) as u16; let hi = self.pull(b) as u16; self.pc = (lo | hi << 8).wrapping_add(1); return Ok(6); }
            0x4C => { self.pc = self.fetch16(b); return Ok(3); }
            0x6C => { let p = self.fetch16(b); let lo = b.read(p) as u16; let hi = b.read((p & 0xFF00) | (p.wrapping_add(1) & 0xFF)) as u16; self.pc = lo | hi << 8; return Ok(5); }
            0x08 => { let p = self.status(true); self.push(b, p); return Ok(3); }
            0x28 => { let p = self.pull(b); self.set_status(p); return Ok(4); }
            0x48 => { let a = self.a; self.push(b, a); return Ok(3); }
            0x68 => { let a = self.pull(b); self.a = self.nz(a); return Ok(4); }
            0x18 => { self.c = false; return Ok(2); } 0x38 => { self.c = true; return Ok(2); }
            0x58 => { self.i = false; return Ok(2); } 0x78 => { self.i = true; return Ok(2); }
            0xB8 => { self.v = false; return Ok(2); }
            0xD8 => { self.d = false; return Ok(2); } 0xF8 => { self.d = true; return Ok(2); }
            0xAA => { self.x = self.nz(self.a); return Ok(2); } 0xA8 => { self.y = self.nz(self.a); return Ok(2); }
            0x8A => { self.a = self.nz(self.x); return Ok(2); } 0x98 => { self.a = self.nz(self.y); return Ok(2); }
            0xBA => { self.x = self.nz(self.sp); return Ok(2); } 0x9A => { self.sp = self.x; return Ok(2); }
            0xE8 => { self.x = self.nz(self.x.wrapping_add(1)); return Ok(2); } 0xC8 => { self.y = self.nz(self.y.wrapping_add(1)); return Ok(2); }
            0xCA => { self.x = self.nz(self.x.wrapping_sub(1)); return Ok(2); } 0x88 => { self.y = self.nz(self.y.wrapping_sub(1)); return Ok(2); }
            0x0A => { self.a = self.asl(self.a); return Ok(2); } 0x4A => { self.a = self.lsr(self.a); return Ok(2); }
            0x2A => { self.a = self.rol(self.a); return Ok(2); } 0x6A => { self.a = self.ror(self.a); return Ok(2); }
            0xEA | 0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => return Ok(2),
            0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => { self.fetch(b); return Ok(2); }
            0x04 | 0x44 | 0x64 => { self.fetch(b); return Ok(3); }
            0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => { self.fetch(b); return Ok(4); }
            0x0C => { self.fetch16(b); return Ok(4); }
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => { self.fetch16(b); return Ok(4); }
            x if x & 0x1F == 0x10 => {
                let off = self.fetch(b) as i8;
                let take = match x >> 5 { 0 => !self.n, 1 => self.n, 2 => !self.v, 3 => self.v, 4 => !self.c, 5 => self.c, 6 => !self.z, _ => self.z };
                if take { let old = self.pc; self.pc = self.pc.wrapping_add(off as u16); return Ok(if (old ^ self.pc) & 0xFF00 != 0 { 4 } else { 3 }); }
                return Ok(2);
            }
            _ => {}
        }
        let (aaa, bbb, cc) = (op >> 5, (op >> 2) & 7, op & 3);
        match cc {
            1 => {
                let m = [M::Izx, M::Zp, M::Imm, M::Abs, M::Izy, M::Zpx, M::Aby, M::Abx][bbb as usize];
                if op == 0x89 { return Err(format!("unbekannter Befehl ${op:02X} bei ${at:04X}")); }
                let (o, cross) = self.operand(b, m);
                let mut cyc = Self::base_cycles(m);
                if aaa == 4 { // STA
                    let Operand::Addr(a) = o else { return Err(format!("unbekannter Befehl ${op:02X} bei ${at:04X}")); };
                    b.write(a, self.a); if matches!(m, M::Abx | M::Aby | M::Izy) { cyc += 1; } return Ok(cyc);
                }
                if cross { cyc += 1; }
                let v = Self::load(b, &o);
                match aaa {
                    0 => { self.a = self.nz(self.a | v); } 1 => { self.a = self.nz(self.a & v); } 2 => { self.a = self.nz(self.a ^ v); }
                    3 => self.adc(v), 5 => { self.a = self.nz(v); } 6 => { let a = self.a; self.cmp(a, v); } _ => self.sbc(v),
                }
                Ok(cyc)
            }
            2 => {
                let store_load_y = matches!(aaa, 4 | 5);   // STX/LDX nutzen Zeropage,Y bzw. Absolut,Y
                if op == 0x9E { return Err(format!("unbekannter Befehl ${op:02X} bei ${at:04X}")); }
                let m = match bbb { 0 => M::Imm, 1 => M::Zp, 3 => M::Abs, 5 => if store_load_y { M::Zpy } else { M::Zpx }, 7 => if store_load_y { M::Aby } else { M::Abx }, _ => return Err(format!("unbekannter Befehl ${op:02X} bei ${at:04X}")) };
                if bbb == 0 && aaa != 5 { return Err(format!("unbekannter Befehl ${op:02X} bei ${at:04X}")); }   // KIL und Verwandte
                let (o, cross) = self.operand(b, m);
                let mut cyc = Self::base_cycles(m);
                match aaa {
                    4 => { let Operand::Addr(a) = o else { return Err("STX".into()) }; b.write(a, self.x); if m == M::Aby { cyc += 1; } Ok(cyc) }
                    5 => { let v = Self::load(b, &o); self.x = self.nz(v); Ok(cyc + cross as u32) }
                    _ => {
                        let Operand::Addr(a) = o else { return Err(format!("unbekannter Befehl ${op:02X} bei ${at:04X}")) };
                        let v = b.read(a);
                        let r = match aaa { 0 => self.asl(v), 1 => self.rol(v), 2 => self.lsr(v), 3 => self.ror(v), 6 => { let r = v.wrapping_sub(1); self.nz(r) } _ => { let r = v.wrapping_add(1); self.nz(r) } };
                        b.write(a, r); Ok(cyc + 2 + if matches!(m, M::Abx) { 1 } else { 0 })
                    }
                }
            }
            0 => {
                if op == 0x9C { return Err(format!("unbekannter Befehl ${op:02X} bei ${at:04X}")); }
                let m = match bbb { 0 => M::Imm, 1 => M::Zp, 3 => M::Abs, 5 => M::Zpx, 7 => M::Abx, _ => return Err(format!("unbekannter Befehl ${op:02X} bei ${at:04X}")) };
                if matches!(aaa, 0 | 1 | 2 | 3) && !(aaa == 1 && matches!(bbb, 1 | 3)) { return Err(format!("unbekannter Befehl ${op:02X} bei ${at:04X}")); }
                let (o, cross) = self.operand(b, m);
                let cyc = Self::base_cycles(m);
                match aaa {
                    1 => { let v = Self::load(b, &o); self.z = self.a & v == 0; self.n = v & 0x80 != 0; self.v = v & 0x40 != 0; Ok(cyc) }
                    4 => { let Operand::Addr(a) = o else { return Err("STY".into()) }; b.write(a, self.y); Ok(cyc) }
                    5 => { let v = Self::load(b, &o); self.y = self.nz(v); Ok(cyc + cross as u32) }
                    6 => { let v = Self::load(b, &o); let y = self.y; self.cmp(y, v); Ok(cyc) }
                    7 => { let v = Self::load(b, &o); let x = self.x; self.cmp(x, v); Ok(cyc) }
                    _ => Err(format!("unbekannter Befehl ${op:02X} bei ${at:04X}")),
                }
            }
            _ => self.illegal(b, op, aaa, bbb, at),
        }
    }

    /// Undokumentierte Befehle (Spalte 3 der Befehlsmatrix)
    fn illegal<B: Bus>(&mut self, b: &mut B, op: u8, aaa: u8, bbb: u8, at: u16) -> Result<u32, String> {
        let bad = || format!("unbekannter Befehl ${op:02X} bei ${at:04X}");
        if bbb == 2 { // Direktoperanden
            let v = self.fetch(b);
            match op {
                0x0B | 0x2B => { self.a = self.nz(self.a & v); self.c = self.n; }
                0x4B => { let t = self.a & v; self.a = self.lsr(t); }
                0x6B => { let t = self.a & v; let c = self.c as u8; let r = t >> 1 | c << 7; self.nz(r); self.c = r & 0x40 != 0; self.v = ((r >> 6) ^ (r >> 5)) & 1 != 0; self.a = r; }
                0x8B => { self.a = self.nz(self.x & v); }
                0xAB => { self.a = self.nz(v); self.x = self.a; }
                0xCB => { let t = self.a & self.x; self.c = t >= v; self.x = self.nz(t.wrapping_sub(v)); }
                0xEB => self.sbc(v),
                _ => return Err(bad()),
            }
            return Ok(2);
        }
        let zpy = matches!(aaa, 4 | 5);
        let m = match bbb { 0 => M::Izx, 1 => M::Zp, 3 => M::Abs, 4 => M::Izy, 5 => if zpy { M::Zpy } else { M::Zpx }, 6 => M::Aby, _ => if aaa == 5 { M::Aby } else { M::Abx } };
        if matches!(op, 0x93 | 0x9B | 0x9F | 0x9C | 0x9E | 0x92) || (op == 0xBB) {
            if op == 0xBB { let (o, _) = self.operand(b, M::Aby); let v = Self::load(b, &o) & self.sp; self.a = self.nz(v); self.x = self.a; self.sp = v; return Ok(4); }
            return Err(bad());
        }
        let (o, cross) = self.operand(b, m);
        let Operand::Addr(a) = o else { return Err(bad()) };
        let mut cyc = Self::base_cycles(m);
        match aaa {
            4 => { b.write(a, self.a & self.x); return Ok(cyc); }                                   // SAX
            5 => { let v = b.read(a); self.a = self.nz(v); self.x = self.a; return Ok(cyc + cross as u32); }   // LAX
            _ => {}
        }
        let v = b.read(a);
        let r = match aaa {
            0 => { let r = self.asl(v); self.a = self.nz(self.a | r); r }                           // SLO
            1 => { let r = self.rol(v); self.a = self.nz(self.a & r); r }                           // RLA
            2 => { let r = self.lsr(v); self.a = self.nz(self.a ^ r); r }                           // SRE
            3 => { let r = self.ror(v); self.adc(r); r }                                            // RRA
            6 => { let r = v.wrapping_sub(1); let a = self.a; self.cmp(a, r); r }                   // DCP
            _ => { let r = v.wrapping_add(1); self.sbc(r); r }                                      // ISC
        };
        b.write(a, r); cyc += 2 + if matches!(m, M::Abx | M::Aby | M::Izy) { 1 } else { 0 };
        Ok(cyc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Ram(Vec<u8>);
    impl Bus for Ram { fn read(&mut self, a: u16) -> u8 { self.0[a as usize] } fn write(&mut self, a: u16, v: u8) { self.0[a as usize] = v; } }
    fn run(code: &[u8], setup: impl FnOnce(&mut Cpu, &mut Ram)) -> (Cpu, Ram) {
        let mut r = Ram(vec![0; 65536]); r.0[0x200..0x200 + code.len()].copy_from_slice(code);
        let mut c = Cpu::new(); c.pc = 0x200; setup(&mut c, &mut r);
        for _ in 0..10000 { if r.0[c.pc as usize] == 0x02 { break; } c.step(&mut r).unwrap(); }   // $02 = Ende (KIL) als Marke
        (c, r)
    }
    #[test]
    fn arithmetic_flags_and_memory() {
        // LDA #$50 ; CLC ; ADC #$50 -> $A0, V=1, N=1 ; STA $10 ; LDX $10 ; INX ; STX $11
        let (c, r) = run(&[0xA9, 0x50, 0x18, 0x69, 0x50, 0x85, 0x10, 0xA6, 0x10, 0xE8, 0x86, 0x11, 0x02], |_, _| {});
        assert_eq!((c.a, c.v, c.n, c.c, r.0[0x10], r.0[0x11]), (0xA0, true, true, false, 0xA0, 0xA1));
        // SEC ; LDA #$10 ; SBC #$20 -> $F0, C=0
        let (c, _) = run(&[0x38, 0xA9, 0x10, 0xE9, 0x20, 0x02], |_, _| {});
        assert_eq!((c.a, c.c, c.n), (0xF0, false, true));
        // Dezimal: SED ; CLC ; LDA #$19 ; ADC #$28 -> $47 ; SEC ; SBC #$08 -> $39
        let (c, _) = run(&[0xF8, 0x18, 0xA9, 0x19, 0x69, 0x28, 0x38, 0xE9, 0x08, 0x02], |_, _| {});
        assert_eq!(c.a, 0x39);
    }
    #[test]
    fn branches_stack_and_subroutines() {
        // LDX #3 ; loop: DEX ; BNE loop ; JSR sub ; JMP end ; sub: LDA #$77 ; RTS ; end: KIL
        let code = [0xA2, 0x03, 0xCA, 0xD0, 0xFD, 0x20, 0x0B, 0x02, 0x4C, 0x0E, 0x02, 0xA9, 0x77, 0x60, 0x02];
        let (c, _) = run(&code, |_, _| {});
        assert_eq!((c.x, c.a, c.sp), (0, 0x77, 0xFD));
        // PHA/PLA, PHP/PLP, Indirekt-indiziert: (zp),Y
        let (c, r) = run(&[0xA9, 0x42, 0x48, 0xA9, 0x00, 0x68, 0xA0, 0x02, 0xB1, 0x20, 0x91, 0x22, 0x02], |_, r| { r.0[0x20] = 0x00; r.0[0x21] = 0x30; r.0[0x22] = 0x00; r.0[0x23] = 0x31; r.0[0x3002] = 0x5A; });
        assert_eq!((c.a, r.0[0x3102]), (0x5A, 0x5A));
    }
    #[test]
    fn read_modify_write_and_illegal_opcodes() {
        // INC $10 ; ASL $10 ; ROR $10 -> (5+1)*2 = 12, ror -> 6
        let (_, r) = run(&[0xE6, 0x10, 0x06, 0x10, 0x66, 0x10, 0x02], |_, r| { r.0[0x10] = 5; });
        assert_eq!(r.0[0x10], 6);
        // LAX $10 ; SAX $11 ; DCP $12 ; SLO $13
        let (c, r) = run(&[0xA7, 0x10, 0x87, 0x11, 0xC7, 0x12, 0x07, 0x13, 0x02], |_, r| { r.0[0x10] = 0x0F; r.0[0x12] = 0x01; r.0[0x13] = 0x40; });
        assert_eq!((c.x, r.0[0x11], r.0[0x12], r.0[0x13]), (0x0F, 0x0F, 0x00, 0x80));
        assert_eq!(c.a, 0x8F);
        // JMP ($02FF) liest das High-Byte wie der echte Prozessor aus $0200 (Seitenüberlauf)
        let (c, _) = run(&[0x6C, 0xFF, 0x02], |_, r| { r.0[0x2FF] = 0x10; r.0[0x300] = 0x99; r.0[0x6C10] = 0x02; });
        assert_eq!(c.pc, 0x6C10);
        // KIL-Befehl führt zu einem Fehler
        let mut r = Ram(vec![0; 65536]); r.0[0x200] = 0x12; let mut c = Cpu::new(); c.pc = 0x200; assert!(c.step(&mut r).is_err());
    }
}
