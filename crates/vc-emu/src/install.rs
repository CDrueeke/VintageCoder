// SPDX-License-Identifier: GPL-3.0-or-later
//! Emulatoren beim Anwender installieren (portabel, unterhalb von VintageCoder): feste Version von der Seite des Projekts laden, SHA-256 prüfen,
//! in den eigenen Ordner (`<Konfigurationsordner>/VintageCoder/emulators/<Kennung>/<Version>`) auspacken. Es wird nichts mitgeliefert und nichts
//! ohne ausdrücklichen Aufruf geladen. Welche Pakete es gibt, steht im Katalog `data/emulators/catalog.json` (Lesen in `vc-ide`).
//! Herunterladen und Auspacken übernehmen `curl` und `tar` (Windows 10/11 und Linux bringen beide mit), die Prüfsumme wird hier berechnet.
use std::path::{Path, PathBuf};
use std::process::Command;

/// Form des Pakets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// ZIP-Archiv (wird mit `tar` ausgepackt; Windows 10/11 bringt ein `tar`, das ZIP kann)
    Zip,
    /// `.tar.gz`
    TarGz,
    /// einzelne ausführbare Datei (AppImage): wird nur gespeichert und ausführbar gemacht
    File,
}

impl Format {
    pub fn parse(s: &str) -> Option<Format> { match s { "zip" => Some(Format::Zip), "tar.gz" => Some(Format::TarGz), "file" => Some(Format::File), _ => None } }
}

/// Ein ladbares Paket
#[derive(Debug, Clone)]
pub struct Package {
    /// Kennung des Emulators (Ordnername), z. B. `atari800`
    pub id: String,
    pub version: String,
    pub url: String,
    /// SHA-256 des Pakets (Kleinbuchstaben-Hex)
    pub sha256: String,
    pub format: Format,
    /// ungefähre Größe in Byte (nur zur Anzeige)
    pub size: u64,
}

impl Package {
    /// Dateiname aus der Adresse
    pub fn file_name(&self) -> String { self.url.rsplit('/').next().unwrap_or("paket").split('?').next().unwrap_or("paket").to_string() }
}

/// Wurzel aller eigenen Emulator-Installationen; `VC_EMU_DIR` überschreibt sie
pub fn root() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("VC_EMU_DIR").filter(|d| !d.is_empty()) { return Some(d.into()); }
    let base = if cfg!(windows) { std::env::var_os("APPDATA").map(PathBuf::from)? }
        else if cfg!(target_os = "macos") { PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support") }
        else { std::env::var_os("XDG_CONFIG_HOME").filter(|d| !d.is_empty()).map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))? };
    Some(base.join("VintageCoder").join("emulators"))
}

/// Ordner eines Emulators (alle Versionen); für Fujisan überschreibt `VC_FUJISAN_DIR` ihn (Tests)
pub fn dir_of(id: &str) -> Option<PathBuf> {
    if id == "fujisan" { if let Some(d) = std::env::var_os("VC_FUJISAN_DIR").filter(|d| !d.is_empty()) { return Some(d.into()); } }
    Some(root()?.join(id))
}

/// Zielordner der Version eines Pakets
pub fn target_dir(p: &Package) -> Result<PathBuf, String> { Ok(dir_of(&p.id).ok_or("Konfigurationsordner nicht ermittelbar")?.join(&p.version)) }

/// Vergleicht Versionsangaben nach Zahlen statt nach Text: „3.10“ ist neuer als „3.9“, „20261104234052-next“ neuer als „20260928222910-next“.
pub fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    fn chunks(s: &str) -> Vec<&str> { s.split(|c: char| !c.is_ascii_alphanumeric()).filter(|c| !c.is_empty()).collect() }
    let (ca, cb) = (chunks(a), chunks(b));
    for (x, y) in ca.iter().zip(cb.iter()) {
        let (nx, ny) = (x.chars().all(|c| c.is_ascii_digit()), y.chars().all(|c| c.is_ascii_digit()));
        let o = if nx && ny { let (tx, ty) = (x.trim_start_matches('0'), y.trim_start_matches('0')); tx.len().cmp(&ty.len()).then(tx.cmp(ty)) } else { x.cmp(y) };
        if o != std::cmp::Ordering::Equal { return o; }
    }
    ca.len().cmp(&cb.len())
}

