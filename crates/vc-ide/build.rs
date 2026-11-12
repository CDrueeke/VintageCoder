// SPDX-License-Identifier: GPL-3.0-or-later
//! Legt den Git-Commit des Baus als `VC_BUILD_COMMIT` ab (für `feedback`: Version und Commit in der Mail). `VC_COMMIT` überschreibt ihn
//! (Bau aus einem Quellarchiv ohne `.git`); ohne beides bleibt er leer. Es wird nichts geschrieben und nichts abgerufen.
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let o = Command::new("git").args(args).output().ok()?;
    if !o.status.success() { return None; }
    let s = String::from_utf8(o.stdout).ok()?.trim().to_string();
    (!s.is_empty()).then_some(s)
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=VC_COMMIT");
    let commit = std::env::var("VC_COMMIT").ok().map(|c| c.trim().to_string()).filter(|c| !c.is_empty()).or_else(|| git(&["rev-parse", "--short=7", "HEAD"])).unwrap_or_default();
    // neu bauen, wenn sich der Commit ändert: HEAD und die Referenz, auf die er zeigt
    if let Some(p) = git(&["rev-parse", "--git-path", "HEAD"]) { println!("cargo:rerun-if-changed={p}"); }
    if let Some(r) = git(&["symbolic-ref", "-q", "HEAD"]) { if let Some(p) = git(&["rev-parse", "--git-path", &r]) { println!("cargo:rerun-if-changed={p}"); } }
    println!("cargo:rustc-env=VC_BUILD_COMMIT={commit}");
}
