// SPDX-License-Identifier: GPL-3.0-or-later
//! `vc-lsp` – Language Server (LSP über stdio) für VintageCoder-Assembler.
use vc_core::msg;
use vc_lsp::{rpc, Server};

fn main() {
    let lang = vc_i18n::resolve(vc_i18n::Choice::Auto);
    let t = |m: vc_core::Msg| m.text_in(lang);
    let usage = t(msg!("lsp-cli-usage"));
    match std::env::args().nth(1).as_deref() {
        Some("-h") | Some("--help") => {
            println!("{}\n", t(msg!("lsp-cli-help-title", version = env!("CARGO_PKG_VERSION"))));
            println!("{}", t(msg!("lsp-cli-help-protocol")));
            println!("{}\n", t(msg!("lsp-cli-help-note")));
            println!("{usage}\n");
            println!("{}", t(msg!("lsp-cli-help-editor")));
            println!("{}", t(msg!("lsp-cli-help-exit")));
            return;
        }
        Some("-V") | Some("--version") => { println!("vc-lsp {}", env!("CARGO_PKG_VERSION")); return; }
        Some(other) => { eprintln!("{}\n{usage}", t(msg!("lsp-cli-unknown-option", option = other))); std::process::exit(2); }
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