/// Installierte Versionen eines Emulators (Ordnername, Ordner), die neueste zuerst
pub fn versions(id: &str) -> Vec<(String, PathBuf)> {
    let Some(dir) = dir_of(id) else { return vec![] };
    let mut v: Vec<(String, PathBuf)> = std::fs::read_dir(dir).map(|rd| rd.flatten().filter(|e| e.path().is_dir()).map(|e| (e.file_name().to_string_lossy().to_string(), e.path())).collect()).unwrap_or_default();
    v.sort_by(|a, b| version_cmp(&b.0, &a.0));
    v
}

/// Sucht ein Programm (`name`, unter Windows mit `.exe`) in der neuesten installierten Version von `id`, bis Tiefe 4. Ordner von FujiNet-PC
/// (`fujinet*`) werden übergangen.
pub fn find_exe(id: &str, name: &str) -> Option<PathBuf> { versions(id).iter().find_map(|(_, d)| find_in(d, name, 4)) }

/// Wie `find_exe`, für eine bestimmte Version
pub fn find_exe_version(id: &str, version: &str, name: &str) -> Option<PathBuf> {
    versions(id).iter().find(|(v, _)| v == version).and_then(|(_, d)| find_in(d, name, 4))
}

static PREFERRED: std::sync::Mutex<Vec<(String, String)>> = std::sync::Mutex::new(Vec::new());

/// Wählt für diesen Prozess eine bestimmte installierte Version (`vcc test --backend vice@3.9`): `find_program` nimmt sie vor allem anderen
pub fn prefer(id: &str, version: &str) {
    let mut p = PREFERRED.lock().unwrap();
    p.retain(|(i, _)| i != id);
    p.push((id.to_string(), version.to_string()));
}

/// Programm aus einer mit `prefer` gewählten Version
pub fn find_preferred(name: &str) -> Option<PathBuf> {
    let p = PREFERRED.lock().unwrap().clone();
    p.iter().find_map(|(id, v)| find_exe_version(id, v, name))
}

/// Sucht ein Programm in allen eigenen Installationen (jeder Emulator, neueste Version zuerst)
pub fn find_any(name: &str) -> Option<PathBuf> {
    let root = root()?;
    let mut ids: Vec<_> = std::fs::read_dir(&root).ok()?.flatten().filter(|e| e.path().is_dir()).map(|e| e.file_name().to_string_lossy().to_string()).collect();
    ids.sort();
    ids.iter().find_map(|id| find_exe(id, name))
}

/// Sucht `name` in einem ausgepackten Paket (Groß-/Kleinschreibung egal; unter Windows ohne Endung mit `.exe`)
pub fn find_in(dir: &Path, name: &str, depth: u32) -> Option<PathBuf> {
    let want = if cfg!(windows) && !name.contains('.') { format!("{name}.exe") } else { name.to_string() };
    let mut subs = vec![];
    for e in std::fs::read_dir(dir).ok()?.flatten() {
        let p = e.path();
        let n = e.file_name().to_string_lossy().to_string();
        if p.is_file() { if n.eq_ignore_ascii_case(&want) { return Some(p); } }
        else if depth > 0 && !n.to_ascii_lowercase().starts_with("fujinet") { subs.push(p); }
    }
    subs.sort();
    subs.into_iter().find_map(|d| find_in(&d, name, depth - 1))
}

// ---- SHA-256 (FIPS 180-4) ----

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// SHA-256 einer Datei als Kleinbuchstaben-Hex
pub fn sha256_file(path: &Path) -> Result<String, String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop { let n = f.read(&mut buf).map_err(|e| e.to_string())?; if n == 0 { break; } h.update(&buf[..n]); }
    Ok(h.finish_hex())
}

pub fn sha256_hex(data: &[u8]) -> String { let mut h = Sha256::new(); h.update(data); h.finish_hex() }

struct Sha256 { s: [u32; 8], buf: Vec<u8>, len: u64 }

