// SPDX-License-Identifier: GPL-3.0-or-later
//! `vc-lsp` – Language Server (LSP über stdio) für VintageCoder-Assembler.
use vc_lsp::{rpc, Server};

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("-h") | Some("--help") => {
            println!("vc-lsp {} - Language Server für VintageCoder-Assembler\n", env!("CARGO_PKG_VERSION"));
            println!("Spricht das Language Server Protocol über stdin und stdout (Nachrichten mit Content-Length).");
            println!("Wird normalerweise vom Editor gestartet, nicht von Hand: ohne Eingabe wartet er auf Nachrichten und gibt nichts aus.\n");
            println!("Aufruf:  vc-lsp [--help] [--version]\n");
            println!("Einbinden im Editor: Befehl \"vc-lsp\" ohne Argumente, Dateien mit der Endung .asm.");
            println!("Rückgabe: 0 nach geordnetem Beenden (shutdown, exit), sonst 1.");
            return;
        }
        Some("-V") | Some("--version") => { println!("vc-lsp {}", env!("CARGO_PKG_VERSION")); return; }
        Some(other) => { eprintln!("vc-lsp: unbekannte Option „{other}“\nAufruf:  vc-lsp [--help] [--version]"); std::process::exit(2); }
        None => {}
    }
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let mut out = std::io::stdout().lock();
    let mut srv = Server::new();
    while let Ok(Some(msg)) = rpc::read_message(&mut input) {
        for reply in srv.handle(msg) {
            if rpc::write_message(&mut out, &reply).is_err() { return; }
        }
        if srv.exit { std::process::exit(if srv.shutdown { 0 } else { 1 }); }
    }
}
