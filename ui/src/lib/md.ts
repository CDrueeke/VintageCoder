// SPDX-License-Identifier: GPL-3.0-or-later
// Kleiner, sicherer Markdown-Teil für Hover-Karten der Sprachdienste: Absätze, **fett**, `Code`, Tabellen, Listen. Nur DOM-Knoten und Text, nie innerHTML.

function inline(parent: Node, s: string): void {
  const re = /(\*\*[^*]+\*\*|`[^`]+`)/g; let last = 0; let m: RegExpExecArray | null;
  while ((m = re.exec(s))) {
    if (m.index > last) parent.appendChild(document.createTextNode(s.slice(last, m.index)));
    const tok = m[0]; const el = document.createElement(tok.startsWith("**") ? "strong" : "code");
    el.textContent = tok.startsWith("**") ? tok.slice(2, -2) : tok.slice(1, -1); parent.appendChild(el); last = m.index + tok.length;
  }
  if (last < s.length) parent.appendChild(document.createTextNode(s.slice(last)));
}

const cells = (l: string): string[] => l.trim().replace(/^\|/, "").replace(/\|$/, "").split("|").map((c) => c.trim());
const isSep = (l: string): boolean => /^\s*\|?\s*:?-{2,}/.test(l);

export function renderMd(text: string): HTMLElement {
  const root = document.createElement("div"); root.className = "md";
  const lines = text.split("\n"); let i = 0;
  while (i < lines.length) {
    const l = lines[i]!;
    if (!l.trim()) { i++; continue; }
    if (l.trim().startsWith("|")) {
      const rows: string[] = []; while (i < lines.length && lines[i]!.trim().startsWith("|")) rows.push(lines[i++]!);
      const tb = document.createElement("table"); let head = true;
      for (const r of rows) {
        if (isSep(r)) { head = false; continue; }
        const tr = document.createElement("tr"); if (cells(r)[0]?.startsWith("▶")) tr.className = "here";
        for (const c of cells(r)) { const td = document.createElement(head && rows.some(isSep) ? "th" : "td"); inline(td, c); tr.appendChild(td); }
        tb.appendChild(tr);
      }
      root.appendChild(tb); continue;
    }
    if (/^\s*[-*] /.test(l)) {
      const ul = document.createElement("ul");
      while (i < lines.length && /^\s*[-*] /.test(lines[i]!)) { const li = document.createElement("li"); inline(li, lines[i++]!.replace(/^\s*[-*] /, "")); ul.appendChild(li); }
      root.appendChild(ul); continue;
    }
    const p = document.createElement("p"); const buf: string[] = [];
    while (i < lines.length && lines[i]!.trim() && !lines[i]!.trim().startsWith("|") && !/^\s*[-*] /.test(lines[i]!)) buf.push(lines[i++]!);
    buf.forEach((b, k) => { if (k) p.appendChild(document.createElement("br")); inline(p, b); });
    root.appendChild(p);
  }
  return root;
}

/** Aufteilung einer Hover-Markdown-Antwort (Sprachdienst, BASIC-Hilfe) in die Teile der Karte: Titel, Kurzbeschreibung, Flags, Rumpf und Fußzeile („Diese Zeile: …“) */
export interface HoverTeile { title: string; subtitle: string; flags: string[]; body: string; footer: string }
export function zerlegeHover(markdown: string): HoverTeile {
  const bloecke = markdown.replace(/\r/g, "").split(/\n\s*\n/).map((b) => b.trim()).filter(Boolean);
  const out: HoverTeile = { title: "", subtitle: "", flags: [], body: "", footer: "" };
  if (!bloecke.length) return out;
  const rest: string[] = [];
  let kopfErkannt = false;
  for (const b of bloecke) {
    if (!kopfErkannt) {
      kopfErkannt = true;
      const f = /^\*\*([^*]+)\*\*\s*(?:[–—-]\s*(.*))?$/s.exec(b);   // **LDA** – Akkumulator laden
      if (f && !b.includes("\n")) { out.title = f[1]!.trim(); out.subtitle = (f[2] ?? "").trim(); continue; }
      const c = /^`([^`]+)`$/.exec(b);                                // `#if BEDINGUNG`: Schreibweise als Titel
      if (c) { out.title = c[1]!; continue; }
      const g = /^`([^`]+)`\s+(.*)$/s.exec(b);                        // `STERNE` ist per `#define` definiert …: Name als Titel, Rest als Kurzbeschreibung
      if (g && !b.includes("\n")) { out.title = g[1]!; out.subtitle = g[2]!.replace(/`/g, "").trim(); continue; }
      rest.push(b); continue;
    }
    const fl = /^Flags:\s*`([^`]*)`\s*$/.exec(b);
    if (fl) { out.flags = fl[1]!.split(/\s+/).filter(Boolean); continue; }
    if (/^Diese Zeile:/i.test(b) || /^This line:/i.test(b)) { out.footer = b; continue; }
    rest.push(b);
  }
  out.body = rest.join("\n\n"); return out;
}

/** Hover-Karte in zwei Spalten: links Titel, Kurzbeschreibung und Flags, rechts Tabelle oder Beschreibung, unten die Zeile des Cursors */
export function hoverKarte(markdown: string, label: string): HTMLElement {
  const t = zerlegeHover(markdown);
  const root = document.createElement("div"); root.className = "vc-hv"; root.setAttribute("role", "group"); root.setAttribute("aria-label", label);
  const row = document.createElement("div"); row.className = "vc-hv-row";
  if (t.title) {
    const l = document.createElement("div"); l.className = "vc-hv-l";
    const h = document.createElement("span"); h.className = "vc-hv-t"; h.textContent = t.title; l.appendChild(h);
    if (t.subtitle) { const k = document.createElement("span"); k.className = "vc-hv-k"; k.textContent = t.subtitle; l.appendChild(k); }
    if (t.flags.length) { const fl = document.createElement("div"); fl.className = "vc-hv-flags"; for (const f of t.flags) { const p = document.createElement("span"); p.className = "vc-pill"; p.textContent = f; fl.appendChild(p); } l.appendChild(fl); }
    row.appendChild(l);
  } else row.classList.add("vc-hv-einspaltig");
  const r = document.createElement("div"); r.className = "vc-hv-r"; if (t.body) r.appendChild(renderMd(t.body)); row.appendChild(r);
  root.appendChild(row);
  if (t.footer) { const f = document.createElement("div"); f.className = "vc-hv-ft"; f.appendChild(renderMd(t.footer.replace(/^[^:]+:\s*/, ""))); const lab = document.createElement("span"); lab.className = "vc-hv-ftl"; lab.textContent = t.footer.split(":")[0]! + ":"; f.prepend(lab, " "); root.appendChild(f); }
  return root;
}
