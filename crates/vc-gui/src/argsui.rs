// SPDX-License-Identifier: GPL-3.0-or-later
//! Startparameter aus dem Katalog (`data/args`) für Emulatoren und den Assembler (Stufe 4a): gruppierte Schalter, Auswahlen und Felder mit Hinweis,
//! freies Feld „Weitere Parameter“, anklickbare Platzhalter, Prüfmeldungen und Vorschau der fertigen Kommandozeile. Gerechnet wird in `vc_ide::args`;
//! hier wird nur die Auswahl im Entwurf geändert (`EmulatorCfg::opts`/`args`, `AsmCfg::opts`/`args`).
use crate::setui::{klein, SetAkt};
use crate::{knopf, App, Msg, Pal};
use iced::widget::{button, checkbox, column, container, row, text, text_input, tooltip};
use iced::{Alignment, Border, Element, Length};
use std::sync::Arc;
use vc_ide::args::{self, Catalog, Level, Param, ParamType, Selection, PLACEHOLDERS};

/// Besitzer der Auswahl: `"asm"` oder die Kennung eines Emulators
#[derive(Clone, Debug)]
pub enum ArgAkt {
    Wert(String, String, String), Frei(String, String), Platzhalter(String, String), Migrieren(String), Zurueck(String), Auf(String, bool),
}

fn a(x: ArgAkt) -> Msg { Msg::Set(SetAkt::Arg(x)) }

fn markiert(p: &Param) -> String { match &p.off_flag { Some(o) => format!("{} / {}", p.flag, o), None => p.flag.clone() } }
fn tipp_text(p: &Param) -> String {
    let mut s = vc_i18n::t!("argsui-p1-schalter-p2", p1 = &p.hint, p2 = markiert(p));
    if let Some(d) = &p.default { s.push_str(&format!("\n{}", vc_i18n::t!("argsui-standard-des-programms-d", d = &d))) }
    if let Some(x) = &p.example { s.push_str(&format!("\n{}", vc_i18n::t!("argsui-beispiel-x", x = &x))) }
    if p.min.is_some() || p.max.is_some() { s.push_str(&format!("\n{}", vc_i18n::t!("argsui-bereich-p1-bis-p2", p1 = p.min.map_or("–".into(), |m| m.to_string()), p2 = p.max.map_or("–".into(), |m| m.to_string())))) }
    if !p.conflicts.is_empty() { s.push_str(&format!("\n{}", vc_i18n::t!("argsui-nicht-zusammen-mit-p1", p1 = p.conflicts.join(", ")))) }
    if !p.requires.is_empty() { s.push_str(&format!("\n{}", vc_i18n::t!("argsui-braucht-p1", p1 = p.requires.join(", ")))) }
    s
}

