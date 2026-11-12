// SPDX-License-Identifier: GPL-3.0-or-later
//! JUnit-XML für Pipelines (`vcc test --junit datei.xml`). Zuordnung: fehlgeschlagen → `<failure>`, Zeitüberschreitung → `<error>`,
//! übersprungen → `<skipped>`; Hinweise (Zyklenbudget ohne Fehlerstufe) und Messwerte stehen als `<system-out>` bei einem bestandenen Fall.
use super::model::{Level, Outcome, State};

/// Ergebnisse einer Testdatei (oder der statischen Prüfungen) als eine `testsuite`
pub struct SuiteResult { pub name: String, pub outcomes: Vec<Outcome>, pub millis: u64 }

fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"), '<' => o.push_str("&lt;"), '>' => o.push_str("&gt;"), '"' => o.push_str("&quot;"), '\'' => o.push_str("&apos;"),
            c if (c as u32) < 0x20 && c != '\n' && c != '\t' && c != '\r' => {}      // in XML 1.0 nicht erlaubt
            c => o.push(c),
        }
    }
    o
}

fn secs(ms: u64) -> String { format!("{}.{:03}", ms / 1000, ms % 1000) }

pub fn render(suites: &[SuiteResult]) -> String {
    let count = |s: &SuiteResult, f: &dyn Fn(&Outcome) -> bool| s.outcomes.iter().filter(|o| f(o)).count();
    let failure = |o: &Outcome| o.state == State::Failed && o.level == Level::Error;
    let error = |o: &Outcome| o.state == State::TimedOut;
    let skipped = |o: &Outcome| o.state == State::Skipped;
    let total = |f: &dyn Fn(&Outcome) -> bool| suites.iter().map(|s| count(s, f)).sum::<usize>();
    let mut x = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    x.push_str(&format!("<testsuites name=\"vcc test\" tests=\"{}\" failures=\"{}\" errors=\"{}\" skipped=\"{}\" time=\"{}\">\n",
        total(&|_| true), total(&failure), total(&error), total(&skipped), secs(suites.iter().map(|s| s.millis).sum())));
    for s in suites {
        x.push_str(&format!("  <testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" errors=\"{}\" skipped=\"{}\" time=\"{}\">\n",
            esc(&s.name), s.outcomes.len(), count(s, &failure), count(s, &error), count(s, &skipped), secs(s.millis)));
        for o in &s.outcomes {
            x.push_str(&format!("    <testcase classname=\"{}\" name=\"{}\"", esc(&s.name), esc(&o.name)));
            let loc = o.location.as_ref().map(|(f, l)| format!("{f}:{l}\n")).unwrap_or_default();
            let note = match (&o.measured, o.state, o.level) {
                (_, State::Failed, Level::Hint) => Some(format!("HINWEIS: {}{}", o.detail, o.measured.as_ref().map(|m| format!(" ({m})")).unwrap_or_default())),
                (Some(m), State::Passed, _) => Some(m.clone()),
                _ => None,
            };
            let body = match o.state {
                State::Failed if o.level == Level::Error => format!("      <failure message=\"{}\" type=\"fehlgeschlagen\">{}{}</failure>\n", esc(first_line(&o.detail)), esc(&loc), esc(&o.detail)),
                State::TimedOut => format!("      <error message=\"{}\" type=\"Zeitüberschreitung\">{}{}</error>\n", esc(first_line(&o.detail)), esc(&loc), esc(&o.detail)),
                State::Skipped => format!("      <skipped message=\"{}\"/>\n", esc(&o.detail)),
                _ => String::new(),
            };
            let out = note.map(|n| format!("      <system-out>{}</system-out>\n", esc(&n))).unwrap_or_default();
            if body.is_empty() && out.is_empty() { x.push_str("/>\n"); } else { x.push_str(">\n"); x.push_str(&body); x.push_str(&out); x.push_str("    </testcase>\n"); }
        }
        x.push_str("  </testsuite>\n");
    }
    x.push_str("</testsuites>\n");
    x
}

fn first_line(s: &str) -> &str { s.lines().next().unwrap_or("") }
