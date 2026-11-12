// SPDX-License-Identifier: GPL-3.0-or-later
//! Minimaler PNG-Dekoder (nur Standardbibliothek): 8/16 Bit, Grau/RGB/Palette/+Alpha, ohne Interlace → RGBA8.

pub struct Image { pub w: usize, pub h: usize, pub rgba: Vec<u8> }

struct Bits<'a> { d: &'a [u8], pos: usize, bit: u32, nbits: u32 }
impl<'a> Bits<'a> {
    fn need(&mut self, n: u32) -> Result<(), String> {
        while self.nbits < n { let b = *self.d.get(self.pos).ok_or("PNG: Datenstrom endet vorzeitig")?; self.pos += 1; self.bit |= (b as u32) << self.nbits; self.nbits += 8; } Ok(())
    }
    fn get(&mut self, n: u32) -> Result<u32, String> { if n == 0 { return Ok(0); } self.need(n)?; let v = self.bit & ((1u32 << n) - 1); self.bit >>= n; self.nbits -= n; Ok(v) }
}
struct Huff { counts: [u16; 16], symbols: Vec<u16> }
impl Huff {
    fn new(lengths: &[u8]) -> Huff {
        let mut counts = [0u16; 16]; for &l in lengths { counts[l as usize] += 1; } counts[0] = 0;
        let mut offs = [0u16; 16]; for i in 1..16 { offs[i] = offs[i - 1] + counts[i - 1]; }
        let mut symbols = vec![0u16; lengths.len()];
        for (s, &l) in lengths.iter().enumerate() { if l != 0 { symbols[offs[l as usize] as usize] = s as u16; offs[l as usize] += 1; } }
        Huff { counts, symbols }
    }
    fn decode(&self, b: &mut Bits) -> Result<u16, String> {
        let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
        for len in 1..16 {
            code |= b.get(1)? as i32; let count = self.counts[len] as i32;
            if code - count < first { return Ok(self.symbols[(index + (code - first)) as usize]); }
            index += count; first += count; first <<= 1; code <<= 1;
        }
        Err("PNG: ungültiger Huffman-Code".into())
    }
}
const LBASE: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
const LEXT: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
const DBASE: [u16; 30] = [1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577];
const DEXT: [u8; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];

/// zlib-Datenstrom (RFC 1950/1951) entpacken
pub fn inflate(z: &[u8]) -> Result<Vec<u8>, String> {
    if z.len() < 2 { return Err("PNG: zlib-Strom zu kurz".into()); }
    let mut b = Bits { d: z, pos: 2, bit: 0, nbits: 0 }; let mut out: Vec<u8> = Vec::new();
    loop {
        let last = b.get(1)?; let kind = b.get(2)?;
        match kind {
            0 => {
                b.bit = 0; b.nbits = 0; let p = b.pos; if p + 4 > z.len() { return Err("PNG: Block abgeschnitten".into()); }
                let len = z[p] as usize | (z[p + 1] as usize) << 8; b.pos = p + 4; if b.pos + len > z.len() { return Err("PNG: Block abgeschnitten".into()); }
                out.extend_from_slice(&z[b.pos..b.pos + len]); b.pos += len;
            }
            1 | 2 => {
                let (lit, dist) = if kind == 1 {
                    let mut l = [0u8; 288]; for (i, v) in l.iter_mut().enumerate() { *v = if i < 144 { 8 } else if i < 256 { 9 } else if i < 280 { 7 } else { 8 }; }
                    (Huff::new(&l), Huff::new(&[5u8; 30]))
                } else {
                    let (nlen, ndist, ncode) = (b.get(5)? as usize + 257, b.get(5)? as usize + 1, b.get(4)? as usize + 4);
                    const ORD: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
                    let mut cl = [0u8; 19]; for &o in ORD.iter().take(ncode) { cl[o] = b.get(3)? as u8; }
                    let ch = Huff::new(&cl); let mut lens = vec![0u8; nlen + ndist]; let mut i = 0;
                    while i < nlen + ndist {
                        let s = ch.decode(&mut b)?;
                        if s < 16 { lens[i] = s as u8; i += 1; } else {
                            let (v, rep) = match s { 16 => { if i == 0 { return Err("PNG: ungültige Längen".into()); } (lens[i - 1], 3 + b.get(2)?) } 17 => (0, 3 + b.get(3)?), _ => (0, 11 + b.get(7)?) };
                            for _ in 0..rep { if i >= lens.len() { return Err("PNG: ungültige Längen".into()); } lens[i] = v; i += 1; }
                        }
                    }
                    (Huff::new(&lens[..nlen]), Huff::new(&lens[nlen..]))
                };
                loop {
                    let s = lit.decode(&mut b)? as usize;
                    if s < 256 { out.push(s as u8); } else if s == 256 { break; } else {
                        let s = s - 257; if s >= 29 { return Err("PNG: ungültiges Symbol".into()); }
                        let len = LBASE[s] as usize + b.get(LEXT[s] as u32)? as usize;
                        let ds = dist.decode(&mut b)? as usize; if ds >= 30 { return Err("PNG: ungültige Distanz".into()); }
                        let d = DBASE[ds] as usize + b.get(DEXT[ds] as u32)? as usize; if d > out.len() { return Err("PNG: Distanz zu groß".into()); }
                        for _ in 0..len { let c = out[out.len() - d]; out.push(c); }
                    }
                }
            }
            _ => return Err("PNG: ungültiger Block".into()),
        }
        if last == 1 { break; }
    }
    Ok(out)
}

