// SPDX-License-Identifier: GPL-3.0-or-later
//! SPIKE (nur macOS): native Menüleiste über `muda` und AccessKit-Anschluss über `accesskit_macos::SubclassingAdapter` auf dem NSView
//! des Iced-Fensters (Iced selbst hat keine AccessKit-Anbindung). Dazu eine Selbstprüfung über die NSAccessibility-Schnittstelle.
use crate::{App, Msg, Tab};
use accesskit::{ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate};
use iced::{Subscription, Task};
use std::cell::RefCell;
use std::sync::Mutex;

thread_local! {
    static MENU: RefCell<Option<muda::Menu>> = const { RefCell::new(None) };
    static ADAPTER: RefCell<Option<accesskit_macos::SubclassingAdapter>> = const { RefCell::new(None) };
    static VIEW: RefCell<usize> = const { RefCell::new(0) };
}
static LAST_TREE: Mutex<Option<TreeUpdate>> = Mutex::new(None);

pub fn init() {
    use muda::{accelerator::{Accelerator, Code, Modifiers}, Menu, MenuItem, PredefinedMenuItem, Submenu};
    let menu = Menu::new();
    let cmd = Modifiers::SUPER;
    let app = Submenu::new("VintageCoder", true);
    let _ = app.append_items(&[&PredefinedMenuItem::about(Some("Über VintageCoder (Spike)"), None), &PredefinedMenuItem::separator(), &PredefinedMenuItem::hide(Some("VintageCoder ausblenden")), &PredefinedMenuItem::separator(), &PredefinedMenuItem::quit(Some("VintageCoder beenden"))]);
    let item = |id: &str, label: &str, acc: Option<Accelerator>| MenuItem::with_id(id, label, true, acc);
    let file = Submenu::new("Datei", true);
    let _ = file.append_items(&[&item("new", "Neuer Reiter (unbenannt)", Some(Accelerator::new(Some(cmd), Code::KeyN))), &item("open", "Datei öffnen …", Some(Accelerator::new(Some(cmd), Code::KeyO))), &item("save", "Speichern", Some(Accelerator::new(Some(cmd), Code::KeyS)))]);
    let view = Submenu::new("Ansicht", true);
    let _ = view.append_items(&[&item("palette", "Befehl oder Datei suchen …", Some(Accelerator::new(Some(cmd), Code::KeyK))), &item("theme", "Hell/Dunkel umschalten", None), &item("wrap", "Zeilenumbruch umschalten", None)]);
    let project = Submenu::new("Projekt", true);
    let _ = project.append_items(&[&item("build", "Projekt bauen", Some(Accelerator::new(None, Code::F7)))]);
    let _ = menu.append_items(&[&app, &file, &view, &project]);
    menu.init_for_nsapp();
    MENU.with(|m| *m.borrow_mut() = Some(menu));
}

pub fn menu_subscription() -> Subscription<Msg> {
    Subscription::run(|| iced::stream::channel(32, async |mut out: iced::futures::channel::mpsc::Sender<Msg>| {
        use iced::futures::SinkExt;
        let (tx, mut rx) = iced::futures::channel::mpsc::unbounded::<String>();
        std::thread::spawn(move || { while let Ok(ev) = muda::MenuEvent::receiver().recv() { if tx.unbounded_send(ev.id.0.clone()).is_err() { break } } });
        use iced::futures::StreamExt;
        while let Some(id) = rx.next().await { let _ = out.send(Msg::Menu(id)).await; }
    }))
}

// ---------------------------------------------------------------- AccessKit
struct Activation;
impl ActivationHandler for Activation { fn request_initial_tree(&mut self) -> Option<TreeUpdate> { LAST_TREE.lock().unwrap().clone() } }
struct Actions;
impl ActionHandler for Actions { fn do_action(&mut self, r: ActionRequest) { eprintln!("[a11y] Aktion angefragt: {:?} auf {:?}", r.action, r.target_node); } }

/// Legt den Adapter am NSView des (ältesten) Fensters an; läuft im Hauptthread des Fensters.
pub fn a11y_task() -> Task<Msg> {
    iced::window::oldest().and_then(|id| iced::window::run(id, |w| {
        use raw_window_handle::{HasWindowHandle as _, RawWindowHandle};
        if let Ok(h) = w.window_handle() { if let RawWindowHandle::AppKit(a) = h.as_raw() {
            let view = a.ns_view.as_ptr();
            let ad = unsafe { accesskit_macos::SubclassingAdapter::new(view, Activation, Actions) };
            ADAPTER.with(|x| *x.borrow_mut() = Some(ad)); VIEW.with(|v| *v.borrow_mut() = view as usize);
        } }
    })).map(|_| Msg::A11yReady)
}

fn id(n: u64) -> NodeId { NodeId(n) }
const ROOT: u64 = 1; const TOOLBAR: u64 = 2; const FILES: u64 = 3; const TABS: u64 = 4; const EDITOR: u64 = 5; const DRAWER: u64 = 6; const STATUS: u64 = 7;

