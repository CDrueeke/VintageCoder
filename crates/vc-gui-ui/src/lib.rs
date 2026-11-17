// SPDX-License-Identifier: GPL-3.0-or-later
//! VintageCoder-Oberfläche: gemeinsame Bausteine ohne Fensterlogik.
//!
//! **Verbindliche Regel (Entscheidung des Projektinhabers vom 2026-11-16, Konzept „UX-Überarbeitung“):** Jedes neue oder überarbeitete Studio
//! und jede neue Seite (Register-Studio, Bildaufbau, SID, Projekt-Einstellungen …) folgt dem gemeinsamen Muster: der Studio-Rahmen mit
//! sechs Zonen (Kopfzeile mit allen Datei-Aktionen, genau eine Werkzeugleiste, Liste, eingepasste Arbeitsfläche ohne Scrollbalken,
//! Seitenspalte aus Karten, Fußzeile), Karten „Werkzeug“ (nicht in der Datei) gegen „Dokument“ (gespeichert), Breitenklassen nach
//! Studiobreite (breit ≥ 900, mittel 640–899, schmal < 640 px), 4-px-Raster und nur Farbrollen aus `theme::Pal`. Ein Studio ohne den
//! Studio-Rahmen wird nicht angenommen. Einzelheiten und Bilder: `assets/docs/Konzept-UX-Ueberarbeitung.md` (lokal) und der Abschnitt
//! „Code und Sprache“ in `CONTRIBUTING.md`.
pub mod bereich;
pub mod fonts;
pub mod kontext;
pub mod theme;
pub mod befehle;
pub mod texte;
pub mod muster;
pub mod flaeche;
