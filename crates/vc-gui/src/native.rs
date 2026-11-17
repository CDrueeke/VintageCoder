// SPDX-License-Identifier: GPL-3.0-or-later
//! macOS: Menüleiste aus der Befehlstabelle (`muda`) und Barrierefreiheit (AccessKit über `accesskit_macos` am NSView; im Spike erprobt).
use crate::Msg;
use accesskit::{ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate};
use iced::{Subscription, Task};
use std::cell::RefCell;
use std::sync::Mutex;
use vc_gui_ui::befehle::{zerlege, BEFEHLE, MENUES};

thread_local! {
    static MENU: RefCell<Option<muda::Menu>> = const { RefCell::new(None) };
    static ITEMS: RefCell<Vec<(vc_gui_ui::befehle::Cmd, muda::MenuItem)>> = const { RefCell::new(Vec::new()) };
    static ADAPTER: RefCell<Option<accesskit_macos::SubclassingAdapter>> = const { RefCell::new(None) };
    static VIEW: RefCell<usize> = const { RefCell::new(0) };
}
static LAST_TREE: Mutex<Option<TreeUpdate>> = Mutex::new(None);

fn code(taste: &str) -> Option<muda::accelerator::Code> {
    use muda::accelerator::Code::*;
    Some(match taste {
        "Plus" => Equal, "Minus" => Minus, "Komma" => Comma, "Rechts" => ArrowRight, "Links" => ArrowLeft,
        "F1" => F1, "F5" => F5, "F6" => F6, "F7" => F7, "F8" => F8,
        "0" => Digit0,
        t if t.len() == 1 && t.as_bytes()[0].is_ascii_uppercase() => match t.as_bytes()[0] { b'A' => KeyA, b'B' => KeyB, b'F' => KeyF, b'J' => KeyJ, b'K' => KeyK, b'L' => KeyL, b'N' => KeyN, b'O' => KeyO, b'S' => KeyS, b'T' => KeyT, b'W' => KeyW, b'Z' => KeyZ, _ => return None },
        _ => return None })
}

/// Menüleiste aus `BEFEHLE`: je Menü ein Untermenü, Einträge mit Kürzel
pub fn init() {
    use muda::{accelerator::{Accelerator, Modifiers}, Menu, MenuItem, PredefinedMenuItem, Submenu};
    let menu = Menu::new(); let app = Submenu::new("VintageCoder", true);
    let _ = app.append_items(&[&PredefinedMenuItem::about(Some("Über VintageCoder"), None), &PredefinedMenuItem::separator(), &PredefinedMenuItem::hide(Some("VintageCoder ausblenden"))]);
    let _ = menu.append(&app);
    let mut items = vec![];
    for m in MENUES {
        let sub = Submenu::new(*m, true);
        // oberste Einträge, dann die Untermenüs (wie die Menüleiste im Fenster)
        let mut anhaengen = |ziel: &Submenu, unter: &str| {
            for i in vc_gui_ui::befehle::menue_eintraege(m, unter) {
                let acc = zerlege(i.tasten).and_then(|(cmd, sh, alt, k)| { let mut mods = Modifiers::empty(); if cmd { mods |= Modifiers::SUPER } if sh { mods |= Modifiers::SHIFT } if alt { mods |= Modifiers::ALT } code(k).map(|c| Accelerator::new(Some(mods), c)) });
                let it = MenuItem::with_id(i.id, vc_gui_ui::befehle::menuename(i), true, acc); let _ = ziel.append(&it); items.push((i.cmd, it));
            }
        };
        anhaengen(&sub, ""); if vc_gui_ui::befehle::menue_eintraege(m, "").next().is_some() && !vc_gui_ui::befehle::untermenues(m).is_empty() { let _ = sub.append(&PredefinedMenuItem::separator()); }
        for u in vc_gui_ui::befehle::untermenues(m) { let us = Submenu::new(*u, true); anhaengen(&us, u); let _ = sub.append(&us); }
        let _ = menu.append(&sub);
    }
    menu.init_for_nsapp(); ITEMS.with(|x| *x.borrow_mut() = items); MENU.with(|m| *m.borrow_mut() = Some(menu));
}

/// Einträge ein-/ausgrauen (Konzept 7: nicht nutzbar = ausgegraut)
pub fn aktiviere(f: impl Fn(vc_gui_ui::befehle::Cmd) -> bool) { ITEMS.with(|x| for (c, it) in x.borrow().iter() { it.set_enabled(f(*c)) }) }

