// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests für Assembler-Projekte. Stufe 1: statische Prüfungen aus `*.vctest`,
//! ohne Emulator (Segmentgrößen, Zeropage, Überlappung, Zyklenspanne aus der `.vcdbg`).
pub mod model;
pub mod cycles;
pub mod junit;
pub mod mailbox;
pub mod runner;
pub mod static_checks;
pub mod suite;

pub use model::{find_tests, Level, Outcome, RegSnap, Report, State, StaticCheck, TestCase, TestFile, TestMode, MailboxCase, MeasureCase};
pub use static_checks::{run_file, Options, Prepared};
