// SPDX-License-Identifier: GPL-3.0-or-later
//! `DocStore`: offene Dokumente je Id (Konzept 4.2). Gerüst für E1 ff.: Die Dokumentmodelle (`vc-ide-model`) leben hier in Rust samt Verlauf,
//! die Oberfläche hält nur Anzeigezustand. Muster für alle Editoren: `open → view`, `apply(op) → delta`, `undo`, `redo`, `save`, `close`.
use serde_json::Value;
use std::collections::BTreeMap;

/// Fehler eines Befehls: Code und Parameter statt fertigem Text (die Oberfläche formatiert mit dem Katalog, Konzept 6.2)
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Fehler { pub code: String, pub args: Value }
impl Fehler {
    pub fn neu(code: &str) -> Fehler { Fehler { code: code.into(), args: Value::Object(Default::default()) } }
    pub fn mit(code: &str, args: Value) -> Fehler { Fehler { code: code.into(), args } }
}

/// Was der Export-Dialog von einem Medien-Dokument braucht
pub struct ExportQuelle { pub rel: String, pub kind: String, pub name: String, pub media_text: String }

/// Ein offenes Dokument. `view` ist das Anzeige-DTO, `apply` nimmt eine kleine Absicht (`op`) und liefert ein Delta.
pub trait Dokument: Send {
    fn art(&self) -> &'static str;
    fn view(&self) -> Value;
    fn apply(&mut self, op: &Value) -> Result<Value, Fehler>;
    fn undo(&mut self) -> Result<Value, Fehler>;
    fn redo(&mut self) -> Result<Value, Fehler>;
    fn save(&mut self) -> Result<(), Fehler>;
    fn undo_tiefe(&self) -> usize;
    /// Quelle für den Export (Medien-Editoren): Pfad, Art, Name und der Stand der Mediendatei (auch ungespeichert)
    fn export_quelle(&self) -> Option<ExportQuelle> { None }
    fn ungesichert(&self) -> bool;
}

#[derive(Default)]
pub struct DocStore { next: u64, docs: BTreeMap<u64, Box<dyn Dokument>> }

impl DocStore {
    pub fn insert(&mut self, d: Box<dyn Dokument>) -> u64 { self.next += 1; self.docs.insert(self.next, d); self.next }
    pub fn get(&mut self, id: u64) -> Result<&mut Box<dyn Dokument>, Fehler> { self.docs.get_mut(&id).ok_or_else(|| Fehler::mit("doc-unbekannt", serde_json::json!({ "id": id }))) }
    pub fn close(&mut self, id: u64) -> bool { self.docs.remove(&id).is_some() }
    pub fn list(&self) -> Vec<Value> { self.docs.iter().map(|(id, d)| serde_json::json!({ "docId": id, "kind": d.art(), "dirty": d.ungesichert(), "undoDepth": d.undo_tiefe() })).collect() }
    pub fn len(&self) -> usize { self.docs.len() }
    pub fn is_empty(&self) -> bool { self.docs.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    /// Kleines Beispieldokument: ein Zähler mit Verlauf (Muster für `PixelDoc` & Co.)
    struct Zaehler { n: i64, verlauf: Vec<i64>, redo: Vec<i64>, gespeichert: i64 }
    impl Dokument for Zaehler {
        fn art(&self) -> &'static str { "zaehler" }
        fn view(&self) -> Value { json!({ "n": self.n }) }
        fn apply(&mut self, op: &Value) -> Result<Value, Fehler> {
            let d = op["add"].as_i64().ok_or_else(|| Fehler::neu("op-ungueltig"))?;
            self.verlauf.push(self.n); self.redo.clear(); self.n += d; Ok(json!({ "n": self.n }))
        }
        fn undo(&mut self) -> Result<Value, Fehler> { let v = self.verlauf.pop().ok_or_else(|| Fehler::neu("undo-leer"))?; self.redo.push(self.n); self.n = v; Ok(json!({ "n": v })) }
        fn redo(&mut self) -> Result<Value, Fehler> { let v = self.redo.pop().ok_or_else(|| Fehler::neu("redo-leer"))?; self.verlauf.push(self.n); self.n = v; Ok(json!({ "n": v })) }
        fn save(&mut self) -> Result<(), Fehler> { self.gespeichert = self.n; Ok(()) }
        fn undo_tiefe(&self) -> usize { self.verlauf.len() }
        fn ungesichert(&self) -> bool { self.n != self.gespeichert }
    }
    #[test]
    fn store_oeffnet_aendert_macht_rueckgaengig_und_schliesst() {
        let mut s = DocStore::default();
        let id = s.insert(Box::new(Zaehler { n: 0, verlauf: vec![], redo: vec![], gespeichert: 0 }));
        let d = s.get(id).unwrap();
        assert_eq!(d.apply(&json!({"add": 5})).unwrap(), json!({"n": 5})); assert!(d.ungesichert());
        assert_eq!(d.undo().unwrap(), json!({"n": 0})); assert_eq!(d.redo().unwrap(), json!({"n": 5}));
        d.save().unwrap(); assert!(!d.ungesichert()); assert_eq!(d.undo_tiefe(), 1);
        assert_eq!(d.apply(&json!({})).unwrap_err().code, "op-ungueltig");
        assert_eq!(s.list().len(), 1); assert!(s.close(id)); assert!(s.is_empty());
        assert_eq!(s.get(id).err().unwrap().code, "doc-unbekannt");
    }
}