impl Sha256 {
    fn new() -> Self { Sha256 { s: [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19], buf: vec![], len: 0 } }
    fn update(&mut self, d: &[u8]) {
        self.len += d.len() as u64;
        self.buf.extend_from_slice(d);
        let n = self.buf.len() / 64 * 64;
        let blocks: Vec<u8> = self.buf.drain(..n).collect();
        for b in blocks.chunks(64) { self.block(b); }
    }
    fn block(&mut self, b: &[u8]) {
        let mut w = [0u32; 64];
        for i in 0..16 { w[i] = u32::from_be_bytes([b[i * 4], b[i * 4 + 1], b[i * 4 + 2], b[i * 4 + 3]]); }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut v = self.s;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [t1.wrapping_add(t2), v[0], v[1], v[2], v[3].wrapping_add(t1), v[4], v[5], v[6]];
        }
        for i in 0..8 { self.s[i] = self.s[i].wrapping_add(v[i]); }
    }
    fn finish_hex(mut self) -> String {
        let bits = self.len * 8;
        let mut pad = vec![0x80u8];
        while (self.buf.len() + pad.len()) % 64 != 56 { pad.push(0); }
        pad.extend_from_slice(&bits.to_be_bytes());
        let tail = std::mem::take(&mut self.buf);
        let mut all = tail; all.extend(pad);
        for b in all.chunks(64) { self.block(b); }
        self.s.iter().map(|x| format!("{x:08x}")).collect()
    }
}

// ---- Herunterladen und Auspacken ----

/// Programm der Systemwerkzeuge: unter Windows die des Systems (nicht ein `tar` aus Git, das keine ZIP-Dateien auspackt)
fn tool(name: &str) -> PathBuf {
    if cfg!(windows) {
        if let Some(root) = std::env::var_os("SystemRoot") { let p = Path::new(&root).join("System32").join(format!("{name}.exe")); if p.is_file() { return p; } }
    }
    PathBuf::from(name)
}

/// Prüft eine heruntergeladene Datei gegen die Prüfsumme des Pakets; bei Abweichung wird sie gelöscht
pub fn verify(pkg: &Package, file: &Path) -> Result<(), String> {
    let got = sha256_file(file)?;
    if !got.eq_ignore_ascii_case(&pkg.sha256) { let _ = std::fs::remove_file(file); return Err(format!("Prüfsumme stimmt nicht (erwartet {}, erhalten {got}); die Datei wurde gelöscht", pkg.sha256)); }
    Ok(())
}

/// Packt eine geprüfte Datei nach `dir` aus (bzw. legt sie dort ab)
pub fn unpack(pkg: &Package, file: &Path, dir: &Path) -> Result<(), String> {
    match pkg.format {
        Format::Zip | Format::TarGz => {
            let st = Command::new(tool("tar")).arg("-xf").arg(file).arg("-C").arg(dir).status().map_err(|e| format!("tar nicht ausführbar: {e}"))?;
            if !st.success() { return Err(format!("Auspacken fehlgeschlagen ({st})")); }
            Ok(())
        }
        Format::File => {
            let dst = dir.join(pkg.file_name());
            if file != dst { std::fs::copy(file, &dst).map_err(|e| e.to_string())?; }
            #[cfg(unix)]
            { use std::os::unix::fs::PermissionsExt; let _ = std::fs::set_permissions(&dst, std::fs::Permissions::from_mode(0o755)); }
            Ok(())
        }
    }
}

