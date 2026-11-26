// SPDX-License-Identifier: GPL-3.0-or-later
//! Sprache der Oberfläche (i18n, Etappe I0): Wahl in den Einstellungen und im Assistenten, Anwenden beim Start und nach dem Wechsel.
//! Die Wahl steht in `Settings::language` (`de`, `en`, leer = automatisch); `VC_LANG` überstimmt sie. Im Testlauf gilt Deutsch, wenn `VC_LANG` fehlt.
use crate::setui::{karte, wahl};
use crate::{App, Msg, Pal};
use iced::Element;
use vc_i18n::{Choice, Lang};

/// Sprache nach der Wahl bestimmen und setzen (im Testlauf nur für den Test-Thread, damit parallele Tests einander nicht stören)
pub(crate) fn sprache_setzen(wahl: Choice) -> Lang {
    let l = if cfg!(test) && std::env::var_os("VC_LANG").is_none() && wahl == Choice::Auto { Lang::De } else { vc_i18n::resolve(wahl) };
    if cfg!(test) { vc_i18n::set_thread_language(Some(l)) } else { vc_i18n::set_language(l) }
    l
}

impl App {
    /// Gewählte Sprache aus den gespeicherten Einstellungen
    pub(crate) fn sprache_wahl(&self) -> Choice { Choice::from_code(&self.satz.gespeichert.language) }
    /// Beim Start: Sprache setzen und fehlende Schlüssel ins Debug-Protokoll schreiben
    pub(crate) fn sprache_anwenden(&self) { sprache_setzen(self.sprache_wahl()); }
    /// Nach einem Wechsel: neue Sprache setzen; die Oberfläche zeichnet sich von selbst neu, die Menüleiste des Systems (macOS) wird neu aufgebaut
    pub(crate) fn sprache_geaendert(&mut self) {
        self.sprache_anwenden();
        #[cfg(target_os = "macos")] if !cfg!(test) { crate::native::neu_aufbauen() }
    }
    /// Auswahlzeile Automatisch / Deutsch / English; `aktuell` = Wert aus den Einstellungen (leer = automatisch)
    pub(crate) fn sprache_wahl_zeile<'a>(&self, pal: Pal, aktuell: &str, m: impl Fn(String) -> Msg + 'a) -> Element<'a, Msg> {
        let auto = vc_i18n::t!("settings-language-auto", lang = vc_i18n::system_language().native_name());
        let wert = if aktuell.is_empty() { "auto" } else { aktuell };
        wahl(pal, &[("auto", auto.as_str()), ("de", Lang::De.native_name()), ("en", Lang::En.native_name())], wert, m)
    }
    pub(crate) fn sprache_karte<'a>(&self, pal: Pal, aktuell: &str, m: impl Fn(String) -> Msg + 'a) -> Element<'a, Msg> { karte(pal, vec![self.sprache_wahl_zeile(pal, aktuell, m)]) }
}
