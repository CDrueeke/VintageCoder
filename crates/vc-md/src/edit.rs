// SPDX-License-Identifier: GPL-3.0-or-later
//! Schreibhilfen als reine Textfunktionen. Positionen sind **Zeichenindizes** (wie im Editor), nicht Bytes.
//! Jede Funktion liefert den neuen Text und die neue Auswahl `a..b`; der Editor setzt sie als **einen** Rückgängig-Schritt.

use crate::lex;

#[derive(Clone, PartialEq, Debug)]
pub struct Edit { pub text: String, pub a: usize, pub b: usize }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ListKind { Bullet, Numbered, Task, Quote }

fn chars(s: &str) -> Vec<char> { s.chars().collect() }
fn string(c: &[char]) -> String { c.iter().collect() }
fn norm(a: usize, b: usize, n: usize) -> (usize, usize) { let (a, b) = (a.min(b).min(n), a.max(b).min(n)); (a, b) }
/// Zeichenindex des Zeilenanfangs und -endes (ohne `\n`) der Zeile, in der `idx` liegt
fn line_span(c: &[char], idx: usize) -> (usize, usize) {
    let idx = idx.min(c.len());
    let s = c[..idx].iter().rposition(|x| *x == '\n').map_or(0, |p| p + 1);
    let e = c[idx..].iter().position(|x| *x == '\n').map_or(c.len(), |p| idx + p);
    (s, e)
}
/// Zeilen (Anfang, Ende) im Bereich `a..b`; endet die Auswahl am Anfang einer Zeile, gehört diese nicht dazu
fn lines_of(c: &[char], a: usize, b: usize) -> Vec<(usize, usize)> {
    let mut out = vec![]; let (mut s, _) = line_span(c, a);
    let last_idx = if b > a && c.get(b.wrapping_sub(1)) == Some(&'\n') { b - 1 } else { b };
    loop { let (ls, le) = line_span(c, s); out.push((ls, le)); if le >= last_idx || le >= c.len() { break; } s = le + 1; }
    out
}
fn replace_lines(c: &[char], lines: &[(usize, usize)], new: &[String]) -> (String, usize, usize) {
    let (first, last) = (lines[0].0, lines[lines.len() - 1].1);
    let mut out: Vec<char> = c[..first].to_vec(); let body = new.join("\n"); let bl = body.chars().count();
    out.extend(body.chars()); out.extend_from_slice(&c[last..]);
    (string(&out), first, first + bl)
}

// ---------------------------------------------------------------- Auszeichnung

fn is_word(c: char) -> bool { c.is_alphanumeric() || c == '_' }

/// Fett (`**`), Kursiv (`*`), Code (`` ` ``), Durchgestrichen (`~~`) umschalten
pub fn toggle_wrap(text: &str, a: usize, b: usize, mark: &str) -> Edit {
    let c = chars(text); let m: Vec<char> = mark.chars().collect(); let l = m.len();
    let (mut sa, mut sb) = norm(a, b, c.len());
    if sa == sb { while sa > 0 && is_word(c[sa - 1]) { sa -= 1; } while sb < c.len() && is_word(c[sb]) { sb += 1; } }
    if sa == sb { let mut o = c[..sa].to_vec(); o.extend(&m); o.extend(&m); o.extend_from_slice(&c[sa..]); return Edit { text: string(&o), a: sa + l, b: sa + l }; }
    let single = l == 1 && (m[0] == '*' || m[0] == '_');
    let same = |i: Option<usize>| i.and_then(|i| c.get(i)).map_or(false, |x| *x == m[0]);
    let outside = sa >= l && sb + l <= c.len() && c[sa - l..sa] == m[..] && c[sb..sb + l] == m[..] && !(single && (same(sa.checked_sub(l + 1)) || same(Some(sb + l))));
    if outside { let mut o = c[..sa - l].to_vec(); o.extend_from_slice(&c[sa..sb]); o.extend_from_slice(&c[sb + l..]); return Edit { text: string(&o), a: sa - l, b: sb - l }; }
    let inside = sb - sa >= 2 * l && c[sa..sa + l] == m[..] && c[sb - l..sb] == m[..] && !(single && sb - sa > 2 && (c[sa + 1] == m[0] || c[sb - 2] == m[0]));
    if inside { let mut o = c[..sa].to_vec(); o.extend_from_slice(&c[sa + l..sb - l]); o.extend_from_slice(&c[sb..]); return Edit { text: string(&o), a: sa, b: sb - 2 * l }; }
    let mut o = c[..sa].to_vec(); o.extend(&m); o.extend_from_slice(&c[sa..sb]); o.extend(&m); o.extend_from_slice(&c[sb..]);
    Edit { text: string(&o), a: sa + l, b: sb + l }
}