impl App {
    /// Zeiger auf Auswahl und freies Feld eines Besitzers im Entwurf
    pub(crate) fn arg_ziel(&mut self, owner: &str) -> Option<(&mut Selection, &mut String)> {
        if owner == "asm" { let c = &mut self.satz.entwurf.assembler; return Some((&mut c.opts, &mut c.args)) }
        self.satz.entwurf.targets.iter_mut().flat_map(|t| t.emulators.iter_mut()).find(|e| e.id == owner).map(|e| (&mut e.opts, &mut e.args))
    }
    pub(crate) fn arg_akt(&mut self, x: ArgAkt) {
        match x {
            ArgAkt::Wert(o, id, v) => { self.satz.args_offen.insert(format!("args:{o}"), true); if let Some(g) = self.arg_katalog(&o).and_then(|k| k.params.iter().find(|p| p.id == id).map(|p| p.group.clone())) { self.satz.args_offen.insert(format!("gruppe:{o}:{g}"), true); } if let Some((opts, _)) = self.arg_ziel(&o) { if v.is_empty() { opts.remove(&id); } else { opts.insert(id.clone(), v); } } self.satz.arg_fokus = Some((o, id)) }
            ArgAkt::Frei(o, v) => { self.satz.args_offen.insert(format!("args:{o}"), true); if let Some((_, f)) = self.arg_ziel(&o) { *f = v } self.satz.arg_fokus = Some((o, String::new())) }
            ArgAkt::Platzhalter(o, name) => {
                let key = match &self.satz.arg_fokus { Some((fo, f)) if *fo == o => f.clone(), _ => String::new() }; let ph = format!("{{{name}}}");
                if let Some((opts, frei)) = self.arg_ziel(&o) { if key.is_empty() { if !frei.is_empty() && !frei.ends_with(' ') { frei.push(' ') } frei.push_str(&ph) } else { opts.entry(key).or_default().push_str(&ph) } }
            }
            ArgAkt::Migrieren(o) => { let kat = self.arg_katalog(&o); if let (Some(c), Some((opts, frei))) = (kat, self.arg_ziel(&o)) { let (s, rest) = args::migrate_free(&c, opts, frei); *opts = s; *frei = rest } }
            ArgAkt::Zurueck(o) => { if let Some((opts, frei)) = self.arg_ziel(&o) { args::reset(opts, frei) } }
            // Auf-/Zuklappen steht ausdrücklich im Modell (nie aus dem Inhalt abgeleitet): Tippen klappt nichts ein
            ArgAkt::Auf(k, aktuell) => { self.satz.args_offen.insert(k, !aktuell); }
        }
    }
    fn arg_katalog(&self, owner: &str) -> Option<Arc<Catalog>> {
        if owner == "asm" { return args::catalog_for("vcc-asm") }
        self.satz.entwurf.targets.iter().flat_map(|t| t.emulators.iter()).find(|e| e.id == owner).and_then(|e| args::catalog_of_kind(&e.kind))
    }

