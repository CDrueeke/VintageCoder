// SPDX-License-Identifier: GPL-3.0-or-later
//! VintageCoder Language Server als Bibliothek (wird von `vc-lsp` und `vc-ide` genutzt).
mod docs;
pub use docs::{directive_doc, directive_doc_in, mnemonic_doc, mnemonic_doc_in, DirectiveTable, DIRECTIVES, directive_names, MNEMONIC_NAMES};
pub mod rpc;
mod project;
pub mod quickfix;
mod server;
mod values;
pub use values::{literal_at, value_hover, value_hover_in};

pub use server::Server;