/// Link einfügen: `[Auswahl](url)`, die Adresse ist markiert. Ohne Auswahl entsteht `[Linktext](url)` mit markiertem Linktext.
pub fn link_insert(text: &str, a: usize, b: usize) -> Edit {
    let c = chars(text); let (sa, sb) = norm(a, b, c.len());
    let sel: String = if sa == sb { "Linktext".into() } else { string(&c[sa..sb]) };
    let ins = format!("[{sel}](url)"); let mut o = c[..sa].to_vec(); o.extend(ins.chars()); o.extend_from_slice(&c[sb..]);
    let n = sel.chars().count();
    if sa == sb { Edit { text: string(&o), a: sa + 1, b: sa + 1 + n } } else { Edit { text: string(&o), a: sa + n + 3, b: sa + n + 6 } }
}

/// Bild einfügen: `![Alt](pfad)`; die Auswahl wird zum Alt-Text, ohne Auswahl steht dort „Beschreibung“ (markiert)
pub fn image_insert(text: &str, a: usize, b: usize, path: &str) -> Edit {
    let c = chars(text); let (sa, sb) = norm(a, b, c.len());
    let alt: String = if sa == sb { "Beschreibung".into() } else { string(&c[sa..sb]) };
    let ins = format!("![{alt}]({path})"); let mut o = c[..sa].to_vec(); o.extend(ins.chars()); o.extend_from_slice(&c[sb..]);
    let n = alt.chars().count();
    Edit { text: string(&o), a: sa + 2, b: sa + 2 + n }
}

/// Code: Auswahl über mehrere Zeilen wird ein Zaunblock (```), sonst `` ` `` um die Auswahl
pub fn code_toggle(text: &str, a: usize, b: usize) -> Edit {
    let c = chars(text); let (sa, sb) = norm(a, b, c.len());
    if sa == sb || !c[sa..sb].contains(&'\n') { return toggle_wrap(text, a, b, "`"); }
    let end = if c[sb - 1] == '\n' { sb - 1 } else { sb };
    let mut o = c[..sa].to_vec(); o.extend("```\n".chars()); o.extend_from_slice(&c[sa..end]); o.extend("\n```".chars()); o.extend_from_slice(&c[end..]);
    Edit { text: string(&o), a: sa + 4, b: sa + 4 + (end - sa) }
}

/// Block (Tabelle, Bild …) an der Eingabemarke einfügen: auf einer leeren Zeile dort, sonst nach einer Leerzeile hinter der aktuellen Zeile.
/// Die Marke steht danach am Anfang des Blocks (`+ skip` Zeichen weiter).
pub fn insert_block(text: &str, idx: usize, block: &str, skip: usize) -> Edit {
    let c = chars(text); let idx = idx.min(c.len()); let (s, e) = line_span(&c, idx);
    let blank = c[s..e].iter().all(|x| x.is_whitespace());
    let prev_filled = s >= 2 && c[s - 1] == '\n' && c[s - 2] != '\n';
    let (at, pre) = if blank { (s, if prev_filled { "\n".to_string() } else { String::new() }) } else { (e, "\n\n".to_string()) };
    let mut o = c[..at].to_vec(); o.extend(pre.chars()); let start = o.len(); o.extend(block.chars());
    let tail_nl = c.get(at).map_or(true, |x| *x == '\n') ; o.extend_from_slice(&c[at.min(c.len())..]);
    let mut text = string(&o); if !tail_nl { text.insert(start + block.chars().count(), '\n'); }
    if blank && e >= c.len() { text.push('\n'); }
    Edit { text, a: start + skip, b: start + skip }
}

// ---------------------------------------------------------------- Zeilenpräfixe

#[derive(Clone, PartialEq, Debug)]
enum Marker { None, Bullet(char), Number(u64, char) }
#[derive(Clone, Debug)]
struct Prefix { indent: String, quote: String, marker: Marker, task: Option<char>, content: usize }

