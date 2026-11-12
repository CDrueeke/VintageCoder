// SPDX-License-Identifier: GPL-3.0-or-later
//! VintageCoder Language Server als Bibliothek (wird von `vc-lsp` und `vc-ide` genutzt).
mod docs;
pub use docs::{directive_doc, mnemonic_doc, DIRECTIVES};
pub mod rpc;
mod server;
mod values;
pub use values::{literal_at, value_hover};

pub use server::Server;
