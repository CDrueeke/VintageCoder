// SPDX-License-Identifier: GPL-3.0-or-later
//! Alle sichtbaren Texte an einer Stelle (Konzept Abschnitt 13): Deutsch, du-Form, Infinitiv auf Knöpfen. Kein Text im Widgetcode.
pub fn app_titel() -> String { vc_i18n::t!("texte-app-titel") }
pub fn starten() -> String { vc_i18n::t!("texte-starten") }
pub fn debuggen() -> String { vc_i18n::t!("texte-debuggen") }
pub fn bauen() -> String { vc_i18n::t!("texte-bauen") }
pub fn baut() -> String { vc_i18n::t!("texte-baut") }
pub fn suchfeld() -> String { vc_i18n::t!("texte-suchfeld") }
pub fn dateien() -> String { vc_i18n::t!("texte-dateien") }
pub fn suche() -> String { vc_i18n::t!("texte-suche") }
pub fn teilen_mind_zwei() -> String { vc_i18n::t!("texte-teilen-mind-zwei") }
pub fn ablegen_rechts() -> String { vc_i18n::t!("texte-ablegen-rechts") }
pub fn streifen() -> String { vc_i18n::t!("texte-streifen") }
pub fn ausgeklappt() -> String { vc_i18n::t!("texte-ausgeklappt") }
pub fn eingeklappt() -> String { vc_i18n::t!("texte-eingeklappt") }
pub fn spalte() -> String { vc_i18n::t!("texte-spalte") } pub fn gruppe_export() -> String { vc_i18n::t!("texte-gruppe-export") }
pub fn dokumente() -> String { vc_i18n::t!("texte-dokumente") }
pub fn schublade() -> String { vc_i18n::t!("texte-schublade") }
pub fn panel() -> String { vc_i18n::t!("texte-panel") }
pub fn normal() -> String { vc_i18n::t!("texte-normal") }
pub fn minimiert() -> String { vc_i18n::t!("texte-minimiert") }
pub fn maximiert() -> String { vc_i18n::t!("texte-maximiert") }
/// Name eines Bereichsknopfs für Hinweis und Bildschirmleser, z. B. „Schublade maximieren“
pub fn knopfname(bereich: &str, aktion: &str) -> String { format!("{bereich} {aktion}") }
pub fn hilfe_titel() -> String { vc_i18n::t!("texte-hilfe-titel") }
pub fn zeichen() -> String { vc_i18n::t!("texte-zeichen") }
pub fn hilfe_suche() -> String { vc_i18n::t!("texte-hilfe-suche") }
pub fn hilfe_zurueck() -> String { vc_i18n::t!("texte-hilfe-zurueck") }
pub fn hilfe_nichts() -> String { vc_i18n::t!("texte-hilfe-nichts") }
pub fn zeichen_suche() -> String { vc_i18n::t!("texte-zeichen-suche") }
pub fn zeichen_nicht_hier() -> String { vc_i18n::t!("texte-zeichen-nicht-hier") }
pub fn art_asm() -> String { vc_i18n::t!("texte-art-asm") }
pub fn art_basic() -> String { vc_i18n::t!("texte-art-basic") }
pub fn art_text() -> String { vc_i18n::t!("texte-art-text") }
pub fn art_notiz() -> String { vc_i18n::t!("texte-art-notiz") }
pub fn dlg_neue_datei() -> String { vc_i18n::t!("texte-dlg-neue-datei") }
pub fn dlg_neuer_ordner() -> String { vc_i18n::t!("texte-dlg-neuer-ordner") }
pub fn dlg_umbenennen() -> String { vc_i18n::t!("texte-dlg-umbenennen") }
pub fn dlg_loeschen() -> String { vc_i18n::t!("texte-dlg-loeschen") }
pub fn dlg_art() -> String { vc_i18n::t!("texte-dlg-art") }
pub fn dlg_name() -> String { vc_i18n::t!("texte-dlg-name") }
pub fn dlg_profil() -> String { vc_i18n::t!("texte-dlg-profil") }
pub fn baum_pfad() -> String { vc_i18n::t!("texte-baum-pfad") }
pub fn baum_ok() -> String { vc_i18n::t!("texte-baum-ok") }
pub fn baum_abbruch() -> String { vc_i18n::t!("texte-baum-abbruch") }
pub fn baum_frage_korb() -> String { vc_i18n::t!("texte-baum-frage-korb") }
pub fn baum_frage_endgueltig() -> String { vc_i18n::t!("texte-baum-frage-endgueltig") }
pub fn baum_keine_datei() -> String { vc_i18n::t!("texte-baum-keine-datei") }
pub fn protokoll() -> String { vc_i18n::t!("texte-protokoll") }
pub fn probleme() -> String { vc_i18n::t!("texte-probleme") }
pub fn bereit() -> String { vc_i18n::t!("texte-bereit") }
pub fn keine_datei() -> String { vc_i18n::t!("texte-keine-datei") }
pub fn kein_projekt() -> String { vc_i18n::t!("texte-kein-projekt") }
pub fn suche_leer() -> String { vc_i18n::t!("texte-suche-leer") }
pub fn suche_platzhalter() -> String { vc_i18n::t!("texte-suche-platzhalter") }
pub fn finden() -> String { vc_i18n::t!("texte-finden") }
pub fn ersetzen() -> String { vc_i18n::t!("texte-ersetzen") }
pub fn alle() -> String { vc_i18n::t!("texte-alle") }
pub fn zeile() -> String { vc_i18n::t!("texte-zeile") }
pub fn treffer() -> String { vc_i18n::t!("texte-treffer") }
pub fn kein_treffer() -> String { vc_i18n::t!("texte-kein-treffer") }
pub fn gliederung() -> String { vc_i18n::t!("texte-gliederung") }
pub fn gliederung_leer() -> String { vc_i18n::t!("texte-gliederung-leer") }
pub fn einstellungen() -> String { vc_i18n::t!("texte-einstellungen") }
pub fn farbschema() -> String { vc_i18n::t!("texte-farbschema") }
pub fn dunkel() -> String { vc_i18n::t!("texte-dunkel") }
pub fn hell() -> String { vc_i18n::t!("texte-hell") }
pub fn system() -> String { vc_i18n::t!("texte-system") }
pub fn kontrast() -> String { vc_i18n::t!("texte-kontrast") }
pub fn oberflaeche() -> String { vc_i18n::t!("texte-oberflaeche") }
pub fn editor_schrift() -> String { vc_i18n::t!("texte-editor-schrift") }
pub fn umbruch() -> String { vc_i18n::t!("texte-umbruch") }
#[cfg(test)] mod tests { use super::*;
    #[test] fn texte_nicht_leer() { for t in [starten(), debuggen(), bauen(), suchfeld(), keine_datei(), gliederung_leer(), farbschema()] { assert!(!t.trim().is_empty() && !t.contains('⟦')) } }
}