pub fn decode(data: &[u8]) -> Result<Image, String> {
    if data.len() < 8 || &data[..8] != b"\x89PNG\r\n\x1a\n" { return Err("keine PNG-Datei".into()); }
    let (mut i, mut idat, mut pal, mut trns) = (8, Vec::new(), Vec::new(), Vec::new());
    let (mut w, mut h, mut depth, mut ct, mut inter) = (0usize, 0usize, 0u8, 0u8, 0u8);
    while i + 8 <= data.len() {
        let n = u32::from_be_bytes(data[i..i + 4].try_into().unwrap()) as usize; let t = &data[i + 4..i + 8]; if i + 12 + n > data.len() { return Err("PNG abgeschnitten".into()); }
        let c = &data[i + 8..i + 8 + n]; i += 12 + n;
        match t { b"IHDR" => { w = u32::from_be_bytes(c[0..4].try_into().unwrap()) as usize; h = u32::from_be_bytes(c[4..8].try_into().unwrap()) as usize; depth = c[8]; ct = c[9]; inter = c[12]; }
                  b"PLTE" => pal = c.to_vec(), b"tRNS" => trns = c.to_vec(), b"IDAT" => idat.extend_from_slice(c), _ => {} }
    }
    if w == 0 || h == 0 || w > 8192 || h > 8192 { return Err("PNG: ungültige Größe".into()); }
    if inter != 0 { return Err("PNG: Interlace wird nicht unterstützt (bitte ohne Interlace speichern)".into()); }
    let chans = match ct { 0 => 1, 2 => 3, 3 => 1, 4 => 2, 6 => 4, _ => return Err("PNG: Farbtyp nicht unterstützt".into()) };
    if !matches!(depth, 1 | 2 | 4 | 8 | 16) || (depth < 8 && ct != 0 && ct != 3) || (depth == 16 && ct == 3) { return Err("PNG: Bittiefe nicht unterstützt".into()); }
    let bpp_bits = chans * depth as usize; let bpp = (bpp_bits + 7) / 8; let stride = (w * bpp_bits + 7) / 8;
    let raw = inflate(&idat)?; if raw.len() < (stride + 1) * h { return Err("PNG: zu wenige Bilddaten".into()); }
    let mut rows: Vec<u8> = vec![0; stride * h];
    for y in 0..h {
        let f = raw[y * (stride + 1)]; let src = &raw[y * (stride + 1) + 1..(y + 1) * (stride + 1)];
        for x in 0..stride {
            let a = if x >= bpp { rows[y * stride + x - bpp] as i32 } else { 0 }; let b = if y > 0 { rows[(y - 1) * stride + x] as i32 } else { 0 };
            let c = if x >= bpp && y > 0 { rows[(y - 1) * stride + x - bpp] as i32 } else { 0 };
            let p = match f { 0 => 0, 1 => a, 2 => b, 3 => (a + b) / 2, 4 => { let pp = a + b - c; let (pa, pb, pc) = ((pp - a).abs(), (pp - b).abs(), (pp - c).abs()); if pa <= pb && pa <= pc { a } else if pb <= pc { b } else { c } } _ => return Err("PNG: ungültiger Filter".into()) };
            rows[y * stride + x] = (src[x] as i32 + p) as u8;
        }
    }
    let sample = |y: usize, x: usize, ch: usize| -> u32 {
        let r = &rows[y * stride..(y + 1) * stride];
        match depth { 8 => r[x * chans + ch] as u32, 16 => r[(x * chans + ch) * 2] as u32, d => { let bit = (x * chans + ch) * d as usize; ((r[bit / 8] >> (8 - d as usize - bit % 8)) & ((1 << d) - 1)) as u32 } }
    };
    let scale = |v: u32| -> u8 { match depth { 1 => (v * 255) as u8, 2 => (v * 85) as u8, 4 => (v * 17) as u8, _ => v as u8 } };
    let mut rgba = Vec::with_capacity(w * h * 4);
    for y in 0..h { for x in 0..w {
        let px = match ct {
            0 => { let g = scale(sample(y, x, 0)); [g, g, g, 255] }
            2 => [scale(sample(y, x, 0)), scale(sample(y, x, 1)), scale(sample(y, x, 2)), 255],
            3 => { let k = sample(y, x, 0) as usize; if k * 3 + 2 >= pal.len() { return Err("PNG: Palettenindex ungültig".into()); } [pal[k * 3], pal[k * 3 + 1], pal[k * 3 + 2], *trns.get(k).unwrap_or(&255)] }
            4 => { let g = scale(sample(y, x, 0)); [g, g, g, scale(sample(y, x, 1))] }
            _ => [scale(sample(y, x, 0)), scale(sample(y, x, 1)), scale(sample(y, x, 2)), scale(sample(y, x, 3))],
        };
        rgba.extend_from_slice(&px);
    } }
    Ok(Image { w, h, rgba })
}