fn build(app: &App) -> TreeUpdate {
    let mut nodes: Vec<(NodeId, Node)> = vec![];
    let mut leaf = |nodes: &mut Vec<(NodeId, Node)>, n: u64, role: Role, label: &str| -> NodeId { let mut x = Node::new(role); x.set_label(label); nodes.push((id(n), x)); id(n) };
    // Werkzeugleiste
    let b1 = leaf(&mut nodes, 10, Role::Button, "Starten"); let b2 = leaf(&mut nodes, 11, Role::Button, "Debuggen"); let b3 = leaf(&mut nodes, 12, Role::Button, if app.building { "Bauen, läuft" } else { "Bauen" }); let b4 = leaf(&mut nodes, 13, Role::Button, "Befehl oder Datei suchen");
    let mut tb = Node::new(Role::Toolbar); tb.set_label("Hauptaktionen"); tb.set_children(vec![b1, b2, b3, b4]); nodes.push((id(TOOLBAR), tb));
    // Dateien
    let mut rows = vec![];
    for (i, f) in app.files.iter().enumerate() { let mut r = Node::new(Role::ListItem); r.set_label(f.as_str()); let open = app.cur_name().as_deref() == Some(f.as_str()); r.set_selected(open); nodes.push((id(100 + i as u64), r)); rows.push(id(100 + i as u64)); }
    let mut fl = Node::new(Role::List); fl.set_label("Dateien"); fl.set_children(rows); nodes.push((id(FILES), fl));
    // Reiter
    let mut tabs = vec![];
    for (i, t) in app.tabs.iter().enumerate() { let (name, dirty) = match t { Tab::File(n) => (n.clone(), app.editors.get(n).map_or(false, |e| e.dirty)), Tab::Studio => ("ball.sprite".to_string(), false) }; let mut r = Node::new(Role::Tab); r.set_label(if dirty { format!("{name}, ungespeichert") } else { name }); r.set_selected(i == app.active); nodes.push((id(1000 + i as u64), r)); tabs.push(id(1000 + i as u64)); }
    let mut tl = Node::new(Role::TabList); tl.set_label("Geöffnete Dokumente"); tl.set_children(tabs); nodes.push((id(TABS), tl));
    // Editor: aktuelle Zeile als Wert
    let mut ed = Node::new(match app.tabs.get(app.active) { Some(Tab::Studio) => Role::Canvas, _ => Role::MultilineTextInput });
    if let Some(Tab::File(n)) = app.tabs.get(app.active) { ed.set_label(format!("Editor {n}")); if let Some(e) = app.editors.get(n) { ed.set_value(format!("Zeile {}: {}", e.cursor.line + 1, e.lines[e.cursor.line])) } } else { ed.set_label("Sprite-Studio, Zeichenfläche 24 mal 21") }
    nodes.push((id(EDITOR), ed));
    let mut dr = Node::new(Role::Log); dr.set_label("Protokoll"); if let Some(l) = app.log.last() { dr.set_value(l.as_str()) } nodes.push((id(DRAWER), dr));
    let mut st = Node::new(Role::Status); st.set_label(if app.building { "baut" } else { "Bereit" }); nodes.push((id(STATUS), st));
    let mut root = Node::new(Role::Window); root.set_label("VintageCoder Spike"); root.set_children(vec![id(TOOLBAR), id(FILES), id(TABS), id(EDITOR), id(DRAWER), id(STATUS)]); nodes.push((id(ROOT), root));
    TreeUpdate { nodes, tree: Some(TreeInfo::new(id(ROOT))), tree_id: TreeId::ROOT, focus: id(EDITOR) }
}

pub fn a11y_update(app: &App) {
    let t = build(app); *LAST_TREE.lock().unwrap() = Some(t.clone());
    ADAPTER.with(|a| { if let Some(a) = a.borrow_mut().as_mut() { if let Some(ev) = a.update_if_active(|| t) { ev.raise(); } } });
}

/// Selbstprüfung: fragt den NSView über NSAccessibility nach seinen Kindern (löst die Aktivierung des Adapters aus) und liefert Rolle und Name.
pub fn self_check() -> String {
    use objc2::{msg_send, runtime::AnyObject};
    use objc2_foundation::NSString;
    let v = VIEW.with(|v| *v.borrow()) as *mut AnyObject; if v.is_null() { return "kein NSView".into() }
    let mut out = String::new();
    unsafe {
        fn s(o: *mut AnyObject) -> String { if o.is_null() { "-".into() } else { unsafe { let d: *mut AnyObject = msg_send![o, description]; (&*(d as *mut NSString)).to_string() } } }
        fn walk(o: *mut AnyObject, depth: usize, out: &mut String) {
            if depth > 3 { return }
            unsafe {
                let role: *mut AnyObject = msg_send![o, accessibilityRole]; let t1: *mut AnyObject = msg_send![o, accessibilityTitle]; let t2: *mut AnyObject = msg_send![o, accessibilityLabel]; let t3: *mut AnyObject = msg_send![o, accessibilityHelp]; let label = if !t1.is_null() { t1 } else if !t2.is_null() { t2 } else { t3 }; let val: *mut AnyObject = msg_send![o, accessibilityValue];
                out.push_str(&format!("{}{} „{}“ {}\n", "  ".repeat(depth), s(role), s(label), if val.is_null() { String::new() } else { format!("= {}", s(val)) }));
                let ch: *mut AnyObject = msg_send![o, accessibilityChildren];
                if !ch.is_null() { let n: usize = msg_send![ch, count]; for i in 0..n { let c: *mut AnyObject = msg_send![ch, objectAtIndex: i]; walk(c, depth + 1, out) } }
            }
        }
        walk(v, 0, &mut out);
    }
    out
}