/// Zerlegt den Anfang einer Zeile: Einrückung, Zitatzeichen, Listenzeichen, Aufgabenkästchen; `content` = Zeichenindex des Inhalts
fn parse_prefix(line: &[char]) -> Prefix {
    let mut i = 0; while i < line.len() && line[i] == ' ' { i += 1; }
    let indent: String = line[..i].iter().collect(); let qs = i; let mut quote = String::new();
    while i < line.len() && line[i] == '>' { i += 1; if i < line.len() && line[i] == ' ' { i += 1; } let mut k = i; while k < line.len() && line[k] == ' ' { k += 1; } if k < line.len() && line[k] == '>' { i = k; } }
    if i > qs { quote = line[qs..i].iter().collect(); }
    let mut j = i; while j < line.len() && line[j] == ' ' { j += 1; }
    let mut marker = Marker::None; let mut task = None; let mut content = i;
    if j + 1 < line.len() + 0 && matches!(line[j], '-' | '*' | '+') && line.get(j + 1) == Some(&' ') { marker = Marker::Bullet(line[j]); content = j + 2; }
    else { let d = line[j..].iter().take_while(|c| c.is_ascii_digit()).count();
        if (1..=9).contains(&d) && matches!(line.get(j + d), Some('.') | Some(')')) && line.get(j + d + 1) == Some(&' ') { marker = Marker::Number(string(&line[j..j + d]).parse().unwrap_or(1), line[j + d]); content = j + d + 2; } }
    if marker != Marker::None && line.len() >= content + 3 && line[content] == '[' && matches!(line[content + 1], ' ' | 'x' | 'X') && line[content + 2] == ']' && (line.len() == content + 3 || line[content + 3] == ' ') {
        task = Some(line[content + 1]); content = (content + 4).min(line.len());
    }
    if marker == Marker::None && quote.is_empty() { content = indent.chars().count(); }
    Prefix { indent, quote, marker, task, content }
}

fn kind_matches(p: &Prefix, k: ListKind) -> bool {
    match k { ListKind::Quote => !p.quote.is_empty(), ListKind::Bullet => matches!(p.marker, Marker::Bullet(_)) && p.task.is_none(), ListKind::Numbered => matches!(p.marker, Marker::Number(..)), ListKind::Task => p.task.is_some() }
}

/// Aufzählung, Nummerierung, Aufgabe oder Zitat für die Zeilen der Auswahl setzen; sind alle schon so, wird es entfernt
pub fn list_toggle(text: &str, a: usize, b: usize, kind: ListKind) -> Edit {
    let c = chars(text); let (sa, sb) = norm(a, b, c.len());
    let lines = lines_of(&c, sa, sb);
    let rows: Vec<(Vec<char>, Prefix)> = lines.iter().map(|(s, e)| { let l = c[*s..*e].to_vec(); let p = parse_prefix(&l); (l, p) }).collect();
    let single = rows.len() == 1;
    let active: Vec<usize> = (0..rows.len()).filter(|i| single || !rows[*i].0.iter().all(|c| c.is_whitespace())).collect();
    let all = !active.is_empty() && active.iter().all(|i| kind_matches(&rows[*i].1, kind));
    let mut n = 0u64; let mut new: Vec<String> = vec![];
    for (i, (l, p)) in rows.iter().enumerate() {
        if !active.contains(&i) { new.push(string(l)); continue; }
        let rest = string(&l[p.content.min(l.len())..]);
        let out = if kind == ListKind::Quote {
            if all { format!("{}{}{}", p.indent, quote_strip(&p.quote), string(&l[p.content.min(l.len())..]).as_str().to_string().chars().collect::<String>()).replace("\u{0}", "") } else { format!("{}> {}", p.indent, string(&l[p.indent.chars().count()..])) }
        } else if all { format!("{}{}{}", p.indent, p.quote, rest) } else {
            n += 1;
            let mk = match kind { ListKind::Bullet => "- ".to_string(), ListKind::Task => "- [ ] ".to_string(), _ => format!("{n}. ") };
            format!("{}{}{}{}", p.indent, p.quote, mk, rest)
        };
        new.push(out);
    }
    let (t, x, y) = replace_lines(&c, &lines, &new); Edit { text: t, a: x, b: y }
}
/// Rest der Zitatzeichen beim Entfernen: nur das äußerste `> ` fällt weg
fn quote_strip(q: &str) -> String { q.strip_prefix('>').map(|r| r.strip_prefix(' ').unwrap_or(r).to_string()).unwrap_or_default() }

/// Überschriftenebene um `delta` ändern (0 = keine Überschrift, höchstens 6)
pub fn heading_shift(text: &str, a: usize, b: usize, delta: i32) -> Edit {
    let c = chars(text); let (sa, sb) = norm(a, b, c.len());
    let lines = lines_of(&c, sa, sb);
    let new: Vec<String> = lines.iter().map(|(s, e)| {
        let l = &c[*s..*e]; let ind = l.iter().take_while(|c| **c == ' ').count();
        if ind >= 4 { return string(l); }
        let h = l[ind..].iter().take_while(|c| **c == '#').count();
        let level = if (1..=6).contains(&h) && (ind + h == l.len() || l[ind + h] == ' ') { h } else { 0 };
        let content = if level > 0 { string(&l[ind + h..]).trim_start().to_string() } else { string(&l[ind..]) };
        let nl = (level as i32 + delta).clamp(0, 6) as usize;
        if nl == 0 { content } else if content.is_empty() { "#".repeat(nl) + " " } else { format!("{} {}", "#".repeat(nl), content) }
    }).collect();
    let (t, _x, y) = replace_lines(&c, &lines, &new); Edit { text: t, a: y, b: y }
}