/// Lädt das Paket, prüft die Prüfsumme und packt es unterhalb von VintageCoder aus. `say` erhält Fortschrittsmeldungen.
/// `have` sagt, ob die Version schon installiert ist (dann geschieht nichts). Gibt den Ordner der Version zurück.
pub fn install(pkg: &Package, have: &dyn Fn(&Path) -> bool, say: &mut dyn FnMut(&str)) -> Result<PathBuf, String> {
    if pkg.sha256.len() != 64 { return Err("Für dieses Paket ist keine Prüfsumme hinterlegt; es wird nicht geladen".into()); }
    let dir = target_dir(pkg)?;
    if have(&dir) { say(&format!("{} {} ist bereits installiert: {}", pkg.id, pkg.version, dir.display())); return Ok(dir); }
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let file = dir.join(pkg.file_name());
    say(&format!("Lade {}", pkg.url));
    let st = Command::new(tool("curl")).args(["--fail", "--location", "--silent", "--show-error", "--output"]).arg(&file).arg(&pkg.url).status().map_err(|e| format!("curl nicht ausführbar: {e}"))?;
    if !st.success() { let _ = std::fs::remove_file(&file); return Err(format!("Download fehlgeschlagen ({st})")); }
    say("Prüfe SHA-256 …");
    verify(pkg, &file)?;
    say("Packe aus …");
    unpack(pkg, &file, &dir)?;
    if pkg.format != Format::File { let _ = std::fs::remove_file(&file); }
    say(&format!("Fertig: {}", dir.display()));
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_bekannte_werte() {
        assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        // Länge über eine Blockgrenze (56 Byte erzwingt einen zweiten Block für die Länge)
        assert_eq!(sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"), "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
        assert_eq!(sha256_hex(&vec![b'a'; 1_000_000]), "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0");
    }

    fn pkg(id: &str, url: &str, sha: &str, format: Format) -> Package { Package { id: id.into(), version: "1.0".into(), url: url.into(), sha256: sha.into(), format, size: 0 } }

    #[test]
    fn versionen_werden_nach_zahlen_verglichen() {
        use std::cmp::Ordering::*;
        assert_eq!(version_cmp("3.10", "3.9"), Greater);
        assert_eq!(version_cmp("7.2.1", "7.10.0"), Less);
        assert_eq!(version_cmp("4.40", "4.31"), Greater);
        assert_eq!(version_cmp("20261104234052-next", "20260928222910-next"), Greater);
        assert_eq!(version_cmp("2.0.5", "2.0.5"), Equal);
        assert_eq!(version_cmp("2.0", "2.0.1"), Less);
    }

    #[test]
    fn dateiname_aus_der_adresse() {
        assert_eq!(pkg("x", "https://h/a/b/Paket-1.zip?viasf=1", "", Format::Zip).file_name(), "Paket-1.zip");
    }

    /// Auspacken und Prüfsumme ohne Netz: ein ZIP und eine Einzeldatei aus dem Temp-Ordner
    #[test]
    fn auspacken_pruefen_und_suchen() {
        let base = std::env::temp_dir().join(format!("vc-install-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let src = base.join("src"); std::fs::create_dir_all(src.join("bin")).unwrap();
        std::fs::write(src.join("bin").join(if cfg!(windows) { "tool.exe" } else { "tool" }), b"programm").unwrap();
        let zip = base.join("t.zip");
        let ok = Command::new(tool("tar")).args(["-a", "-c", "-f"]).arg(&zip).arg("-C").arg(&src).arg(".").status().map(|s| s.success()).unwrap_or(false);
        if ok {
            let sha = sha256_file(&zip).unwrap();
            let p = pkg("t", "file:///t.zip", &sha, Format::Zip);
            let out = base.join("out"); std::fs::create_dir_all(&out).unwrap();
            verify(&p, &zip).unwrap();
            unpack(&p, &zip, &out).unwrap();
            assert!(find_in(&out, "tool", 4).is_some());
            // falsche Prüfsumme: Datei wird gelöscht
            let bad = pkg("t", "file:///t.zip", &"0".repeat(64), Format::Zip);
            assert!(verify(&bad, &zip).unwrap_err().contains("Prüfsumme"));
            assert!(!zip.exists());
        } else { eprintln!("tar kann kein ZIP erzeugen: Teil übersprungen"); }
        let f = base.join("App.AppImage"); std::fs::write(&f, b"x").unwrap();
        let sha = sha256_file(&f).unwrap();
        let p = pkg("a", "https://h/App.AppImage", &sha, Format::File);
        let out = base.join("out2"); std::fs::create_dir_all(&out).unwrap();
        unpack(&p, &f, &out).unwrap();
        assert!(out.join("App.AppImage").is_file());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn ohne_pruefsumme_wird_nichts_geladen() {
        let p = pkg("t", "https://h/x.zip", "", Format::Zip);
        assert!(install(&p, &|_| false, &mut |_| {}).unwrap_err().contains("Prüfsumme"));
    }
}