pub fn menu_subscription() -> Subscription<Msg> {
    Subscription::run(|| iced::stream::channel(32, async |mut out: iced::futures::channel::mpsc::Sender<Msg>| {
        use iced::futures::{SinkExt, StreamExt};
        let (tx, mut rx) = iced::futures::channel::mpsc::unbounded::<String>();
        std::thread::spawn(move || { while let Ok(ev) = muda::MenuEvent::receiver().recv() { if tx.unbounded_send(ev.id.0.clone()).is_err() { break } } });
        while let Some(id) = rx.next().await { let _ = out.send(Msg::Menue(id)).await; }
    }))
}

// ------------------------------------------------------------ AccessKit
#[derive(Default, Clone)]
pub struct A11y { pub reiter: Vec<(String, bool, bool)>, pub dateien: Vec<(String, bool)>, pub editor_name: String, pub editor_wert: String, pub protokoll: String, pub baut: bool, pub streifen: Vec<(String, bool)>, pub bereiche: Vec<(String, String, Vec<String>)>, pub kontext: Option<(String, Vec<(String, bool)>)> , pub studio: Option<(String, String)> }

struct Activation; impl ActivationHandler for Activation { fn request_initial_tree(&mut self) -> Option<TreeUpdate> { LAST_TREE.lock().unwrap().clone() } }
struct Actions; impl ActionHandler for Actions { fn do_action(&mut self, r: ActionRequest) { eprintln!("[a11y] Aktion angefragt: {:?} auf {:?}", r.action, r.target_node) } }

pub fn a11y_task() -> Task<Msg> {
    iced::window::oldest().and_then(|id| iced::window::run(id, |w| {
        use raw_window_handle::{HasWindowHandle as _, RawWindowHandle};
        if let Ok(h) = w.window_handle() { if let RawWindowHandle::AppKit(a) = h.as_raw() {
            let view = a.ns_view.as_ptr(); let ad = unsafe { accesskit_macos::SubclassingAdapter::new(view, Activation, Actions) };
            ADAPTER.with(|x| *x.borrow_mut() = Some(ad)); VIEW.with(|v| *v.borrow_mut() = view as usize);
        } }
    })).map(|_| Msg::A11yBereit)
}