    fn param_zeile<'a>(&self, pal: Pal, owner: &str, opts: &Selection, p: &Param) -> Element<'a, Msg> {
        let cur = opts.get(&p.id).cloned().unwrap_or_default(); let (o, id) = (owner.to_string(), p.id.clone());
        let setzen = { let (o, id) = (o.clone(), id.clone()); move |v: String| a(ArgAkt::Wert(o.clone(), id.clone(), v)) };
        let weg: Element<Msg> = if cur.is_empty() { iced::widget::space::horizontal().width(18).into() } else { tooltip(button(text("×").size(13).color(pal.mute)).padding([0, 6]).on_press(setzen(String::new())).style(|_, _| button::Style::default()), container(text(vc_i18n::t!("argsui-zurueck-auf-den-standard-nicht-setzen")).size(11.5).color(pal.text)).padding([3, 7]).style(move |_| crate::flaeche(pal.panel2, pal.line)), tooltip::Position::Bottom).into() };
        let wahlzeile = |opt: Vec<(String, String)>, aktuell: &str, setzen: &dyn Fn(String) -> Msg| -> Element<'a, Msg> {
            row(opt.into_iter().map(|(w, l)| { let on = w == aktuell; button(text(l).size(12).color(pal.text)).padding([3, 8]).on_press(setzen(w)).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }).into() }).collect::<Vec<_>>()).spacing(5).wrap().vertical_spacing(5).into()
        };
        let steuer: Element<Msg> = match p.ty {
            ParamType::Switch if p.off_flag.is_none() => { let s2 = setzen.clone(); checkbox(cur == "on").label(p.label.clone()).on_toggle(move |b| s2(if b { "on".into() } else { String::new() })).size(15).text_size(12.5).into() }
            ParamType::Switch => row![wahlzeile(vec![(String::new(), vc_i18n::t!("argsui-standard").into()), ("on".into(), vc_i18n::t!("setui-an-klein").into()), ("off".into(), vc_i18n::t!("setui-aus-klein").into())], &cur, &setzen), text(p.label.clone()).size(12.5).color(pal.text)].spacing(8).align_y(Alignment::Center).wrap().into(),
            ParamType::Choice => { let mut opt = vec![(String::new(), vc_i18n::t!("argsui-standard").to_string())]; for c in &p.choices { opt.push((c.value.clone(), format!("{} ({})", c.label, c.value))) } row![wahlzeile(opt, &cur, &setzen), text(p.label.clone()).size(12.5).color(pal.text)].spacing(8).align_y(Alignment::Center).wrap().into() }
            ParamType::File => { let s2 = setzen.clone(); row![crate::pfadfeld::pfadfeld(pal, &p.example.clone().unwrap_or_default(), &cur, crate::pfadfeld::PfadArt::Datei, Length::Fixed(260.0), move |v| s2(v)), text(p.label.clone()).size(12.5).color(pal.text)].spacing(8).align_y(Alignment::Center).wrap().into() }
            ParamType::Text | ParamType::Number => { let br = if p.ty == ParamType::Number { Length::Fixed(90.0) } else { Length::Fixed(260.0) }; let s2 = setzen.clone(); row![text_input(&p.example.clone().unwrap_or_default(), &cur).on_input(move |v| s2(v)).size(12.5).padding(4).width(br).font(vc_gui_ui::fonts::MONO), text(p.label.clone()).size(12.5).color(pal.text)].spacing(8).align_y(Alignment::Center).wrap().into() }
        };
        let zeile = row![weg, steuer, text(markiert(p)).size(11).font(vc_gui_ui::fonts::MONO).color(pal.mute)].spacing(8).align_y(Alignment::Center).wrap().vertical_spacing(3);
        tooltip(zeile, container(text(tipp_text(p)).size(11.5).color(pal.text)).padding([4, 8]).max_width(420).style(move |_| crate::flaeche(pal.panel2, pal.line)), tooltip::Position::Bottom).into()
    }

    /// Parametereditor: `titel` Kopf, `kat` Katalog der Sorte, `ctx` Beispielwerte der Platzhalter, `programm` Kopfzeile der Vorschau
    pub(crate) fn args_editor<'a>(&'a self, pal: Pal, owner: &str, titel: &str, kat: Option<Arc<Catalog>>, ctx: args::ArgCtx, programm: &str, hint: &str, opts: &'a Selection, frei: &'a str, reserviert: bool) -> Element<'a, Msg> {
        let n_set = kat.as_deref().map_or(0, |c| args::changed_count(c, opts));
        let key = format!("args:{owner}"); let offen = self.satz.args_offen.get(&key).copied().unwrap_or(n_set > 0 || !frei.trim().is_empty());
        let kopf = format!("{} {}{}", if offen { "▾" } else { "▸" }, titel, if n_set > 0 { format!("   {}", vc_i18n::t!("argsui-n-set-gesetzt", n_set = &n_set)) } else { String::new() });
        let knopf_kopf = button(text(kopf).size(13).font(vc_gui_ui::fonts::UI_FETT).color(pal.text)).padding([4, 6]).on_press(a(ArgAkt::Auf(key.clone(), offen))).style(|_, _| button::Style::default());
        if !offen { return knopf_kopf.into() }
        let mut c = column![knopf_kopf].spacing(6);
        match &kat {
            Some(k) => {
                c = c.push(klein(pal, vc_i18n::t!("argsui-katalog-p1-nicht-gesetzte-parameter-bleiben", p1 = &k.label, p2 = if k.verified.starts_with("nicht geprüft") { vc_i18n::t!("argsui-achtung-dieser-katalog-ist-nicht-gegen") } else { String::new() })));
                for g in &k.groups {
                    if k.in_group(&g.id).count() == 0 { continue }
                    let n = k.in_group(&g.id).filter(|p| opts.get(&p.id).is_some_and(|v| !v.trim().is_empty())).count();
                    let gk = format!("gruppe:{owner}:{}", g.id); let go = self.satz.args_offen.get(&gk).copied().unwrap_or(n > 0);
                    c = c.push(button(text(format!("{} {}{}", if go { "▾" } else { "▸" }, g.label, if n > 0 { format!("   {}", vc_i18n::t!("argsui-n-gesetzt", n = &n)) } else { String::new() })).size(12.5).font(vc_gui_ui::fonts::UI_FETT).color(pal.text)).padding([3, 6]).on_press(a(ArgAkt::Auf(gk, go))).style(|_, _| button::Style::default()));
                    if go { for p in k.in_group(&g.id) { c = c.push(self.param_zeile(pal, owner, opts, p)) } }
                }
            }
            None => c = c.push(klein(pal, vc_i18n::t!("argsui-fuer-diese-sorte-gibt-es-keinen"))),
        }
        let o2 = owner.to_string();
        c = c.push(row![text(vc_i18n::t!("argsui-weitere-parameter")).size(12.5).color(pal.mute).width(140), text_input(hint, frei).on_input(move |v| a(ArgAkt::Frei(o2.clone(), v))).size(12.5).padding(5).font(vc_gui_ui::fonts::MONO).width(Length::Fill)].spacing(10).align_y(Alignment::Center));
        let mut ph = row![klein(pal, vc_i18n::t!("argsui-variable-einfuegen"))].spacing(5).align_y(Alignment::Center);
        for p in PLACEHOLDERS { let bsp = args::expand(&format!("{{{}}}", p.name), &ctx).unwrap_or_else(|_| vc_i18n::t!("argsui-nicht-in-jedem-projekt-vorhanden").into()); ph = ph.push(tooltip(button(text(format!("{{{}}}", p.name)).size(11).font(vc_gui_ui::fonts::MONO).color(pal.accent)).padding([1, 7]).on_press(a(ArgAkt::Platzhalter(owner.to_string(), p.name.to_string()))).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.accent, width: 1.0, radius: 10.0.into() }, text_color: pal.accent, ..Default::default() }), container(text(vc_i18n::t!("argsui-p1-p2-beispiel-bsp-fuegt-den", p1 = &p.label, p2 = &p.hint, bsp = &bsp)).size(11.5).color(pal.text)).padding([4, 8]).max_width(380).style(move |_| crate::flaeche(pal.panel2, pal.line)), tooltip::Position::Bottom)) }
        c = c.push(ph.wrap().vertical_spacing(4));
        if let Some(k) = kat.as_deref() { let n = args::migratable(k, opts, frei); if n > 0 { c = c.push(row![klein(pal, vc_i18n::t!("argsui-n-parameter-im-freien-feld-kennt", n = &n)), knopf(pal, vc_i18n::t!("argsui-in-die-auswahl-uebernehmen"), Some(a(ArgAkt::Migrieren(owner.to_string()))), pal.text, false)].spacing(8).align_y(Alignment::Center).wrap()) } }
        for i in args::check(kat.as_deref(), opts, frei, Some(&ctx)) { let (sym, col) = match i.level { Level::Error => ("✖", pal.err), Level::Warn => ("⚠", pal.warn), Level::Info => ("ℹ", pal.mute) }; c = c.push(text(vc_i18n::t!("argsui-sym-p2", sym = &sym, p2 = &i.text)).size(11.5).color(col)) }
        let (aufgeloest, _) = args::preview(kat.as_deref(), opts, frei, &ctx); let zeile = args::shell_line(programm, &aufgeloest);
        let mut vor = column![klein(pal, vc_i18n::t!("argsui-vorschau-beispielprojekt-demo")), text(zeile).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.text)].spacing(3);
        if let Some(k) = kat.as_deref().filter(|_| reserviert) { if !k.reserved.is_empty() { vor = vor.push(klein(pal, vc_i18n::t!("argsui-beim-starten-setzt-vintagecoder-selbst-p1", p1 = k.reserved.iter().take(6).cloned().collect::<Vec<_>>().join(" ")))) } }
        c = c.push(container(vor).padding(8).width(Length::Fill).style(move |_| crate::flaeche(pal.bg, pal.line)));
        let geaendert = !opts.is_empty() || !frei.trim().is_empty();
        c = c.push(knopf(pal, vc_i18n::t!("argsui-auf-standard-zuruecksetzen"), geaendert.then(|| a(ArgAkt::Zurueck(owner.to_string()))), pal.text, false));
        c.into()
    }
}
