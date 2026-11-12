// SPDX-License-Identifier: GPL-3.0-or-later
//! Abschnitt „Physische Zielhardware & Netzwerkgeräte“: Netzwerk (Ultimate) und Seriell (MEGA65) mit Verbindungstest und Port-Scan.
use super::{services, state::{hw_key, Probe, UiState}, widgets};
use eframe::egui::{self, RichText};
use vc_ide::platform::BAUD_RATES;
use vc_ide::settings::Settings;
use vc_ide::targets;

pub fn show(ui: &mut egui::Ui, s: &mut Settings, st: &mut UiState) {
    let Some(info) = targets::system(&st.system) else { return };
    let mut add: Option<&'static str> = None;
    // nur Geräte, deren Anbindung im Release gewählt ist (die übrigen Einträge bleiben in der Datei)
    let hw_kinds: Vec<&'static str> = info.hardware().iter().copied().filter(|k| vc_ide::emulators::device_enabled(k)).collect();
    widgets::section(ui, "Physische Zielhardware & Netzwerkgeräte", "Kompilierte Programme direkt über Ethernet oder die serielle Schnittstelle auf echte Maschinen übertragen.", |ui| {
        ui.add_enabled_ui(!hw_kinds.is_empty(), |ui| {
            let r = ui.menu_button("+  Hardware hinzufügen", |ui| { for k in &hw_kinds { if ui.button(targets::hardware_label(k)).clicked() { add = Some(*k); ui.close(); } } });
            if hw_kinds.is_empty() { r.response.on_disabled_hover_text(if info.hardware().is_empty() { "Für dieses System gibt es keine unterstützte Hardware-Anbindung" } else { "Diese Hardware-Anbindung ist in diesem Release nicht enthalten" }); }
        });
    });
    if let Some(k) = add { s.add_machine(info.id, k); }
    let ids: Vec<String> = s.machines_for(info.id).filter(|m| vc_ide::emulators::device_enabled(&m.kind)).map(|m| m.id.clone()).collect();
    if ids.is_empty() { ui.label(RichText::new(if hw_kinds.is_empty() { if info.hardware().is_empty() { "Für dieses System ist keine echte Hardware anbindbar." } else { "In diesem Release ist keine Hardware-Anbindung für dieses System enthalten." } } else { "Noch keine Hardware eingetragen." }).weak()); }
    let ctx = ui.ctx().clone();
    let (mut remove, mut test, mut scan) = (None::<String>, None::<String>, false);
    let mut test_link: Option<(String, vc_ide::mega65net::Link)> = None;
    for id in &ids {
        let Some(m) = s.machines.iter_mut().find(|m| &m.id == id) else { continue };
        let key = hw_key(&m.id);
        widgets::card(ui, false, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut m.name).hint_text("Name").desired_width(210.0));
                ui.label(RichText::new(targets::hardware_label(&m.kind)).weak());
                match st.probe(&key) {
                    Some(Probe::Ok(_)) => widgets::chip(ui, "Online", crate::theme::ok()),
                    Some(Probe::Err(_)) => widgets::chip(ui, "Nicht erreichbar", crate::theme::bad()),
                    Some(Probe::Running) => widgets::chip(ui, "prüfe …", crate::theme::warn()),
                    None => widgets::chip(ui, "ungeprüft", crate::theme::dim()),
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui.button("Entfernen").clicked() { remove = Some(m.id.clone()); } });
            });
            ui.add_space(4.0);
            egui::Grid::new(("hw_grid", &m.id)).num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                if m.kind == "ultimate64" {
                    widgets::label(ui, "Adresse");
                    ui.add(egui::TextEdit::singleline(&mut m.host).desired_width(280.0).font(egui::TextStyle::Monospace).hint_text("192.168.1.50 oder host:port"));
                    ui.end_row();
                    widgets::label(ui, "Passwort");
                    ui.horizontal(|ui| {
                        let shown = st.show_pw.contains(&m.id);
                        ui.add(egui::TextEdit::singleline(&mut m.password).password(!shown).desired_width(280.0).hint_text("leer = kein Netzwerkpasswort"));
                        if ui.button(if shown { "Verbergen" } else { "Zeigen" }).clicked() { if shown { st.show_pw.remove(&m.id); } else { st.show_pw.insert(m.id.clone()); } }
                    });
                    ui.end_row();
                } else {
                    widgets::label(ui, "Verbindung");
                    ui.vertical(|ui| {
                        let mut cur = if m.link == "eth" { "eth".to_string() } else { "usb".to_string() };
                        ui.horizontal(|ui| {
                            crate::ui::widgets::segmented(ui, &mut cur, &[("usb", "USB (Standard)"), ("eth", "Ethernet")]);
                            ui.label(RichText::new("Standard für alle Aktionen").weak().small());
                        });
                        m.link = cur;
                        egui::Grid::new(("hw_links", &m.id)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                            for (name, field, hint) in [("Starten", &mut m.link_start, "Programm laden und starten (USB: eigener Monitor; Ethernet: etherload)"),
                                                         ("Kopieren", &mut m.link_copy, "Disketten auf die SD-Karte kopieren (nur Ethernet; USB-Übertragung ist zurückgestellt)"),
                                                         ("Debuggen", &mut m.link_debug, "Haltepunkte und Register (nur USB)")] {
                                ui.label(RichText::new(name).weak().small()).on_hover_text(hint);
                                let mut c = field.clone();
                                crate::ui::widgets::segmented(ui, &mut c, &[("", "wie Standard"), ("usb", "USB"), ("eth", "Ethernet")]);
                                *field = c;
                                ui.end_row();
                            }
                        });
                    });
                    ui.end_row();
                    widgets::label(ui, "Schnittstelle");
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut m.device).desired_width(280.0).font(egui::TextStyle::Monospace).hint_text(if cfg!(windows) { "COM3" } else if cfg!(target_os = "linux") { "/dev/ttyUSB0" } else { "/dev/cu.usbserial-…" }));
                        egui::ComboBox::from_id_salt(("hw_port", &m.id)).selected_text("gefundene Ports").show_ui(ui, |ui| {
                            if st.ports.is_empty() { ui.label(RichText::new("keine USB-Seriell-Geräte gefunden").weak()); }
                            for p in &st.ports { if ui.selectable_label(m.device == *p, p).clicked() { m.device = p.clone(); } }
                        });
                        if ui.add_enabled(!st.scanning, egui::Button::new(if st.scanning { "sucht …" } else { "Ports scannen" })).clicked() { scan = true; }
                    });
                    ui.end_row();
                    widgets::label(ui, "Baudrate");
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut m.baud).desired_width(100.0).font(egui::TextStyle::Monospace).hint_text("2000000"));
                        egui::ComboBox::from_id_salt(("hw_baud", &m.id)).selected_text("üblich").show_ui(ui, |ui| { for b in BAUD_RATES { if ui.selectable_label(m.baud == *b, *b).clicked() { m.baud = b.to_string(); } } });
                    });
                    ui.end_row();
                    widgets::label(ui, "Ethernet");
                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!("{} – nur gegen Attrappen getestet. Voraussetzung am Gerät: Core ab 0.96, DIP-Schalter 2 ON, gültige MAC-Adresse, Fernzugriff mit Shift+£ (gilt bis zum nächsten Reset).", vc_ide::mega65net::UNCHECKED)).color(crate::theme::warn()).small());
                        egui::Grid::new(("hw_eth", &m.id)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                            ui.label(RichText::new("Netzwerkschnittstelle").weak().small());
                            ui.add(egui::TextEdit::singleline(&mut m.eth_iface).desired_width(200.0).font(egui::TextStyle::Monospace).hint_text(if cfg!(windows) { "Index, z. B. 5" } else if cfg!(target_os = "macos") { "en0" } else { "eth0" }));
                            ui.end_row();
                            ui.label(RichText::new("etherload").weak().small());
                            ui.add(egui::TextEdit::singleline(&mut m.etherload).desired_width(280.0).font(egui::TextStyle::Monospace).hint_text("leer = im PATH suchen"));
                            ui.end_row();
                            ui.label(RichText::new("mega65_ftp").weak().small());
                            ui.add(egui::TextEdit::singleline(&mut m.ftp_tool).desired_width(280.0).font(egui::TextStyle::Monospace).hint_text("leer = im PATH suchen"));
                            ui.end_row();
                            ui.label(RichText::new("SD-Ordner").weak().small());
                            ui.add(egui::TextEdit::singleline(&mut m.sd_dir).desired_width(200.0).font(egui::TextStyle::Monospace).hint_text("leer = oberste Ebene, z. B. VintageCoder"));
                            ui.end_row();
                        });
                    });
                    ui.end_row();
                }
                widgets::label(ui, "Start");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut m.run_after, "Nach Übertragung RUN ausführen");
                    if m.kind == "ultimate64" { ui.checkbox(&mut m.reset_before, "Maschine vorher zurücksetzen"); }
                });
                ui.end_row();
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let busy = st.running(&key);
                let eth = m.kind == "mega65" && m.link_default() == vc_ide::mega65net::Link::Eth;
                let ready = match m.kind.as_str() { "ultimate64" => !m.host.trim().is_empty(), _ => eth || !m.device.trim().is_empty() };
                if ui.add_enabled(!busy && ready, egui::Button::new(if busy { "prüfe …" } else if m.kind == "mega65" { if eth { "Ethernet testen" } else { "USB testen" } } else { "Verbindung testen" })).on_disabled_hover_text("Adresse bzw. Schnittstelle fehlt").clicked() { test = Some(m.id.clone()); }
                if m.kind == "mega65" {
                    let other = if eth { vc_ide::mega65net::Link::Usb } else { vc_ide::mega65net::Link::Eth };
                    let other_ready = if eth { !m.device.trim().is_empty() } else { true };
                    if ui.add_enabled(!busy && other_ready, egui::Button::new(format!("{} testen", other.label()))).clicked() { test_link = Some((m.id.clone(), other)); }
                }
                match st.probe(&key) {
                    Some(Probe::Ok(t)) => { ui.label(RichText::new(format!("OK: {t}")).color(crate::theme::ok()).small()); }
                    Some(Probe::Err(t)) => { ui.label(RichText::new(t).color(crate::theme::bad()).small()); }
                    _ => {}
                }
            });
        });
        ui.add_space(8.0);
    }
    if scan { services::scan_ports(st, &ctx); }
    if let Some(id) = test { if let Some(m) = s.machine(&id).cloned() { services::test_hardware(st, &ctx, &m); } }
    if let Some((id, l)) = test_link { if let Some(m) = s.machine(&id).cloned() { services::test_hardware_link(st, &ctx, &m, l); } }
    if let Some(id) = remove { s.remove_machine(&id); st.probes.remove(&hw_key(&id)); }
}