// Debuggen (Panel)
pub fn debuggen_panel() -> String { vc_i18n::t!("texte-debuggen-panel") }
pub fn dbg_weiter() -> String { vc_i18n::t!("texte-dbg-weiter") } pub fn dbg_pause() -> String { vc_i18n::t!("texte-dbg-pause") } pub fn dbg_ende() -> String { vc_i18n::t!("texte-dbg-ende") }
pub fn dbg_ueber() -> String { vc_i18n::t!("texte-dbg-ueber") } pub fn dbg_hinein() -> String { vc_i18n::t!("texte-dbg-hinein") } pub fn dbg_raus() -> String { vc_i18n::t!("texte-dbg-raus") }
pub fn dbg_startet() -> String { vc_i18n::t!("texte-dbg-startet") } pub fn dbg_laeuft() -> String { vc_i18n::t!("texte-dbg-laeuft") } pub fn dbg_angehalten() -> String { vc_i18n::t!("texte-dbg-angehalten") } pub fn dbg_wartet() -> String { vc_i18n::t!("texte-dbg-wartet") }
pub fn dbg_register() -> String { vc_i18n::t!("texte-dbg-register") } pub fn dbg_disassembler() -> String { vc_i18n::t!("texte-dbg-disassembler") } pub fn dbg_speicher() -> String { vc_i18n::t!("texte-dbg-speicher") } pub fn dbg_beobachten() -> String { vc_i18n::t!("texte-dbg-beobachten") } pub fn dbg_haltepunkte() -> String { vc_i18n::t!("texte-dbg-haltepunkte") } pub fn dbg_ausdruck() -> String { vc_i18n::t!("texte-dbg-ausdruck") }
pub fn karte() -> String { vc_i18n::t!("texte-karte") } pub fn karte_leer() -> String { vc_i18n::t!("texte-karte-leer") }
pub fn dbg_moment() -> String { vc_i18n::t!("texte-dbg-moment") } pub fn dbg_moment_erkennen() -> String { vc_i18n::t!("texte-dbg-moment-erkennen") } pub fn dbg_moment_sprites() -> String { vc_i18n::t!("texte-dbg-moment-sprites") } pub fn dbg_moment_kacheln() -> String { vc_i18n::t!("texte-dbg-moment-kacheln") } pub fn dbg_moment_rom() -> String { vc_i18n::t!("texte-dbg-moment-rom") } pub fn dbg_moment_name() -> String { vc_i18n::t!("texte-dbg-moment-name") } pub fn dbg_moment_speichern() -> String { vc_i18n::t!("texte-dbg-moment-speichern") }
pub fn tests() -> String { vc_i18n::t!("texte-tests") } pub fn tests_leer() -> String { vc_i18n::t!("texte-tests-leer") } pub fn tests_wahl() -> String { vc_i18n::t!("texte-tests-wahl") } pub fn test_debug() -> String { vc_i18n::t!("texte-test-debug") }
pub fn emulator_bild() -> String { vc_i18n::t!("texte-emulator-bild") } pub fn emu_aktualisieren() -> String { vc_i18n::t!("texte-emu-aktualisieren") } pub fn emu_auto() -> String { vc_i18n::t!("texte-emu-auto") } pub fn emu_live() -> String { vc_i18n::t!("texte-emu-live") } pub fn emu_laeuft() -> String { vc_i18n::t!("texte-emu-laeuft") } pub fn emu_kein_bild() -> String { vc_i18n::t!("texte-emu-kein-bild") } pub fn emu_text() -> String { vc_i18n::t!("texte-emu-text") }
pub fn rq_titel() -> String { vc_i18n::t!("texte-rq-titel") } pub fn rq_beenden() -> String { vc_i18n::t!("texte-rq-beenden") } pub fn tests_parallel() -> String { vc_i18n::t!("texte-tests-parallel") }
pub fn debug_reiter() -> String { vc_i18n::t!("texte-debug-reiter") } pub fn dbg_reg_laeuft() -> String { vc_i18n::t!("texte-dbg-reg-laeuft") } pub fn dbg_speicher_titel() -> String { vc_i18n::t!("texte-dbg-speicher-titel") } pub fn dbg_speicher_feld() -> String { vc_i18n::t!("texte-dbg-speicher-feld") } pub fn dbg_keine_haltepunkte() -> String { vc_i18n::t!("texte-dbg-keine-haltepunkte") }
pub fn export_titel() -> String { vc_i18n::t!("texte-export-titel") } pub fn export_kein_studio() -> String { vc_i18n::t!("texte-export-kein-studio") } pub fn export_name() -> String { vc_i18n::t!("texte-export-name") } pub fn export_ladeadresse() -> String { vc_i18n::t!("texte-export-ladeadresse") } pub fn export_adresse() -> String { vc_i18n::t!("texte-export-adresse") } pub fn export_format() -> String { vc_i18n::t!("texte-export-format") } pub fn export_umfang() -> String { vc_i18n::t!("texte-export-umfang") } pub fn export_tab_text() -> String { vc_i18n::t!("texte-export-tab-text") } pub fn export_testen() -> String { vc_i18n::t!("texte-export-testen") } pub fn export_speichern() -> String { vc_i18n::t!("texte-export-speichern") } pub fn export_einfuegen() -> String { vc_i18n::t!("texte-export-einfuegen") }
pub fn import_titel() -> String { vc_i18n::t!("texte-import-titel") } pub fn import_hinweis() -> String { vc_i18n::t!("texte-import-hinweis") }
pub fn palette_hinweis() -> String { vc_i18n::t!("texte-palette-hinweis") }
