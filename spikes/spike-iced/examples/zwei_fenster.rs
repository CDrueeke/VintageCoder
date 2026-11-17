// SPDX-License-Identifier: GPL-3.0-or-later
//! Spike (kein Produktcode): Iced 0.14 mit zwei Fenstern und EINEM gemeinsamen Modell (Entscheidung 22, Konzept 14.5/17).
//! Prüft: `daemon` + `window::open`, je Fenster eine Sicht, Zustand gemeinsam, Schließen des zweiten Fensters, Fenstergröße/-position lesen.
//! Selbsttest: `cargo run -p spike-iced --example zwei_fenster` öffnet das Hauptfenster und ein gelöstes Fenster, zählt im gemeinsamen
//! Modell hoch, schließt das gelöste Fenster nach 1,5 s, meldet das Ergebnis auf der Standardausgabe und beendet das Programm.
use iced::widget::{button, column, text};
use iced::{window, Element, Subscription, Task};
use std::time::Duration;

#[derive(Debug, Clone)]
enum Msg { Geoeffnet(window::Id), Geschlossen(window::Id), Zaehle, Tick, Lage(window::Id, Option<iced::Point>, iced::Size) }

struct App { haupt: window::Id, gelöst: Option<window::Id>, zaehler: u32, ticks: u32, log: Vec<String> }

fn boot() -> (App, Task<Msg>) {
    let (haupt, t1) = window::open(window::Settings { size: iced::Size::new(500.0, 300.0), ..Default::default() });
    (App { haupt, gelöst: None, zaehler: 0, ticks: 0, log: vec![] }, t1.map(Msg::Geoeffnet))
}

impl App {
    fn update(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Geoeffnet(id) if id == self.haupt => {
                // das zweite Fenster erst nach dem ersten öffnen
                let (id2, t) = window::open(window::Settings { size: iced::Size::new(320.0, 200.0), position: window::Position::Specific(iced::Point::new(700.0, 120.0)), ..Default::default() });
                self.gelöst = Some(id2); self.log.push("Hauptfenster offen, gelöstes Fenster wird geöffnet".into());
                t.map(Msg::Geoeffnet)
            }
            Msg::Geoeffnet(id) => { self.log.push(format!("gelöstes Fenster offen: {id:?}")); Task::batch([window::position(id).then(move |p| window::size(id).map(move |s| Msg::Lage(id, p, s)))]) }
            Msg::Lage(id, p, s) => { self.log.push(format!("Lage {id:?}: Position {p:?}, Größe {}×{}", s.width, s.height)); Task::none() }
            Msg::Zaehle => { self.zaehler += 1; Task::none() }
            Msg::Tick => {
                self.ticks += 1;
                if self.ticks == 3 { return self.update(Msg::Zaehle) }
                if self.ticks == 6 { if let Some(id) = self.gelöst { return window::close(id) } }
                if self.ticks == 10 {
                    println!("ERGEBNIS zaehler={} (beide Sichten lesen dasselbe Modell) gelöst_zu={} Protokoll={:?}", self.zaehler, self.gelöst.is_none(), self.log);
                    return iced::exit()
                }
                Task::none()
            }
            Msg::Geschlossen(id) => { if Some(id) == self.gelöst { self.gelöst = None; self.log.push("gelöstes Fenster geschlossen (Modell lebt weiter)".into()) } Task::none() }
        }
    }
    fn view(&self, id: window::Id) -> Element<'_, Msg> {
        if id == self.haupt { column![text(format!("Hauptfenster · Zähler {}", self.zaehler)), button("zählen").on_press(Msg::Zaehle), text(if self.gelöst.is_some() { "Bereich ist gelöst" } else { "Bereich ist angedockt" })].spacing(8).padding(16).into() }
        else { column![text(format!("Gelöstes Fenster · Zähler {}", self.zaehler)), button("zählen").on_press(Msg::Zaehle)].spacing(8).padding(16).into() }
    }
    fn abo(&self) -> Subscription<Msg> { Subscription::batch([iced::time::every(Duration::from_millis(250)).map(|_| Msg::Tick), window::close_events().map(Msg::Geschlossen)]) }
}

fn main() -> iced::Result {
    iced::daemon(boot, App::update, App::view).title(|_: &App, _| "Spike Zwei Fenster".to_string()).subscription(App::abo).run()
}
