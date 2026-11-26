// SPDX-License-Identifier: GPL-3.0-or-later
//! Sicherheit: Pfade aus einer fremden `.vcprj` dürfen nichts außerhalb des Projektordners lesen oder schreiben.
use std::path::PathBuf;
use vc_ide::build::Ctx;
use vc_ide::projfile::*;

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vc_safepath_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d
}
fn data() -> PathBuf { vc_ide::find_data_dir(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))).unwrap() }
const PRG: &str = ".target c64\n.org $c000\nstart: lda #1\n sta $d020\n rts\n";

fn projekt(root: &PathBuf, disk_file: &str) -> ProjectFile {
    std::fs::write(root.join("a.asm"), PRG).unwrap();
    ProjectFile { targets: vec![TargetDef { id: "c64".into(), ..Default::default() }], main: Some("a.asm".into()),
        sources: vec![Source { targets: vec!["c64".into()], kind: "asm".into(), files: vec!["a.asm".into()], ..Default::default() }],
        disk: Some(DiskSpec { format: "d64".into(), file: disk_file.into(), ..Default::default() }), ..Default::default() }
}

#[test]
fn fremde_diskettenpfade_schreiben_nichts() {
    let aussen = tmp("aussen");
    let ziel = aussen.join("opfer.d64");
    let root = tmp("proj");
    let abs = ziel.to_string_lossy().into_owned();
    let hoch = format!("../{}/opfer.d64", aussen.file_name().unwrap().to_string_lossy());
    for pfad in [abs.as_str(), hoch.as_str(), "C:\\Windows\\x.d64", "\\\\server\\share\\x.d64", "..\\x.d64", "build/../../x.d64", "/etc/x.d64"] {
        let pf = projekt(&root, pfad);
        let data = data();
        let r = build_project(&Ctx { root: &root, data: &data }, &pf, None);
        assert_eq!(r["ok"], false, "{pfad}: {r}");
        assert!(r.to_string().contains("Unzul") || r.to_string().contains("Invalid"), "{pfad}: klare Meldung erwartet: {r}");
        assert!(!ziel.exists(), "{pfad}: außerhalb geschrieben!");
        assert!(!root.join("build").exists(), "{pfad}: Bau muss vor dem Schreiben abbrechen");
        assert!(!pf.validate(&root).is_empty(), "{pfad}: validate meldet es");
        let spec = pf.disk.clone().unwrap();
        assert!(make_disk(&root, &pf, &spec, &[("a.asm".into(), "build/a.prg".into())]).is_err(), "{pfad}: make_disk");
        assert!(!ziel.exists());
    }
}

#[cfg(unix)]
#[test]
fn symlink_aus_dem_projekt_heraus_wird_nicht_beschrieben() {
    let aussen = tmp("sym-aussen");
    let root = tmp("sym-proj");
    std::os::unix::fs::symlink(&aussen, root.join("build")).unwrap();
    let pf = projekt(&root, "build/x.d64");
    let data = data();
    let r = build_project(&Ctx { root: &root, data: &data }, &pf, None);
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(std::fs::read_dir(&aussen).unwrap().count(), 0, "nichts im Zielordner des Links");
}

#[test]
fn gueltiger_unterordner_bleibt_erlaubt() {
    let root = tmp("gut");
    let pf = projekt(&root, "ausgabe/neu/x.d64");
    let data = data();
    let r = build_project(&Ctx { root: &root, data: &data }, &pf, None);
    assert_eq!(r["ok"], true, "{r}");
    assert!(root.join("ausgabe/neu/x.d64").is_file());
}

#[test]
fn convert_out_und_asset_und_turbo_pfade_werden_geprueft() {
    let root = tmp("andere");
    let mut pf = projekt(&root, "");
    pf.disk = None;
    pf.sources[0].convert = Some(ConvertSpec { out: "../raus.asm".into(), ..Default::default() });
    assert!(pf.unsafe_paths(&root).len() == 1, "convert.out");
    pf.sources[0].convert = None;
    pf.assets.push(Asset { file: "/etc/passwd".into(), category: "binary".into(), description: String::new() });
    pf.turbo = Some(TurboSpec { disk: "..\\x.atr".into(), files: vec!["/etc/hosts:HOSTS".into(), "C:\\x\\y.bin".into()], ..Default::default() });
    assert_eq!(pf.unsafe_paths(&root).len(), 4, "{:?}", pf.unsafe_paths(&root));
    let r = vc_ide::turbo::make_turbo_disk(&root, &pf, "a.asm", "build/a.prg");
    assert!(r.is_err());
    assert!(vc_ide::turbo::reduce_project_copy(&root).is_err());
    let req = { let mut q = vc_ide::b2a::Req::new("a.bas"); q.out = "../x.asm".into(); q };
    std::fs::write(root.join("a.bas"), "10 PRINT 1\n").unwrap();
    assert!(vc_ide::b2a::convert_file(&root, &data(), &req).is_err());
    assert!(!root.parent().unwrap().join("x.asm").exists());
}
