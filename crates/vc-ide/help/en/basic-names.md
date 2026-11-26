# BASIC: Long Variable Names

A **variable** is a named place in memory. On the C64, the VIC-20, the C128, the Plus/4 and the MEGA65, the BASIC tells variable names apart by their **first two characters** only. `punkte`, `punkt` and `pu` are **the same** variable there. This is one of the biggest traps of classic BASIC. With the line `#longnames` you write long, meaningful names, and the build turns them into matching short names. This page belongs to the [BASIC manual](help:basic).

## The problem without `#longnames`

Without help, you notice the mix-up only when the program calculates wrongly:

```basic liste
punkte = 10
punkt = 5
print punkte
```

```text
10 punkte = 10
20 punkt = 5
30 print punkte
```

On the C64, `punkte` and `punkt` are both `PU`, so the output would be 5, not 10. Without `#longnames` the translator does not report this, because for it this is normal BASIC.

A second trap is **keywords inside names**. The BASIC also recognizes commands in the middle of a word: `total` contains `TO`, `score` contains `OR`, `form` contains `FOR`. When you type the name, the machine splits it into commands and reports `?SYNTAX ERROR`. Names like `status`, `time` and `error` also cause trouble: `ST`, `TI` and `ER` are system variables.

## Turning on long names

Write this on a line of its own in the file:

```
#longnames
```

The line can be anywhere (the top is usual), upper or lower case does not matter. It applies to the **whole program**, also to included files (see [Directives](help:basic-directives)). It gets no line number and costs nothing.

After that, every name can consist of **letters, digits and `_`**, must begin with a letter and can be as long as you like:

```basic ausgabe
#longnames
punkte = 10
punkt = 5
print punkte; punkt
```

```text
punkte                     →  PU         (Number)
punkt                      →  PN         (Number)
```

The build has turned `punkte` into `PU` and `punkt` into `PN`: separate variables with names that the machine can tell apart. The finished program looks like this:

```basic liste ausgabe
#longnames
punkte = 10
punkt = 5
print punkte; punkt
```

```text
10 pu = 10
20 pn = 5
30 print pu; pn
```

The build has reported with a **warning** (W313) that the mix-up is ruled out. More about this further down.

### How the short name is made

The build assigns the short names one after the other, in the order in which the long names first appear in the program:

1. Names that you already wrote short (at most two characters, without `_`) **stay unchanged** and are taken.
2. A long name first gets its **first two characters** (without `_`). This gives what you would have written by hand in classic BASIC: `punkte` becomes `PU`, `hero_x` becomes `HE`.
3. If this short name is already taken, the build uses another one with the same first letter and a later letter of the name (`punkt` becomes `PN`), then any one with the same first letter, and last of all any free one.
4. The build **never** assigns short names that form or contain a keyword (`TO`, `OR`, `IF`, `ON`, `GO`, `FN` and all others that begin with a command) or that the system reserves (`ST`, `TI`, `DS`, `ER`, `EL`). So `total` does not become `TO` but `TT`, and `status` does not become `ST` but `SA`.

```basic ausgabe
#longnames
score = 1 : total = 2 : form = 3 : status = 4 : tim = 5
print score; total; form; status; tim
```

```text
score                      →  SC         (Number)
total                      →  TT         (Number)
form                       →  FO         (Number)
status                     →  SA         (Number)
tim                        →  TM         (Number)
```

The result is, byte for byte, the program that you would have written by hand with the short names. There are no run-time costs and no table in the memory of the machine.

### Words are always whole names

With `#longnames`, a word is **either exactly one keyword or a whole name**. Classic BASIC, in contrast, also recognizes commands when they stick to names or numbers. This has a consequence: **keywords need separators** (spaces or punctuation):

```basic liste
#longnames
for i = 1 to 3
next i
```

```text
10 for i = 1 to 3
20 next i
```

Written stuck together, as you know it from old listings (`FORI=1TO3`), `fori` would be a name. The build then makes `fo = 1 to 3` of it and the `FOR` is missing:

```basic liste
#longnames
fori = 1 to 3
```

```text
10 fo = 1 to 3
```

So write `for i = 1 to 3`, `goto 100`, `if a then`. Every space takes one byte in the program, and the translator does not remove any. If you need every byte, leave out `#longnames` and stick the words together by hand.

## Types: `name`, `name%`, `name$`, `name(`

Number, integer, text and array are different variables in BASIC, even if the name is the same. This stays so: they share the short name but stay separate.

```basic ausgabe
#longnames
dim feld(3)
zaehler% = 0
name$ = "hero"
name = 2
feld(1) = 5
print name$; name; zaehler%; feld(1)
```

