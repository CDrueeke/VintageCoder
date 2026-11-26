// SPDX-License-Identifier: GPL-3.0-or-later
//! Native Menüleiste aus der Befehlstabelle `vc-commands` (Konzept 4.4). Neuaufbau beim Sprachwechsel; Einträge werden je nach Zustand
//! (Projekt offen, Editor aktiv) ein- oder ausgegraut.
use tauri::menu::{Menu, MenuBuilder, MenuItem, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::{AppHandle, Wry};
use vc_commands::dynamisch::{menuename, untermenue_titel_alle, untermenues_alle};
use vc_commands::{menue_buchstabe, menue_titel, Braucht, Zeile, MENUES};

/// Zustand, von dem das Ein- und Ausgrauen abhängt
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Kontext { pub projekt: bool, pub editor: bool, /** Turbo-BASIC-XL-Projekt: Debuggen ist ausgegraut (Konzept T7) */ pub turbo: bool }

/// Befehle des Debuggens (im Turbo-BASIC-XL-Projekt ausgegraut)
pub const DEBUG_BEFEHLE: &[&str] = &["debuggen", "debuggen_aktiv", "debug_weiter", "debug_pause", "debug_ueber", "debug_hinein", "debug_raus", "debug_hierher"];

impl Kontext {
    pub fn erlaubt(self, b: Braucht) -> bool { match b { Braucht::Nichts => true, Braucht::Projekt => self.projekt, Braucht::Editor => self.editor } }
    /// Wie [`Kontext::erlaubt`], dazu: im Turbo-BASIC-XL-Projekt sind die Debug-Befehle aus (Turbo-Programme lassen sich noch nicht schrittweise ausführen)
    pub fn erlaubt_id(self, id: &str, b: Braucht) -> bool { self.erlaubt(b) && !(self.turbo && DEBUG_BEFEHLE.contains(&id)) }
}

/// „Cmd+Umschalt+S“ → „CmdOrCtrl+Shift+S“ (Tauri-Schreibweise); `None`, wenn kein Kürzel
pub fn beschleuniger(tasten: &str) -> Option<String> {
    if tasten.is_empty() { return None; }
    let mut teile: Vec<&str> = tasten.split('+').collect();
    let taste = teile.pop()?;
    let mut out: Vec<String> = teile.iter().map(|m| match *m { "Cmd" => "CmdOrCtrl".to_string(), "Umschalt" => "Shift".into(), "Alt" => "Alt".into(), x => x.to_string() }).collect();
    out.push(match taste { "Plus" => "Plus".into(), "Minus" => "Minus".into(), "Komma" => ",".into(), "Rechts" => "Right".into(), "Links" => "Left".into(), t => t.to_string() });
    Some(out.join("+"))
}

/// Titel mit Alt-Buchstabe („&“ für muda auf Windows und Linux; macOS kennt das nicht)
fn mit_buchstabe(m: &str) -> String {
    let titel = menue_titel(m);
    if cfg!(target_os = "macos") { return titel; }
    let Some(c) = menue_buchstabe(m) else { return titel };
    let marke = '&';   // muda (Windows, Linux/GTK) liest „&“ als Alt-Buchstaben, ein „_“ erschiene als Zeichen
    let mut out = String::new(); let mut gesetzt = false;
    for z in titel.chars() { if !gesetzt && z.to_lowercase().next() == Some(c) { out.push(marke); gesetzt = true; } out.push(z); }
    out
}

type Eintraege = Vec<(String, MenuItem<Wry>)>;

fn eintrag(app: &AppHandle, i: &Zeile, k: Kontext, items: &mut Eintraege) -> tauri::Result<MenuItem<Wry>> {
    let mut b = MenuItemBuilder::with_id(i.id.clone(), menuename(i)).enabled(k.erlaubt_id(&i.id, i.braucht));
    if let Some(a) = beschleuniger(vc_commands::tasten_fuer(&i.id, &i.tasten, cfg!(target_os = "macos"))) { b = b.accelerator(a); }
    let it = b.build(app)?;
    items.push((i.id.clone(), it.clone()));
    Ok(it)
}

fn anhaengen<'a>(app: &'a AppHandle, m: &str, mut sb: SubmenuBuilder<'a, Wry, AppHandle>, unter: &str, zeilen: &[Zeile], k: Kontext, items: &mut Eintraege) -> tauri::Result<SubmenuBuilder<'a, Wry, AppHandle>> {
    for i in zeilen.iter().filter(|z| z.menue == m && z.unter == unter) {
        // Zwischenablage-Befehle gehören dem Webview (sonst wirken ⌘C/⌘V in Eingabefeldern nicht)
        sb = match i.id.as_str() {
            "ausschneiden" => sb.item(&PredefinedMenuItem::cut(app, Some(&menuename(i)))?),
            "kopieren" => sb.item(&PredefinedMenuItem::copy(app, Some(&menuename(i)))?),
            "einfuegen" => sb.item(&PredefinedMenuItem::paste(app, Some(&menuename(i)))?),
            "alle_auswaehlen" => sb.item(&PredefinedMenuItem::select_all(app, Some(&menuename(i)))?),
            _ => sb.item(&eintrag(app, i, k, items)?),
        };
    }
    Ok(sb)
}

/// Menüleiste neu bauen (nach Sprachwechsel oder Zustandswechsel); liefert auch die Einträge zum späteren Ein-/Ausgrauen
pub fn bauen(app: &AppHandle, k: Kontext) -> tauri::Result<(Menu<Wry>, Eintraege)> {
    let mut items = vec![];
    // Befehle nach Registry und Release-Auswahl: abgewählte Editoren und ihre Befehle fehlen; „Erweiterungen“ gibt es nur, wenn Pakete Befehle mitbringen
    let zeilen = crate::erweiterungen::zeilen_sichtbar(vc_i18n::language().code());
    let mut mb = MenuBuilder::new(app);
    // App-Menü (macOS): Über, Ausblenden; der Titel ist der Programmname
    let appm = SubmenuBuilder::new(app, "VintageCoder")
        .item(&PredefinedMenuItem::about(app, Some(&vc_i18n::t!("menu-app-about")), None)?)
        .separator()
        .item(&PredefinedMenuItem::hide(app, Some(&vc_i18n::t!("menu-app-hide")))?)
        .build()?;
    if cfg!(target_os = "macos") { mb = mb.item(&appm); }
    for m in MENUES {
        let mut sb = SubmenuBuilder::new(app, mit_buchstabe(m));
        sb = anhaengen(app, m, sb, "", &zeilen, k, &mut items)?;
        let unter: Vec<String> = untermenues_alle(m).into_iter().filter(|u| zeilen.iter().any(|z| z.menue == *m && z.unter == *u)).collect();   // keine leeren Untermenüs
        if zeilen.iter().any(|z| z.menue == *m && z.unter.is_empty()) && !unter.is_empty() { sb = sb.separator(); }
        for u in &unter {
            let mut us = SubmenuBuilder::new(app, untermenue_titel_alle(u));
            us = anhaengen(app, m, us, u, &zeilen, k, &mut items)?;
            sb = sb.item(&us.build()?);
        }
        mb = mb.item(&sb.build()?);
    }
    Ok((mb.build()?, items))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn beschleuniger_schreibweise() {
        assert_eq!(beschleuniger("Cmd+Umschalt+S").as_deref(), Some("CmdOrCtrl+Shift+S"));
        assert_eq!(beschleuniger("F7").as_deref(), Some("F7"));
        assert_eq!(beschleuniger("Cmd+Komma").as_deref(), Some("CmdOrCtrl+,"));
        assert_eq!(beschleuniger("Cmd+Alt+Rechts").as_deref(), Some("CmdOrCtrl+Alt+Right"));
        assert_eq!(beschleuniger(""), None);
    }
    #[test] fn kontext_graut_aus() {
        let k = Kontext::default();
        assert!(k.erlaubt(Braucht::Nichts) && !k.erlaubt(Braucht::Projekt) && !k.erlaubt(Braucht::Editor));
        assert!(Kontext { projekt: true, ..Default::default() }.erlaubt(Braucht::Projekt));
        let t = Kontext { projekt: true, turbo: true, ..Default::default() };
        assert!(!t.erlaubt_id("debuggen", Braucht::Projekt) && t.erlaubt_id("starten", Braucht::Projekt) && Kontext { projekt: true, ..Default::default() }.erlaubt_id("debuggen", Braucht::Projekt));
    }
}