/// Aufgabe in der Zeile bei `idx` umschalten (`[ ]` ↔ `[x]`); ein Listenpunkt ohne Kästchen bekommt eins. Keine Listenzeile: `None`.
pub fn toggle_task(text: &str, idx: usize) -> Option<Edit> {
    let c = chars(text); let (s, e) = line_span(&c, idx); let l = c[s..e].to_vec(); let p = parse_prefix(&l);
    if p.marker == Marker::None { return None; }
    let mark_at = p.content - if p.task.is_some() { 4 } else { 0 };
    let new_line: String = match p.task {
        Some(' ') => { let mut x = l.clone(); x[mark_at + 1] = 'x'; string(&x) }
        Some(_) => { let mut x = l.clone(); x[mark_at + 1] = ' '; string(&x) }
        None => { let mut x = l[..p.content].to_vec(); x.extend("[ ] ".chars()); x.extend_from_slice(&l[p.content..]); string(&x) }
    };
    let (t, _, y) = replace_lines(&c, &[(s, e)], &[new_line]); let _ = y;
    let col = idx.saturating_sub(s) + if p.task.is_none() { 4 } else { 0 };
    Some(Edit { text: t, a: s + col, b: s + col })
}

fn fence_before(c: &[char], line_start: usize) -> bool {
    let text = string(&c[..line_start]); let mut st = lex::State::new(&string(c));
    for (n, l) in text.split('\n').enumerate() { if n * 0 == 0 && line_start == 0 { break; } let _ = lex::line(l, n, &mut st); }
    st.in_fence()
}
fn front_at(c: &[char], line_start: usize) -> bool { let n = c[..line_start].iter().filter(|x| **x == '\n').count(); lex::State::new(&string(c)).in_front(n) }

/// Eingabetaste in einer Liste oder einem Zitat: neuer Punkt mit gleichem Präfix; auf einem **leeren** Punkt endet die Liste.
/// `None`, wenn die Zeile keine Liste ist, die Auswahl nicht leer ist oder die Zeile in einem Codeblock oder im Front Matter liegt.
pub fn continue_list(text: &str, idx: usize) -> Option<Edit> {
    let c = chars(text); if idx > c.len() { return None; }
    let (s, e) = line_span(&c, idx); let l = &c[s..e]; let p = parse_prefix(l);
    if p.marker == Marker::None && p.quote.is_empty() { return None; }
    if fence_before(&c, s) || front_at(&c, s) { return None; }
    let col = idx - s; if col < p.content { return None; }
    let rest_after = string(&l[col..]);
    let empty_before = l[p.content..col].iter().all(|c| c.is_whitespace());
    if empty_before && rest_after.trim().is_empty() {
        // leerer Punkt: Präfix entfernen
        let mut o = c[..s].to_vec(); o.extend_from_slice(&c[e..]);
        return Some(Edit { text: string(&o), a: s, b: s });
    }
    let mk = match (&p.marker, p.task) { (Marker::Bullet(ch), Some(_)) => format!("{ch} [ ] "), (Marker::Bullet(ch), None) => format!("{ch} "), (Marker::Number(n, d), t) => format!("{}{d} {}", n + 1, if t.is_some() { "[ ] " } else { "" }), (Marker::None, _) => String::new() };
    let pre = format!("{}{}{}", p.indent, p.quote, mk);
    let mut o = c[..idx].to_vec(); o.push('\n'); o.extend(pre.chars()); let at = o.len(); o.extend_from_slice(&c[idx..]);
    Some(Edit { text: string(&o), a: at, b: at })
}

