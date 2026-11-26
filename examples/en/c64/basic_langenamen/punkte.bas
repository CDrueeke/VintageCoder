#longnames
; punkte.bas - C64 BASIC V2 with long variable names (#longnames)
;
; Purpose: The C64 BASIC tells variables apart only by their first two characters: `punkte` and `punkt` would be the same variable (PU), and
;   names like `total` (contains TO) or `score` (contains OR) are split into keywords by the interpreter. The line `#longnames` (here the
;   first line; it may appear anywhere and gets no line number) allows long, descriptive names instead: the build maps each one to
;   a unique name pair (here e.g. punkte -> PU, runde -> RU, zaehler% -> ZA%, name$ -> NA$, feld() -> FE(), fn quad -> FN QU).
;   The complete mapping is in the build log and in build/punkte.vars.txt, the classic listing in build/punkte.classic.txt.
;
; Flow: Build with `vcc build examples/c64/basic_langenamen`, load build/punkte.prg and start it with RUN. The program clears the
;   screen (white text), computes the square numbers 1, 4 and 9 in a loop and adds them up in `punkte`
;   (14 per pass). It prints three lines of the form "hero round 1 points 14" (round 2: 28, round 3: 42) and then stays in an
;   endless loop (the STOP key ends it).
;
; Language features the example shows:
;   - `@neu:` and `@ende:` are labels; `then @neu` and `goto @ende` jump there, the build assigns the line numbers
;     (10, 20, 30 ...). The labels do not count as variables.
;   - Comment lines with `;` at the start of the line (like this one) are removed by the build and cost nothing in the program. A `;` in the middle of
;     a line is not a comment (in PRINT it is the append character), so the explanations are on their own lines.
;   - Control characters inside quotes are written in curly braces: {clr} = clear screen, {wht} = text color white.
;   - Names with type characters: `zaehler%` is an integer variable, `name$` a string; `feld(...)` an array and `fn quad` a
;     function defined with `def fn`. Keywords need spaces around them so that they do not run into names.
print "{clr}{wht}long names on the c64"
; Array with the indexes 0..3; 1..3 are used (one square number each)
dim feld(3)
; Function "square"; `wert` is only the placeholder for the argument
def fn quad(wert) = wert * wert
; zaehler% counts the printed lines, name$ is printed before every line; punkte starts at 0 without an assignment
zaehler% = 0 : name$ = "hero"

@neu:
; inner loop: squares 1, 4, 9; the sum is never reset and so grows over the passes to 14, 28, 42
for runde = 1 to 3
  feld(runde) = fn quad(runde)
  punkte = punkte + feld(runde)
next runde
zaehler% = zaehler% + 1
print name$; " round"; zaehler%; " points"; punkte
; repeat three times
if zaehler% < 3 then @neu
@ende:
; endless loop, so that "READY." does not overwrite the screen
goto @ende
