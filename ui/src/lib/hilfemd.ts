// SPDX-License-Identifier: GPL-3.0-or-later
// Markdown der Hilfe als sichere DOM-Knoten (nie innerHTML): Überschriften, Absätze, Listen (auch nummeriert), Zitate, Code, Tabellen, Trennlinien, Hervorhebungen, Links (hilfe:/help: innerhalb der Hilfe) und Bilder (bild:<Kennung>, vom Aufrufer nachgeladen).
export interface Rueckrufe { thema: (id: string) => void; bild: (id: string, alt: string, el: HTMLElement) => void }

const URL_OK = /^(hilfe:|help:)/;
function inline(parent: Node, s: string, rr: Rueckrufe): void {
  const re = /(!\[[^\]]*\]\((?:[^)"]|"[^"]*")*\)|\[[^\]]+\]\((?:[^)"]|"[^"]*")*\)|\*\*[^*]+\*\*|\*[^*\s][^*]*\*|`[^`]+`)/g; let last = 0; let m: RegExpExecArray | null;
  while ((m = re.exec(s))) {
    if (m.index > last) parent.appendChild(document.createTextNode(s.slice(last, m.index)));
    const tok = m[0];
    if (tok.startsWith("![")) {
      const mm = /^!\[([^\]]*)\]\(([^)\s"]*)(?:\s+"[^"]*")?\)$/.exec(tok); const el = document.createElement("span"); el.className = "bild"; const alt = mm?.[1] ?? "";
      const ziel = mm?.[2] ?? ""; if (ziel.startsWith("bild:")) { el.setAttribute("data-bild", ziel.slice(5)); el.textContent = alt; rr.bild(ziel.slice(5), alt, el); } else el.textContent = alt; parent.appendChild(el);
    } else if (tok.startsWith("[")) {
      const mm = /^\[([^\]]+)\]\(([^)\s"]*)(?:\s+"[^"]*")?\)$/.exec(tok); const text = mm?.[1] ?? tok; const ziel = mm?.[2] ?? "";
      if (URL_OK.test(ziel)) { const a = document.createElement("a"); a.href = "#"; a.className = "hl"; a.setAttribute("data-thema", ziel.replace(/^(hilfe:|help:)/, "")); a.textContent = text; a.addEventListener("click", (e) => { e.preventDefault(); rr.thema(a.getAttribute("data-thema") ?? ""); }); parent.appendChild(a); }
      else parent.appendChild(document.createTextNode(text));   // äußere Adressen werden nicht verlinkt
    } else if (tok.startsWith("**")) { const el = document.createElement("strong"); el.textContent = tok.slice(2, -2); parent.appendChild(el); }
    else if (tok.startsWith("`")) { const el = document.createElement("code"); el.textContent = tok.slice(1, -1); parent.appendChild(el); }
    else { const el = document.createElement("em"); el.textContent = tok.slice(1, -1); parent.appendChild(el); }
    last = m.index + tok.length;
  }
  if (last < s.length) parent.appendChild(document.createTextNode(s.slice(last)));
}
const cells = (l: string): string[] => l.trim().replace(/^\|/, "").replace(/\|$/, "").split("|").map((c) => c.trim());
const isSep = (l: string): boolean => /^\s*\|?\s*:?-{2,}/.test(l);

export function renderHilfe(text: string, rr: Rueckrufe): HTMLElement {
  const root = document.createElement("div"); root.className = "hmd";
  const zeilen = text.replace(/\r\n/g, "\n").split("\n"); let i = 0;
  while (i < zeilen.length) {
    const l = zeilen[i]!;
    if (!l.trim()) { i++; continue; }
    if (l.trim().startsWith("```")) { const lang = l.trim().slice(3); const buf: string[] = []; i++; while (i < zeilen.length && !zeilen[i]!.trim().startsWith("```")) buf.push(zeilen[i++]!); i++; const pre = document.createElement("pre"); if (lang) pre.setAttribute("data-lang", lang); const c = document.createElement("code"); c.textContent = buf.join("\n"); pre.appendChild(c); root.appendChild(pre); continue; }
    const h = /^(#{1,4})\s+(.*)$/.exec(l); if (h) { const el = document.createElement(`h${Math.min(4, h[1]!.length + 1)}`); inline(el, h[2]!, rr); root.appendChild(el); i++; continue; }
    if (/^\s*([-*_])\1{2,}\s*$/.test(l)) { root.appendChild(document.createElement("hr")); i++; continue; }
    if (l.trim().startsWith("|")) {
      const rows: string[] = []; while (i < zeilen.length && zeilen[i]!.trim().startsWith("|")) rows.push(zeilen[i++]!);
      const tb = document.createElement("table"); let kopf = rows.some(isSep);
      for (const r of rows) { if (isSep(r)) { kopf = false; continue; } const tr = document.createElement("tr"); const istKopf = kopf; for (const c of cells(r)) { const td = document.createElement(istKopf ? "th" : "td"); inline(td, c, rr); tr.appendChild(td); } tb.appendChild(tr); }
      root.appendChild(tb); continue;
    }
    if (/^\s*>/.test(l)) { const buf: string[] = []; while (i < zeilen.length && /^\s*>/.test(zeilen[i]!)) buf.push(zeilen[i++]!.replace(/^\s*>\s?/, "")); const q = document.createElement("blockquote"); inline(q, buf.join(" "), rr); root.appendChild(q); continue; }
    const li = /^(\s*)([-*]|\d+[.)])\s+(.*)$/;
    if (li.test(l)) {
      const nummeriert = /^\s*\d/.test(l); const ul = document.createElement(nummeriert ? "ol" : "ul"); let cur: HTMLLIElement | null = null;
      while (i < zeilen.length && (li.test(zeilen[i]!) || (cur && /^\s{2,}\S/.test(zeilen[i]!) && !li.test(zeilen[i]!)))) {
        const z = zeilen[i++]!; const m = li.exec(z);
        if (m) { cur = document.createElement("li"); inline(cur, m[3]!, rr); ul.appendChild(cur); } else if (cur) { cur.appendChild(document.createTextNode(" ")); inline(cur, z.trim(), rr); }
      }
      root.appendChild(ul); continue;
    }
    const buf: string[] = [];
    while (i < zeilen.length && zeilen[i]!.trim() && !/^(#{1,4}\s|```|\s*>|\s*\||\s*([-*]|\d+[.)])\s)/.test(zeilen[i]!)) buf.push(zeilen[i++]!.trim());
    if (!buf.length) { i++; continue; }
    const p = document.createElement("p"); inline(p, buf.join(" "), rr); root.appendChild(p);
  }
  return root;
}