/// Einrücken (`out = false`) oder Ausrücken in Listenzeilen; sonst `None` (dann bleibt Tab, was es ist)
pub fn indent(text: &str, a: usize, b: usize, out: bool) -> Option<Edit> {
    let c = chars(text); let (sa, sb) = norm(a, b, c.len()); let lines = lines_of(&c, sa, sb);
    let rows: Vec<(Vec<char>, Prefix)> = lines.iter().map(|(s, e)| { let l = c[*s..*e].to_vec(); let p = parse_prefix(&l); (l, p) }).collect();
    if rows.iter().any(|(_, p)| p.marker == Marker::None) || fence_before(&c, lines[0].0) { return None; }
    let mut new = vec![]; let (mut d_first, mut d_all) = (0isize, 0isize);
    for (i, (l, p)) in rows.iter().enumerate() {
        let w = match p.marker { Marker::Number(n, _) => n.to_string().len() + 2, _ => 2 };
        let s = string(l);
        let ns = if out { let k = s.chars().take_while(|c| *c == ' ').count().min(w); s.chars().skip(k).collect::<String>() } else { format!("{}{}", " ".repeat(w), s) };
        let d = ns.chars().count() as isize - s.chars().count() as isize; if i == 0 { d_first = d; } d_all += d; new.push(ns);
    }
    if out && d_all == 0 { return None; }
    let (t, _, _) = replace_lines(&c, &lines, &new);
    let na = (sa as isize + d_first).max(lines[0].0 as isize) as usize; let nb = (sb as isize + d_all).max(na as isize) as usize;
    Some(Edit { text: t, a: na, b: nb })
}

// ---------------------------------------------------------------- Tabellen

fn wide(c: char) -> usize {
    let u = c as u32;
    if matches!(u, 0x0300..=0x036F | 0x200B..=0x200F | 0xFE00..=0xFE0F) { 0 }
    else if matches!(u, 0x1100..=0x115F | 0x2E80..=0xA4CF | 0xAC00..=0xD7A3 | 0xF900..=0xFAFF | 0xFE30..=0xFE6F | 0xFF00..=0xFF60 | 0xFFE0..=0xFFE6 | 0x1F300..=0x1F64F | 0x1F900..=0x1F9FF | 0x20000..=0x3FFFD) { 2 } else { 1 }
}
/// Anzeigebreite in Monospace-Zellen (Emoji und CJK doppelt; für manche Zeichen weicht die Schrift ab)
pub fn width(s: &str) -> usize { s.chars().map(wide).sum() }

fn is_sep_cells(cells: &[String]) -> bool { !cells.is_empty() && cells.iter().all(|c| { let t = c.trim(); t.contains('-') && t.chars().all(|x| matches!(x, '-' | ':')) }) }
/// Zellen einer Tabellenzeile (ohne äußere `|`, getrimmt; `\|` bleibt im Inhalt)
pub fn split_cells(line: &str) -> Vec<String> {
    let t = line.trim(); let t = t.strip_prefix('|').unwrap_or(t);
    let t = if t.ends_with('|') && !t.ends_with("\\|") { &t[..t.len() - 1] } else { t };
    let mut cells = vec![]; let mut cur = String::new(); let mut esc = false;
    for ch in t.chars() { if esc { cur.push(ch); esc = false; } else if ch == '\\' { cur.push(ch); esc = true; } else if ch == '|' { cells.push(cur.trim().to_string()); cur.clear(); } else { cur.push(ch); } }
    cells.push(cur.trim().to_string()); cells
}

/// Zeilenbereich (erste, letzte Zeilennummer) der Tabelle, in der `line` liegt: aufeinanderfolgende Zeilen mit `|`, zweite Zeile ist die Trennzeile
pub fn table_range(lines: &[&str], line: usize) -> Option<(usize, usize)> {
    let has = |i: usize| lines.get(i).map_or(false, |l| l.contains('|') && !l.trim().is_empty());
    if !has(line) { return None; }
    let (mut f, mut l) = (line, line);
    while f > 0 && has(f - 1) { f -= 1; } while has(l + 1) { l += 1; }
    (l > f && is_sep_cells(&split_cells(lines[f + 1]))).then_some((f, l))
}

fn align_of(sep: &str) -> (bool, bool) { let t = sep.trim(); (t.starts_with(':'), t.ends_with(':')) }

fn format_rows(rows: &[Vec<String>]) -> Vec<String> {
    let n = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut w = vec![3usize; n];
    for (ri, r) in rows.iter().enumerate() { if ri == 1 { continue; } for (i, c) in r.iter().enumerate() { w[i] = w[i].max(width(c)); } }
    rows.iter().enumerate().map(|(ri, r)| {
        let mut s = String::from("|");
        for i in 0..n {
            let cell = r.get(i).map(|x| x.as_str()).unwrap_or("");
            if ri == 1 {
                let (l, rt) = align_of(cell); let mut d = "-".repeat(w[i] + 2);
                if l { d.replace_range(0..1, ":"); } if rt { let k = d.len(); d.replace_range(k - 1..k, ":"); }
                s.push_str(&d);
            } else { let pad = w[i].saturating_sub(width(cell)); let (l, rt) = align_of(rows[1].get(i).map(|x| x.as_str()).unwrap_or(""));
                let (pl, pr) = if l && rt { (pad / 2, pad - pad / 2) } else if rt { (pad, 0) } else { (0, pad) };
                s.push_str(&format!(" {}{}{} ", " ".repeat(pl), cell, " ".repeat(pr))); }
            s.push('|');
        }
        s
    }).collect()
}

