# SPDX-License-Identifier: GPL-3.0-or-later
"""Kleiner Rust-Zerleger für tools/i18n_extract.py: findet Zeichenkettenliterale mit ihrem Kontext (umgebender Aufruf, Argumentnummer).

Nur so genau, wie die Extraktion es braucht: Kommentare, normale/rohe/Byte-Zeichenketten, Zeichenliterale und Lebensdauern werden unterschieden,
Klammern gezählt. Kein vollständiger Parser."""
import re

IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


class Lit:
    __slots__ = ("start", "end", "value", "raw", "callee", "argno", "path", "line", "prefix_ident", "in_macro", "open_pos", "const_item", "opens")

    def __repr__(self):
        return f"Lit({self.value!r} in {self.callee}#{self.argno} @{self.line})"


def unescape(body):
    out, i = [], 0
    while i < len(body):
        c = body[i]
        if c != "\\":
            out.append(c)
            i += 1
            continue
        n = body[i + 1] if i + 1 < len(body) else ""
        i += 2
        if n == "n":
            out.append("\n")
        elif n == "t":
            out.append("\t")
        elif n == "r":
            out.append("\r")
        elif n == "0":
            out.append("\0")
        elif n in '"\\\'':
            out.append(n)
        elif n == "u":
            m = re.match(r"\{([0-9a-fA-F_]+)\}", body[i:])
            out.append(chr(int(m.group(1).replace("_", ""), 16)))
            i += m.end()
        elif n == "x":
            out.append(chr(int(body[i:i + 2], 16)))
            i += 2
        elif n == "\n":
            while i < len(body) and body[i] in " \t\r\n":
                i += 1
        else:
            out.append("\\" + n)
    return "".join(out)


def tokens(src):
    """Erzeugt Lit-Objekte für alle normalen Zeichenkettenliterale (ohne rohe und Byte-Zeichenketten)."""
    lits, stack, i, n = [], [], 0, len(src)
    line = 1
    item_words, brace = [], 0   # erste Bezeichner des aktuellen Elements außerhalb jeder {…}
    last_ident = ""      # Bezeichner (inkl. Pfad, `a::b`, mit führendem `.` bei Methoden) direkt vor der nächsten Klammer
    prev_sig = ""        # letztes bedeutsames Zeichen
    while i < n:
        c = src[i]
        if c == "\n":
            line += 1
            i += 1
            continue
        if c in " \t\r":
            i += 1
            continue
        if src.startswith("//", i):
            j = src.find("\n", i)
            i = n if j < 0 else j
            continue
        if src.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth += 1
                    j += 2
                elif src.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    if src[j] == "\n":
                        line += 1
                    j += 1
            i = j
            continue
        m = re.match(r'(?:b?r)(#*)"', src[i:i + 12])
        if m:
            end = '"' + m.group(1)
            j = src.find(end, i + m.end())
            j = n if j < 0 else j + len(end)
            line += src.count("\n", i, j)
            i = j
            prev_sig = '"'
            continue
        if c == "b" and src.startswith('b"', i):
            j = i + 2
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            line += src.count("\n", i, j)
            i = j + 1
            continue
        if c == '"':
            j = i + 1
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            body = src[i + 1:j]
            lit = Lit()
            lit.start, lit.end, lit.raw, lit.line = i, j + 1, body, line
            lit.value = unescape(body)
            top = stack[-1] if stack else None
            lit.callee = top["callee"] if top else ""
            lit.argno = top["argno"] if top else -1
            lit.in_macro = bool(top and top["macro"])
            lit.open_pos = top["pos"] if top else -1
            lit.opens = [s["pos"] for s in stack]
            lit.const_item = brace == 0 and ("const" in item_words or "static" in item_words) and "fn" not in item_words
            lit.path = [s["callee"] for s in stack]
            lit.prefix_ident = last_ident if prev_sig == "=" else ""
            lits.append(lit)
            line += body.count("\n")
            i = j + 1
            prev_sig = '"'
            continue
        if c == "'":
            m = re.match(r"'(?:\\.[^']*|[^\\'])'", src[i:i + 12])
            if m:
                i += m.end()
                prev_sig = "'"
                continue
            m = re.match(r"'[A-Za-z_][A-Za-z0-9_]*", src[i:])
            i += m.end() if m else 1
            continue
        m = IDENT.match(src, i)
        if m:
            # Pfad und Methodenpunkt mitnehmen
            j = m.end()
            name = m.group(0)
            k = i - 1
            while k >= 0 and src[k] in " \t":
                k -= 1
            if k >= 1 and src[k - 1:k + 1] == "::":
                name = "::" + name
            elif k >= 0 and src[k] == ".":
                name = "." + name
            last_ident = (last_ident + name) if name.startswith("::") and last_ident else name
            if brace == 0 and len(item_words) < 6:
                item_words.append(m.group(0))
            i = j
            prev_sig = "a"
            continue
        if c in "([{":
            macro = prev_sig == "!"
            top = {"callee": last_ident, "argno": 0, "macro": macro, "open": c, "pos": i}
            stack.append(top)
            if c == "{":
                brace += 1
            last_ident = ""
            i += 1
            prev_sig = c
            continue
        if c in ")]}":
            if stack:
                stack.pop()
            if c == "}":
                brace = max(0, brace - 1)
                if brace == 0:
                    item_words = []
            i += 1
            prev_sig = c
            last_ident = ""
            continue
        if c == ",":
            if stack:
                stack[-1]["argno"] += 1
            i += 1
            prev_sig = c
            continue
        if c == ";":
            last_ident = ""
            if brace == 0 and not stack:
                item_words = []
        if c == "!":
            prev_sig = "!"
            i += 1
            continue
        prev_sig = c
        i += 1
    return lits


def test_module_start(src):
    """Offset des ersten `#[cfg(test)]` am Zeilenanfang (alles danach ist Testcode), sonst Länge."""
    m = re.search(r"^#\[cfg\(test\)\]", src, re.M)
    return m.start() if m else len(src)