/// Test-Hilfe: unkomprimiertes PNG (gespeicherte Deflate-Blöcke) schreiben
pub fn encode_rgba(w: usize, h: usize, rgba: &[u8]) -> Vec<u8> {
    fn crc(d: &[u8]) -> u32 { let mut c = !0u32; for &b in d { c ^= b as u32; for _ in 0..8 { c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 }; } } !c }
    fn adler(d: &[u8]) -> u32 { let (mut a, mut b) = (1u32, 0u32); for &x in d { a = (a + x as u32) % 65521; b = (b + a) % 65521; } (b << 16) | a }
    let mut raw = Vec::new(); for y in 0..h { raw.push(0); raw.extend_from_slice(&rgba[y * w * 4..(y + 1) * w * 4]); }
    let mut z = vec![0x78, 0x01]; let chunks: Vec<&[u8]> = raw.chunks(65535).collect();
    for (i, c) in chunks.iter().enumerate() { z.push(if i + 1 == chunks.len() { 1 } else { 0 }); z.extend_from_slice(&(c.len() as u16).to_le_bytes()); z.extend_from_slice(&(!(c.len() as u16)).to_le_bytes()); z.extend_from_slice(c); }
    z.extend_from_slice(&adler(&raw).to_be_bytes());
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut chunk = |t: &[u8; 4], d: &[u8]| { out.extend_from_slice(&(d.len() as u32).to_be_bytes()); let mut td = t.to_vec(); td.extend_from_slice(d); out.extend_from_slice(&td); out.extend_from_slice(&crc(&td).to_be_bytes()); };
    let mut ihdr = Vec::new(); ihdr.extend_from_slice(&(w as u32).to_be_bytes()); ihdr.extend_from_slice(&(h as u32).to_be_bytes()); ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(b"IHDR", &ihdr); chunk(b"IDAT", &z); chunk(b"IEND", &[]);
    out
}