/// Neue Tabelle als Text (Kopfzeile, Trennzeile, `rows` leere Zeilen); `align`: 0 links, 1 mitte, 2 rechts, sonst ohne
pub fn table_new(cols: usize, rows: usize, header: bool, align: u8) -> String {
    let cols = cols.clamp(1, 20);
    let head: Vec<String> = (1..=cols).map(|i| if header { format!("Spalte {i}") } else { String::new() }).collect();
    let sep: Vec<String> = (0..cols).map(|_| match align { 0 => ":--", 1 => ":-:", 2 => "--:", _ => "---" }.to_string()).collect();
    let mut all = vec![head, sep]; for _ in 0..rows { all.push(vec![String::new(); cols]); }
    format_rows(&all).join("\n")
}

/// Tabelle bei `idx` ausrichten; Zeile und Spalte der Eingabemarke bleiben erhalten. `None`, wenn dort keine Tabelle ist.
pub fn table_align(text: &str, idx: usize) -> Option<Edit> {
    let c = chars(text); let lines: Vec<&str> = text.split('\n').collect();
    let ln = c[..idx.min(c.len())].iter().filter(|x| **x == '\n').count();
    let (f, l) = table_range(&lines, ln)?;
    let rows: Vec<Vec<String>> = (f..=l).map(|i| split_cells(lines[i])).collect();
    let new = format_rows(&rows);
    let mut out: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    for (k, i) in (f..=l).enumerate() { out[i] = new[k].clone(); }
    let before: usize = out[..ln].iter().map(|s| s.chars().count() + 1).sum();
    Some(Edit { text: out.join("\n"), a: before + 2, b: before + 2 })
}

/// Grenzen der Zellinhalte einer Zeile als Zeichenindizes innerhalb der Zeile
fn cell_bounds(l: &[char]) -> Vec<(usize, usize)> {
    let mut pipes = vec![]; let mut esc = false;
    for (i, ch) in l.iter().enumerate() { if esc { esc = false; } else if *ch == '\\' { esc = true; } else if *ch == '|' { pipes.push(i); } }
    let mut edges: Vec<usize> = vec![]; if pipes.first().map_or(true, |p| l[..*p].iter().any(|c| !c.is_whitespace())) { edges.push(usize::MAX); }
    edges.extend(&pipes); if pipes.last().map_or(true, |p| l[*p + 1..].iter().any(|c| !c.is_whitespace())) { edges.push(l.len()); }
    edges.windows(2).map(|w| { let (mut s, mut e) = (if w[0] == usize::MAX { 0 } else { w[0] + 1 }, w[1]); while s < e && l[s].is_whitespace() { s += 1; } while e > s && l[e - 1].is_whitespace() { e -= 1; } if s == e { s = if w[0] == usize::MAX { 0 } else { (w[0] + 2).min(w[1]) }; e = s; } (s, e) }).collect()
}

