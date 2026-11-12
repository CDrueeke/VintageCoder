// SPDX-License-Identifier: GPL-3.0-or-later
//! ROM-Verzeichnis: Suchorte, Erkennung per Größe und Prüfsumme, Rollen je System, Import.
//!
//! Das Crate enthält und liefert **keine** ROM-Daten; `data/roms/known.json` führt nur Größen und Prüfsummen.
//! Konzept: `Konzept-ROM-Verzeichnis.md`. Regeln für den portablen Modus: siehe [`env`].
pub mod emit;
pub mod fallback;
pub mod env;
pub mod hash;
pub mod import;
pub mod known;
pub mod store;
pub mod systems;
pub mod zip;

pub use env::{Place, Portable, RomEnv, Stage};
pub use known::{KnownRom, KnownTable};
pub use store::{Found, Ident, Resolved, RomStore, Searched, Status};
