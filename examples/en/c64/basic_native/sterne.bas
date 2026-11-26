; sterne.bas - Native C64 BASIC V2 without line numbers: labels instead of GOTO numbers
;
; Purpose: Shows the "native" source text: the lines are numbered at build time (10, 20, 30 ...), "@name" is replaced by the line number.
;   The result is an ordinary C64 BASIC V2 program
;   (see build/sterne.classic.txt and build/sterne.labels.txt for the listing and the label mapping).
;
; Run: Build with `vcc build examples/c64/basic_native`, load build/sterne.prg and start it with RUN. The program sets the border and
;   background to black, clears the screen and scatters 200 stars ("*") in random colors and at random places
;   (columns 1-38, rows 2-23; row 0 holds the heading). Then it writes "done" in the last screen row and stays in an
;   endless loop (the STOP key ends it). Random numbers come from RND(1); without its own seed the sequence starts the same after every
;   reset (a second RUN without a reset continues it). A star can overwrite an earlier one; so at most
;   200 stars appear.
;
; Hardware and memory addressed directly with POKE:
;   53280 ($D020) border color, 53281 ($D021) background color
;   1024 ($0400) screen memory, 40 characters per row: address = 1024 + row*40 + column (values are screen codes, 42 = "*")
;   55296 ($D800) color RAM with the same layout: address = 55296 + row*40 + column (value = color 0-15)
; Variables: x = column, y = row, n = counter of the stars placed (starts at 0 without an assignment).
; Note: `;` is a comment only at the start of a line (in the middle of a line it would be the append character in PRINT); explanations are
;   therefore on their own lines. `rem`, on the other hand, stays in the program and costs memory and time.
poke 53280,0 : poke 53281,0
print "{clr}{wht}stars for the c64"

@neu:
; Pick column 1..38 (int(rnd*38) gives 0..37) and row 2..23 (int(rnd*22) gives 0..21)
x = int(rnd(1)*38) + 1
y = int(rnd(1)*22) + 2
poke 1024 + y*40 + x, 42          : rem star (screen code 42)
; Color 1..15 (black, 0, is excluded so that the star stays visible on the black background)
poke 55296 + y*40 + x, int(rnd(1)*15) + 1
n = n + 1
if n < 200 then @neu

; Move the cursor to the top left and 24 rows down, i.e. to the last row (row 24 of the 25-row screen)
print "{home}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}done"
@ende:
; Endless loop so that "READY." does not overwrite the screen
goto @ende