/// Tab in einer Tabelle: zur nächsten (oder, mit `back`, vorigen) Zelle; in der letzten Zeile entsteht eine neue Zeile. `None` außerhalb von Tabellen.
pub fn table_cell(text: &str, idx: usize, back: bool) -> Option<Edit> {
    let c = chars(text); let mut lines: Vec<String> = text.split('\n').map(String::from).collect();
    let ln = c[..idx.min(c.len())].iter().filter(|x| **x == '\n').count();
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let (f, mut l) = table_range(&refs, ln)?;
    let (ls, _) = line_span(&c, idx); let col = idx - ls;
    let cur: Vec<char> = lines[ln].chars().collect(); let cb = cell_bounds(&cur);
    let ci = cb.iter().rposition(|(s, _)| *s <= col).unwrap_or(0);
    let (tl, tc): (usize, usize) = if !back {
        if ci + 1 < cb.len() { (ln, ci + 1) } else {
            let mut nl = ln + 1; if nl == f + 1 { nl += 1; }
            if nl > l { let n = split_cells(&lines[f]).len(); lines.insert(l + 1, format!("|{}", " |".repeat(n))); l += 1; let _ = l; }
            (nl, 0)
        }
    } else if ci > 0 { (ln, ci - 1) } else { let mut pl = ln.checked_sub(1)?; if pl == f + 1 { pl = pl.checked_sub(1)?; } if pl < f { return None; } (pl, usize::MAX) };
    // neue Zeile ausrichten
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let (f2, l2) = table_range(&refs, f)?;
    let rows: Vec<Vec<String>> = (f2..=l2).map(|i| split_cells(&lines[i])).collect();
    let new = format_rows(&rows); for (k, i) in (f2..=l2).enumerate() { lines[i] = new[k].clone(); }
    let tcells = cell_bounds(&lines[tl].chars().collect::<Vec<_>>());
    let tc = if tc == usize::MAX { tcells.len().saturating_sub(1) } else { tc.min(tcells.len().saturating_sub(1)) };
    let (s, e) = tcells.get(tc).copied().unwrap_or((0, 0));
    let before: usize = lines[..tl].iter().map(|x| x.chars().count() + 1).sum();
    Some(Edit { text: lines.join("\n"), a: before + s, b: before + e })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ed(e: Edit) -> (String, usize, usize) { (e.text, e.a, e.b) }

    #[test]
    fn wrap_selection_word_empty_and_toggle_back() {
        assert_eq!(ed(toggle_wrap("ein Wort hier", 4, 8, "**")), ("ein **Wort** hier".into(), 6, 10));
        assert_eq!(ed(toggle_wrap("ein Wort hier", 6, 6, "**")), ("ein **Wort** hier".into(), 6, 10), "Wort um den Cursor");
        assert_eq!(ed(toggle_wrap("ab  cd", 3, 3, "`")), ("ab `` cd".into(), 4, 4));
        assert_eq!(ed(toggle_wrap("ein **Wort** hier", 6, 10, "**")), ("ein Wort hier".into(), 4, 8));
        assert_eq!(ed(toggle_wrap("ein **Wort** hier", 4, 12, "**")), ("ein Wort hier".into(), 4, 8), "Auswahl samt Zeichen");
        assert_eq!(ed(toggle_wrap("äö ü", 0, 2, "*")), ("*äö* ü".into(), 1, 3));
    }

    #[test]
    fn italic_does_not_eat_bold_marks() {
        let r = toggle_wrap("**fett**", 2, 6, "*"); assert_eq!(r.text, "***fett***");
    }

    #[test]
    fn link_and_image() {
        assert_eq!(ed(link_insert("siehe Handbuch", 6, 14)), ("siehe [Handbuch](url)".into(), 17, 20));
        assert_eq!(ed(link_insert("", 0, 0)), ("[Linktext](url)".into(), 1, 9));
        assert_eq!(ed(image_insert("", 0, 0, "pics/a.png")), ("![Beschreibung](pics/a.png)".into(), 2, 14));
    }

    #[test]
    fn list_toggle_on_and_off_for_several_lines() {
        let r = list_toggle("a\nb\n\nc", 0, 6, ListKind::Bullet); assert_eq!(r.text, "- a\n- b\n\n- c");
        let r2 = list_toggle(&r.text, 0, r.text.chars().count(), ListKind::Bullet); assert_eq!(r2.text, "a\nb\n\nc");
        assert_eq!(list_toggle("a\nb", 0, 3, ListKind::Numbered).text, "1. a\n2. b");
        assert_eq!(list_toggle("- a\n- b", 0, 7, ListKind::Numbered).text, "1. a\n2. b", "anderes Listenzeichen wird ersetzt");
        assert_eq!(list_toggle("a", 0, 0, ListKind::Task).text, "- [ ] a");
        assert_eq!(list_toggle("  x", 3, 3, ListKind::Bullet).text, "  - x", "Einrückung bleibt");
        assert_eq!(list_toggle("a\nb", 0, 3, ListKind::Quote).text, "> a\n> b");
        assert_eq!(list_toggle("> a\n> b", 0, 7, ListKind::Quote).text, "a\nb");
        assert_eq!(list_toggle("a\nb\n", 0, 2, ListKind::Bullet).text, "- a\nb\n", "Auswahl endet am Zeilenanfang: Zeile b gehört nicht dazu");
    }

    #[test]
    fn headings_shift_within_bounds() {
        assert_eq!(heading_shift("Text", 0, 0, 1).text, "# Text");
        assert_eq!(heading_shift("# Text", 0, 0, 1).text, "## Text");
        assert_eq!(heading_shift("###### T", 0, 0, 1).text, "###### T");
        assert_eq!(heading_shift("## T", 0, 0, -1).text, "# T");
        assert_eq!(heading_shift("# T", 0, 0, -1).text, "T");
        assert_eq!(heading_shift("T", 0, 0, -1).text, "T");
        assert_eq!(heading_shift("#kein\nb", 0, 7, 1).text, "# #kein\n# b");
    }

    #[test]
    fn task_toggle() {
        assert_eq!(ed(toggle_task("- [ ] a", 3).unwrap()), ("- [x] a".into(), 3, 3));
        assert_eq!(toggle_task("- [X] a", 0).unwrap().text, "- [ ] a");
        assert_eq!(toggle_task("- a", 2).unwrap().text, "- [ ] a");
        assert!(toggle_task("kein punkt", 0).is_none());
    }

    #[test]
    fn enter_continues_and_ends_lists() {
        assert_eq!(ed(continue_list("- eins", 6).unwrap()), ("- eins\n- ".into(), 9, 9));
        assert_eq!(continue_list("1. a", 4).unwrap().text, "1. a\n2. ");
        assert_eq!(continue_list("- [x] a", 7).unwrap().text, "- [x] a\n- [ ] ");
        assert_eq!(continue_list("  * a", 5).unwrap().text, "  * a\n  * ");
        assert_eq!(continue_list("> zitat", 7).unwrap().text, "> zitat\n> ");
        assert_eq!(ed(continue_list("- a\n- ", 6).unwrap()), ("- a\n".into(), 4, 4), "leerer Punkt beendet die Liste");
        assert_eq!(continue_list("- ab", 3).unwrap().text, "- a\n- b", "Zeile wird geteilt");
        assert!(continue_list("kein Punkt", 4).is_none());
        assert!(continue_list("```\n- a\n```", 7).is_none(), "nicht im Codeblock");
        assert!(continue_list("---\nt: x\n---\n- y", 16).is_some());
    }

    #[test]
    fn indent_only_in_lists() {
        assert_eq!(ed(indent("- a\n- b", 6, 6, false).unwrap()), ("- a\n  - b".into(), 8, 8));
        assert_eq!(indent("  - b", 5, 5, true).unwrap().text, "- b");
        assert_eq!(indent("1. a", 0, 0, false).unwrap().text, "   1. a");
        assert!(indent("text", 0, 0, false).is_none());
        assert!(indent("- b", 0, 0, true).is_none());
    }

    #[test]
    fn tables_new_and_align() {
        let t = table_new(2, 1, true, 0);
        assert_eq!(t, "| Spalte 1 | Spalte 2 |\n| :------- | :------- |\n|          |          |".replace("| :------- | :------- |", "|:---------|:---------|"));
        let e = table_align("| Register | Adresse |\n|---|---|\n| Rahmen | $D020 |", 0).unwrap();
        assert_eq!(e.text, "| Register | Adresse |\n|----------|---------|\n| Rahmen   | $D020   |");
        assert!(table_align("kein | Tisch", 0).is_none());
        let r = table_align("| a | b |\n|--:|:-:|\n| 1 | 2 |", 0).unwrap();
        assert_eq!(r.text, "|   a |  b  |\n|----:|:---:|\n|   1 |  2  |");
        assert_eq!(width("a😀ä"), 4);
    }

    #[test]
    fn tab_walks_through_cells_and_adds_rows() {
        let t = "| a | b |\n|---|---|\n| c | d |";
        let e = table_cell(t, 2, false).unwrap(); // in Zelle a → Zelle b
        assert_eq!(&e.text.chars().collect::<Vec<_>>()[e.a..e.b].iter().collect::<String>(), "b");
        let e2 = table_cell(t, 26, false).unwrap(); // letzte Zeile, Zelle d → neue Zeile
        assert_eq!(e2.text.split('\n').count(), 4);
        assert_eq!(e2.a, e2.b);
        let back = table_cell(t, 6, true).unwrap(); // Zelle b → Zelle a
        assert_eq!(&back.text.chars().collect::<Vec<_>>()[back.a..back.b].iter().collect::<String>(), "a");
        assert!(table_cell("kein tisch", 3, false).is_none());
    }

    #[test]
    fn code_toggle_single_and_fenced() {
        assert_eq!(code_toggle("a b", 0, 1).text, "`a` b");
        let e = code_toggle("x\ny\nz", 0, 3); assert_eq!(e.text, "```\nx\ny\n```\nz"); assert_eq!((e.a, e.b), (4, 7));
        assert_eq!(code_toggle("x\ny", 0, 4).text, "```\nx\ny\n```");
    }

    #[test]
    fn blocks_are_inserted_after_a_blank_line() {
        let e = insert_block("Text", 2, "| a |", 2); assert_eq!(e.text, "Text\n\n| a |"); assert_eq!(e.a, 8);
        let e = insert_block("A\n\nB", 2, "| a |", 2); assert_eq!(e.text, "A\n\n| a |\nB");
        let e = insert_block("", 0, "| a |", 0); assert_eq!(e.text, "| a |\n");
    }
}