```text
feld(…)                    →  FE(…)      (Number, array)
zaehler%                   →  ZA%        (Integer)
name$                      →  NA$        (Text)
name                       →  NA         (Number)
```

On the machine, `name` and `name$` are called `NA` and `NA$`: two variables. An array `feld(…)` is again a variable of its own, so `feld` (without parenthesis) is a different one from `feld(…)`.

## Functions: `def fn`

Names of functions (`def fn name(x) = ...`) have a **namespace of their own**. Their short names do not compete with those of the variables.

```basic liste
#longnames
def fn quad(wert) = wert * wert
print fn quad(7)
```

```text
10 def fn qu(we) = we * we
20 print fn qu(7)
```

A variable name that begins with `fn` (`fnx`) is read as a function call, as BASIC itself does. Do not give variables such names.

## Seeing the assignment

Where the build converts names, it lists the assignment:

- in the **build log** (line "Long names: n converted" with the table below it),
- in the file **`build/<name>.vars.txt`**,
- with `vcc basic` on the command line (section `Long names → short names:`),
- as `names` in the line map of [basic2asm](help:basic-tools),
- in the **editor**: When you move the mouse over a long name, a hint shows its short name.

A row of the table has the form `long  →  SHORT  (kind)`; the kind is `Number`, `Integer`, `Text`, `Function` or `Number, array`. Names that you wrote short yourself are not in the table. Even if a long name is the beginning of a short one, the short one stays unchanged and the long one gets another name:

```basic ausgabe
#longnames
pu = 1
punkte = 2
print pu; punkte
```

```text
punkte                     →  PN         (Number)
```

## Collisions and warnings

If two long names differ **only after the first two characters**, they get separate short names, and the build warns with **W313**. The warning helps you not to fall into a trap on a BASIC without this conversion:

```basic warnung
#longnames
punkte = 1
punkt = 2
print punkte; punkt
```

```text
main.bas:3: warning W313: 'punkte', 'punkt' differ only after the first 2 characters ('PU'); they are kept apart here (PU, PN). A BASIC without this conversion would see only one variable
```

The warning is a hint, not an error. The program builds and runs correctly. If you want to get rid of it, give the names different beginnings (`punktzahl`, `rest`).

There is an error only when all short names are taken (**B131**, `no free short name is left for the name 'x' (too many variables)`). This happens only with hundreds of different variables, which the machine cannot hold anyway.

Characters other than letters, digits and `_` are not allowed in names; a name does not begin with a digit.

## What stays untouched

The build converts names only where they are names. These are **not changed**: strings (`"total"`), `REM` lines, `DATA` lines, `{...}` control characters, numbers (also `$ff` and `%101` on systems that know them) and marks (`@name`).

```basic liste
#longnames
total = 3
print "total:"; total
rem total ist die summe
data total, 5
```

```text
10 tt = 3
20 print "total:"; tt
30 rem total ist die summe
40 data total, 5
```

## Atari BASIC and Turbo BASIC XL

Atari BASIC already knows long names itself. There, all names stay **unchanged**. `#longnames` does only two more things:

- `_` is not allowed in Atari names and is **dropped**: `hero_x` becomes `herox`.
- If two names collide because of this, the build appends a number: `hero_x` and `herox` give `herox2` and `herox`.
- Names that are the same as a command or a function are avoided.

Without `#longnames`, `_` in an Atari name is an error:

```basic fehler system=atari8
hero_x = 1
? hero_x
```

```text
main.bas:1: error B180: '=' expected after 'HERO'
main.bas:2: error B100: unexpected character '_'
```

With `#longnames` it works:

```basic liste system=atari8
#longnames
hero_x = 1
herox = 2
? hero_x; herox
```

```text
10 HEROX2=1
20 HEROX=2
30 ? HEROX2;HEROX
```

The limits: at most 128 different variables in Atari BASIC, 256 in Turbo BASIC XL (error `more than 128 variables`).

## Typical errors at a glance

| Symptom | Cause | Remedy |
|---|---|---|
| Two variables behave like one | Names differ only after character 2, `#longnames` is missing | Write `#longnames` at the top |
| `?SYNTAX ERROR` with a harmless name | The name contains a keyword (`total`, `score`) | Turn on `#longnames` |
| `FOR` or `GOTO` is missing in the result | A keyword sticks to a name or number (`fori`, `goto100`) | Put in spaces |
| W313 when building | Two long names have the same beginning | Read the hint; rename only if you want to |
| B131 when building | No free short names left | Save variables, use arrays instead of single variables |
| Hover shows no short name | `#longnames` is missing, or the name is already short | Check the `#longnames` line |

*This feature is dedicated to Birko from N3rdroom.*
