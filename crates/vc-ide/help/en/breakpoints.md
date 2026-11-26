# Breakpoints

The **Breakpoints** list on the left in the **Debug** section shows all points of the session with type (Execute, Read, Write, Access), address or range, condition (`if ...`), hits to ignore (`xN`) and hit counter. A ✕ removes a point. Breakpoints from the editor appear pale with the mark **Editor** and `file:line` next to them.

## Execution
Click in the margin next to the line number in the source, or put the cursor in the line and press **F9** (menu **Edit > Toggle breakpoint**). In the disassembly, a click on the round button in front of a line sets a point. Breakpoints in the editor apply even before you run; VintageCoder maps them to an address using the line mapping of the last build. **Debug: Run to here** (menu **Project > Debug**) sets a one-time point on the line at the cursor and continues running.

## Condition, ignore and data breakpoints
Below the list, a form creates points:

- **Kind of point:** **Execute**, **Read**, **Write** or **Access**. The last three are data breakpoints (only with VICE) and cannot be chosen otherwise.
- **Address:** `$D020`, `0x0400` or `2049` (no symbol name).
- **End (for a range):** only for data breakpoints; empty means a single address.
- **Condition:** an expression, for example `a == 5`; the point stops only if it is true.
- **Ignore:** skips the first N hits.
- **Add** creates the point. An error in the condition appears as a red message below the list.

## Expressions
The same expression language applies to **Watch**, to conditions and in the tests:
```
a == $10
(x & 15) == 5 && fc
[$d020] != 3
peekw($fffe) == $ea31
lin > 100
```
Available: registers `a x y sp pc p`, flags `fn fv fb fd fi fz fc`, `lin`/`cyc` (raster position), memory `[adr]` / `peek()` / `peekw()`, symbols of the last build. Operators as in C - comparisons bind more strongly than `&` and `|`, so use parentheses if in doubt. A syntax error is reported immediately; a runtime error stops the program.

Note: the app evaluates conditions after every stop and continues running if they are not true - at places that are hit very often, this takes time. atari800 supports only one execution breakpoint.