fn id(n: u64) -> NodeId { NodeId(n) }
fn baue(a: &A11y) -> TreeUpdate {
    let mut nodes: Vec<(NodeId, Node)> = vec![]; let leaf = |nodes: &mut Vec<(NodeId, Node)>, n: u64, role: Role, label: &str| -> NodeId { let mut x = Node::new(role); x.set_label(label); nodes.push((id(n), x)); id(n) };
    let b: Vec<NodeId> = [("Starten", 10), ("Debuggen", 11), (if a.baut { "Bauen, läuft" } else { "Bauen" }, 12), ("Befehl oder Datei suchen", 13)].iter().map(|(l, n)| leaf(&mut nodes, *n, Role::Button, l)).collect();
    let mut tb = Node::new(Role::Toolbar); tb.set_label("Hauptaktionen"); tb.set_children(b); nodes.push((id(2), tb));
    let mut rows = vec![]; for (i, (f, sel)) in a.dateien.iter().enumerate() { let mut r = Node::new(Role::ListItem); r.set_label(f.as_str()); r.set_selected(*sel); nodes.push((id(100 + i as u64), r)); rows.push(id(100 + i as u64)) }
    let mut fl = Node::new(Role::List); fl.set_label("Dateien"); fl.set_children(rows); nodes.push((id(3), fl));
    let mut tabs = vec![]; for (i, (n, sel, dirty)) in a.reiter.iter().enumerate() { let mut r = Node::new(Role::Tab); r.set_label(if *dirty { format!("{n}, ungespeichert") } else { n.clone() }); r.set_selected(*sel); nodes.push((id(100000 + i as u64), r)); tabs.push(id(100000 + i as u64)) }
    let mut tl = Node::new(Role::TabList); tl.set_label("Geöffnete Dokumente"); tl.set_children(tabs); nodes.push((id(4), tl));
    let mut ed = Node::new(Role::MultilineTextInput); ed.set_label(a.editor_name.as_str()); ed.set_value(a.editor_wert.as_str()); nodes.push((id(5), ed));
    let mut dr = Node::new(Role::Log); dr.set_label("Protokoll"); dr.set_value(a.protokoll.as_str()); nodes.push((id(6), dr));
    let mut st = Node::new(Role::Status); st.set_label(if a.baut { "baut" } else { "Bereit" }); nodes.push((id(7), st));
    let sk: Vec<NodeId> = a.streifen.iter().enumerate().map(|(i, (name, auf))| { let mut x = Node::new(Role::Button); x.set_label(format!("{name}, {}", if *auf { "ausgeklappt" } else { "eingeklappt" })); x.set_expanded(*auf); nodes.push((id(200000 + i as u64), x)); id(200000 + i as u64) }).collect();
    let mut sg = Node::new(Role::Group); sg.set_label("Rechte Leiste"); sg.set_children(sk); nodes.push((id(8), sg));
    let mut bkinder: Vec<NodeId> = vec![];
    for (bi, (name, zustand, knoepfe)) in a.bereiche.iter().enumerate() {
        let ks: Vec<NodeId> = knoepfe.iter().enumerate().map(|(ki, kn)| { let nid = 300000 + (bi * 10 + ki) as u64; let mut x = Node::new(Role::Button); x.set_label(kn.as_str()); nodes.push((id(nid), x)); id(nid) }).collect();
        let gid = 310000 + bi as u64; let mut g = Node::new(Role::Group); g.set_label(format!("{name}, {zustand}")); g.set_expanded(zustand != "minimiert"); g.set_children(ks); nodes.push((id(gid), g)); bkinder.push(id(gid));
    }
    let mut bg = Node::new(Role::Group); bg.set_label("Bereiche"); bg.set_children(bkinder); nodes.push((id(9), bg));
    let mut kind_ids: Vec<NodeId> = vec![];
    if let Some((name, eintraege)) = &a.kontext {
        let items: Vec<NodeId> = eintraege.iter().enumerate().map(|(i, (n, ok))| { let mut x = Node::new(Role::MenuItem); x.set_label(n.as_str()); if !*ok { x.set_disabled() } nodes.push((id(400000 + i as u64), x)); id(400000 + i as u64) }).collect();
        let mut m = Node::new(Role::Menu); m.set_label(name.as_str()); m.set_children(items); nodes.push((id(15), m)); kind_ids.push(id(15));
    }
    if let Some((name, wert)) = &a.studio { let mut z = Node::new(Role::Canvas); z.set_label(name.as_str()); z.set_description(wert.as_str()); nodes.push((id(16), z)); kind_ids.push(id(16)); }
    let mut root = Node::new(Role::Window); root.set_label("VintageCoder"); { let mut kids = vec![id(2), id(3), id(4), id(5), id(8), id(9), id(6), id(7)]; kids.extend(kind_ids); root.set_children(kids); } nodes.push((id(1), root));
    TreeUpdate { nodes, tree: Some(TreeInfo::new(id(1))), tree_id: TreeId::ROOT, focus: id(5) }
}
pub fn a11y_update(a: &A11y) {
    let t = baue(a); *LAST_TREE.lock().unwrap() = Some(t.clone());
    ADAPTER.with(|x| { if let Some(ad) = x.borrow_mut().as_mut() { if let Some(ev) = ad.update_if_active(|| t) { ev.raise() } } });
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Jede Knoten-Kennung darf nur einmal vorkommen (AccessKit bricht sonst mit einem Panic ab, z. B. beim Öffnen des Kontextmenüs)
    #[test] fn alle_knoten_kennungen_sind_eindeutig_und_Kinder_existieren() {
        let a = A11y { reiter: vec![("a.asm".into(), true, false), ("b.asm".into(), false, true)], dateien: vec![("a.asm".into(), true), ("b.asm".into(), false)], editor_name: "Editor a.asm".into(), editor_wert: "x".into(), protokoll: "p".into(), baut: false,
            streifen: vec![("Gliederung".into(), true), ("Hilfe".into(), false), ("Zeichen".into(), false)],
            bereiche: vec![("Linke Spalte".into(), "normal".into(), vec!["a".into(), "b".into()]), ("Dokumentenfeld".into(), "normal".into(), vec!["c".into()]), ("Schublade".into(), "minimiert".into(), vec!["d".into()]), ("Rechtes Panel".into(), "normal".into(), vec!["e".into(), "f".into()])],
            kontext: Some(("Kontextmenü Editor".into(), vec![("Kopieren".into(), true), ("Einfügen".into(), false), ("Haltepunkt umschalten".into(), true)])), studio: Some(("Zeichenfläche hero.vcsprite (Sprite)".into(), "Frame 0 von 2, 12×21 Pixel".into())) };
        for mit_menue in [true, false] {
            let a = A11y { kontext: if mit_menue { a.kontext.clone() } else { None }, ..a.clone() };
            let t = baue(&a); let mut ids: Vec<u64> = t.nodes.iter().map(|(i, _)| i.0).collect(); let n = ids.len(); ids.sort(); ids.dedup();
            assert_eq!(n, ids.len(), "doppelte Knoten-Kennung (mit Menü: {mit_menue})");
            let alle: std::collections::HashSet<u64> = ids.into_iter().collect();
            for (_, nd) in &t.nodes { for k in nd.children() { assert!(alle.contains(&k.0), "Kind {} fehlt", k.0) } }
        }
    }
}
