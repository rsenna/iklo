---
**Document:** Berkeley Logo (UCBLogo) Reference
**Purpose:** Authoritative documentation of UCBLogo semantics — the stable foundation that LogoScript builds on and extends. Not a LogoScript spec; read this to understand the baseline, then see AGENTS.md for deviations.
**Audience:** Language semanticists, logo interpreters, AI agents reasoning about Logo behavior.
**Last Updated:** 2026-07-01 (stable)
**Sources:**
  - Brian Harvey, "Computer Science Logo Style", 2nd ed., Vols. 1–3, MIT Press, 1997.
  - Berkeley Logo Reference Manual (appendix to Vol. 2, pp. 267–309).
  - Berkeley Logo source: https://people.eecs.berkeley.edu/~bh/logo.html
**Cross-refs:**
  - [AGENTS.md](../../AGENTS.md) — LogoScript deviations and extensions
  - [spec/backlog.md](../../spec/backlog.md) — implementation status of LogoScript features (and the per-epic specs under [spec/](../../spec/))
**Important:** Examples here use UCBLogo syntax. LogoScript syntax differs significantly: see AGENTS.md introduction for key differences.
---

# Berkeley Logo (UCBLogo) Reference

This file is the **stable reference for UCBLogo semantics**. It is *not* a LogoScript spec, and LogoScript does **not** inherit from it: UCBLogo is a **comparison baseline and a source of inspiration**, not a compatibility target ([ADR-0008](../../design/decisions/ADR-0008-ucblogo-is-inspiration-not-a-compatibility-target.md)). Read it to understand what UCBLogo does; consult `AGENTS.md` for what LogoScript does.

> **Note on code examples**: All examples here use **UCBLogo syntax**. LogoScript deviates in several places — most notably: procedure bodies use `is ... end` instead of just `end`; conditionals use `if <cond> then ... else ... end` instead of `if <cond> [...]`; word literals use `'word` (single quote) instead of `"word`; comments use `#` (`;` is deprecated; `//` and `/* */` were removed by Grammar 2.0 §13b); scope is lexical by default; and there is no `catch`/`throw` — errors are values ([ADR-0009](../../design/decisions/ADR-0009-errors-are-values-no-exceptions.md)). Read the examples for their semantic content; do not copy UCBLogo syntax directly into LogoScript.

*Sources:*
- *Brian Harvey, "Computer Science Logo Style", 2nd ed., Vols. 1–3, MIT Press, 1997.*
- *Berkeley Logo Reference Manual (appendix to Vol. 2, pp. 267–309).*
- *Berkeley Logo source: https://people.eecs.berkeley.edu/~bh/logo.html*

---

## Tokenization

Names of procedures, variables, and property lists are **case-insensitive**. The special words `true` and `false` are also case-insensitive. Case of letters is preserved in everything you type.

Within square brackets, words are delimited only by spaces and square brackets. `[2+3]` is a list containing one word.

After a quotation mark outside square brackets, a word is delimited by a space, a square bracket, or a parenthesis.

A word not after a quotation mark or inside square brackets is delimited by a space, a bracket, a parenthesis, or an infix operator `+ - * / = < >`. Words following colons (`:name`) are in this category — quote and colon are not delimiters.

### Line Continuation

A line can be continued to the next line if its last character is a **tilde** (`~`). `readword` preserves the tilde and the newline; `readlist` does not.

Lines with unmatched brackets, parentheses, or vertical bars are automatically continued to the next line (but not if the continuation line contains only `end`).

### Comments

A **semicolon** (`;`) begins a comment in an instruction line. Logo ignores everything from the semicolon to the end of the line. A tilde as the last character still indicates a continuation line, but not a continuation of the comment.

### Backslash and Vertical Bars

To include otherwise-delimiting characters (including semicolons or tildes) in a word, precede them with **backslash** (`\`). If the last character of a line is a backslash, the newline character becomes part of the last word, and the line continues onto the following line. To include a literal backslash, use `\\`.

An alternative notation is to enclose a group of characters in **vertical bars** (`|...|`). All characters between vertical bars are treated as if they were letters. Within vertical bars, only backslash and vertical bar themselves must be backslashed.

Characters entered between vertical bars are "forever special" — even if the word is later reparsed with `parse` or `runparse`. Characters typed after a backslash lose their special quality when the quoted word is runparsed.

### `?` as Procedure Call

A word consisting of a question mark followed by a number (e.g. `?37`), when runparsed where a procedure name is expected, is treated as `( ? 37 )` — making the number an input to the `?` procedure. This is the mechanism behind template slot variables.

---

## Evaluation Model

Logo uses **prefix notation** for procedure calls (except infix arithmetic, which is also supported). Everything is a procedure call.

The **plumbing diagram** metaphor: each operation "outputs" a value that flows as input to the enclosing expression. Logo reads left-to-right, consuming exactly the number of inputs each procedure requires.

```logo
print sum 2 3          ; prints 5
print sentence "Hello, [world]
```

Parentheses can override normal input count:
```logo
(print "a "b "c)       ; print with 3 inputs
(sum 1 2 3 4)           ; sum with 4 inputs
```

---

## Data Types

| Type | Description | Examples |
|------|-------------|---------|
| **Word** | Atomic token, including numbers | `"hello`, `42`, `3.14`, `true` |
| **List** | Ordered sequence of words/lists | `[a b c]`, `[[1 2] [3 4]]` |
| **Array** | Mutable, constant-time indexed | `{a b c}` (1-indexed by default) |
| **Number** | A word that happens to be numeric | `42`, `-3.5` |

A **datum** is a word or list (not an array). A **sentence** (or flat list) has only words as members.

### Quoting
- `"word` — quotes a word (prevents evaluation)
- `[list contents]` — quotes a list literally; brackets are never evaluated
- `:name` — shorthand for `thing "name` (dereferences a variable)
- Numbers are self-quoting

---

## Procedures

### Defining Procedures

```logo
to square :side
  repeat 4 [forward :side right 90]
end

to greet :person
  print sentence "Hello, :person
end
```

- **Commands**: produce effects, no output value (like `print`, `forward`)
- **Operations**: compute and output a value (like `sum`, `first`, `sentence`)
- The `to` line lists the procedure name and all formal parameters (`:name`)
- `stop` exits a command; `output <expr>` exits an operation with a value
- `to` is a special form: it takes its inputs as the actual words typed, not as evaluated results

### Variable Number of Inputs

Every procedure has a *minimum*, *default*, and *maximum* number of inputs.

- **Required inputs** come first: `:inputname`
- **Optional inputs** follow, with default values: `[:inputname default.value.expression]`
- A single **rest input** may come last: `[:inputname]` (no default = rest parameter)

```logo
to proc :in1 [:in2 "foo] [:in3]
```
- `proc "x` → in1=x, in2=foo, in3=[]
- `( proc "a "b "c "d )` → in1=a, in2=b, in3=[c d]

The *maximum* is infinite if a rest input is given; otherwise it equals required + optional. The *default* number equals the minimum unless overridden by putting the desired default number as the last thing on the `to` line:

```logo
to proc :in1 [:in2 "foo] [:in3] 3
```
This procedure has a minimum of 1, a default of 3, and an infinite maximum.

When a procedure is invoked without parentheses, it consumes exactly *default* inputs. With parentheses, any number from *minimum* to *maximum* is accepted.

### `define` and `text`

```logo
define "procname text        ; create procedure from list-of-lists
text "procname               ; output the text of a procedure (for define)
fulltext "procname            ; output with formatting preserved
copydef "newname "oldname     ; copy a procedure definition
```

### Calling with Extra Inputs

Use parentheses to supply non-default input counts:
```logo
(print "a "b "c)
(sentence "a "b "c "d)
```

---

## Variables

```logo
make "x 5                  ; global variable (top-level)
make "x :x + 1             ; update

to myprocedure :input
  local "temp
  make "temp :input * 2
  print :temp
end
```

- **Global variables**: created by `make` at top level, or `make` inside a procedure if not locally declared
- **Local variables**: procedure inputs (`:name`) + explicitly declared with `local "name`
- **Dynamic scope** *(UCBLogo)*: subprocedures can read/write variables of their calling procedures — this is intentional in Berkeley Logo. **LogoScript uses lexical scope by default**, so this pattern does not apply unless dynamic scope is explicitly opted into.
- `:name` is syntactic sugar for `thing "name` — the colon is an abbreviation for `thing`, not for the combination of colon+name
- `local [var1 var2]` declares multiple locals at once
- `localmake "var value` = `local "var` + `make "var value` (library procedure)
- `name value varname` = `make` with arguments reversed (library procedure)

### Dynamic Scope Example

```logo
; UCBLogo dynamic scope — works in UCBLogo, NOT by default in LogoScript
to water :pitcher
  output item :pitcher :state    ; accesses :state from calling procedure
end
```

---

## Control Flow

### Conditionals

```logo
if :x > 0 [print "positive]

ifelse :x > 0
  [print "positive]
  [print "non-positive]

; if with 3 inputs acts like ifelse (with a warning)
(if :x > 0 [print "yes] [print "no])

; test/iftrue/iffalse — share a test across two branches
test emptyp :list
iftrue  [print "empty]          ; also: ift
iffalse [print first :list]     ; also: iff
```

The effect of `test` is local to the procedure in which it is used; any corresponding `iftrue` or `iffalse` must be in the same procedure or a subprocedure.

### Loops

```logo
repeat 10 [forward 50 right 36]

for [i 1 10] [print :i]
for [i 1 10 2] [print :i]        ; step size 2
for [i 10 1 -1] [print :i]       ; count down

foreach [a b c d] [print ?]       ; ? = current member
foreach [a b c d] "print           ; same, using procedure name

forever [...]                      ; infinite loop; use stop/throw to exit

; Library loop procedures
while [:x > 0] [make "x :x - 1]          ; test first, may never run
do.while [make "x :x - 1] [:x > 0]       ; run at least once
until [:x = 0] [make "x :x - 1]          ; test first, may never run
do.until [make "x :x - 1] [:x = 0]       ; run at least once
```

The `for` control list `[var start end step]` is evaluated by `run`. Start, end, and step can be expressions. If step is omitted, it defaults to 1 or -1 depending on whether limit > start or limit < start. An explicit step can lead to a zero-trip for (e.g. `for [i 1 0 1] [...]` runs zero times).

### Stop and Output

```logo
stop                          ; exits a command (no value)
output <value>                ; exits an operation with a value (also: op)
.maybeoutput <expr>           ; like output, but if expr produces no value, acts like stop
```

`.maybeoutput` is intended for control-structure definitions where you don't know whether the body expression will output a value.

### `run` and `runresult`

```logo
run [forward 50 right 90]     ; evaluate an instruction list
run :some.list                 ; dynamic dispatch

; runresult runs the instructions; outputs [] if no value, or a 1-element list with the value
local "result
make "result runresult [something]
if emptyp :result [stop]
output first :result
```

`runresult` is useful for inventing command-or-operation control structures.

---

## Nonlocal Exit: `catch` and `throw`

```logo
catch "tag [instructionlist]
throw "tag
(throw "tag value)
```

`catch` runs its second input. If, during execution, a `throw` is executed with a tag equal to the first input (case-insensitive comparison), the instructionlist is terminated immediately. If `throw` is used with two inputs, the second provides an output value for the `catch`.

### Special Tags

- `throw "toplevel` — terminates all running procedures and returns to the top-level prompt. Typing the system interrupt character (ctrl-C on Unix) has the same effect.
- `throw "error` — generates an error condition. If a second input is given, it becomes the error message text. The corresponding `catch "error` does not output.
- `throw "system` — immediately exits Logo.

### Error Handling

```logo
; Auto-pause on error:
make "erract [pause]

; Catch errors programmatically:
catch "error [do.something.risky]
show error    ; outputs a list: [code message procedure line]
```

If the variable `erract` exists, its value is run as an instructionlist when an error occurs (before the error message is printed). If it invokes `pause`, the error message is printed before the pause. Certain errors are *recoverable*: if the erract instructionlist outputs a value, that value is used in place of the expression that caused the error.

### Error Codes

| Code | Message | Notes |
|------|---------|-------|
| 0 | Fatal internal error | can't be caught |
| 1 | Out of memory | |
| 2 | Stack overflow | |
| 3 | Turtle out of bounds | |
| 4 | *proc* doesn't like *datum* as input | not recoverable |
| 5 | *proc* didn't output to *proc* | |
| 6 | Not enough inputs to *proc* | |
| 7 | *proc* doesn't like *datum* as input | recoverable |
| 8 | Too much inside ()'s | |
| 9 | You don't say what to do with *datum* | |
| 10 | ')' not found | |
| 11 | *var* has no value | |
| 12 | Unexpected ')' | |
| 13 | I don't know how to *proc* | recoverable |
| 14 | Can't find catch tag for *throwtag* | |
| 15 | *proc* is already defined | |
| 16 | Stopped | |
| 17 | Already dribbling | |
| 18 | File system error | |
| 19 | Assuming you mean IFELSE, not IF | warning only |
| 20 | *var* shadowed by local in procedure call | warning only |
| 21 | Throw "Error | |
| 22 | *proc* is a primitive | |
| 23 | Can't use TO inside a procedure | |
| 24 | I don't know how to *proc* | not recoverable |
| 25 | IFTRUE/IFFALSE without TEST | |
| 29 | Macro returned *value* instead of a list | |
| 31 | Can only use STOP or OUTPUT inside a procedure | |
| 32 | APPLY doesn't like *badthing* as input | |
| 34 | Really out of memory | can't be caught |

---

## Recursion

### The Three Laws

1. **Stop rule** comes before the recursive call
2. **Recursive call** advances toward the base case
3. **Combining step** assembles partial results (for operations)

### Command Recursion (combining method)

```logo
to downup :word
  print :word
  if equalp count :word 1 [stop]
  downup butlast :word
  print :word
end
```

### Operation Recursion

```logo
to length :list
  if emptyp :list [output 0]
  output 1 + length butfirst :list
end
```

### Tail Recursion

The recursive call is the **last thing** — no work after it:
```logo
; command tail recursion — stack space O(1) in Berkeley Logo
to count.down :n
  if :n = 0 [stop]
  print :n
  count.down :n - 1
end

; operation tail recursion — must be direct input to output
to fact.helper :n :acc
  if :n = 0 [output :acc]
  output fact.helper :n-1 :n*:acc
end
```

---

## Template-Based Iteration

Templates are instruction lists or expression lists with **slots** for the iteration tools to fill with varying data. Three forms of template are supported:

### 1. Explicit-Slot (Question-Mark) Form

The most common form. `?` is replaced by the current datum. `?1`, `?2` for parallel data. `?rest` for the portion of the data input to the right of the current element. `#` for the 1-based position index.

```logo
show map [? * ?] [2 3 4 5]              ; [4 9 16 25]
show (map [word ?1 ?2] [a b c] [d e f]) ; [ad be cf]
```

### 2. Named-Procedure Form

If the template is a word rather than a list, it is taken as a procedure name:

```logo
show map "first [apple banana cherry]   ; [a b c]
```

The procedure must accept a number of inputs equal to the number of parallel data slots.

### 3. Named-Slot (Lambda) Form

A template list whose first member is itself a list. The first member is a list of names; local variables are created with those names and given the available data in order:

```logo
to matmul :m1 :m2 [:tm2 transpose :m2]
  output map [[row] map [[col] dotprod :row :col :tm2] :m1
end
```

### Iteration Primitives

**`apply`** `template inputlist` — runs the template, filling its slots with the members of inputlist.

**`invoke`** `template input ...` *(library procedure)* — like `apply` but inputs are separate expressions, not a list.

**`foreach`** `data template` — evaluates the template once for each member of the data list. `?rest` represents the portion of the data list to the right of `?`. `#` represents the 1-based position. With multiple data lists: `(foreach data1 data2 ... template)` — all must be the same length.

**`map`** `template data` — outputs a word or list (same type as data) by evaluating the template for each member. With multiple data lists: `(map template data1 data2 ...)`.

**`map.se`** `template data` — like `map`, but concatenates results using `sentence`. The output list may therefore be a different length from the input.

**`filter`** `tftemplate data` — outputs a word or list containing only the members for which the template outputs `true`.

**`find`** `tftemplate data` — outputs the first constituent for which the template outputs `true`, or the empty list if none.

**`reduce`** `template data` — left-folds the data using a two-slot template. If the data has only one member, outputs that member. Otherwise, the template is first applied with `?1` = next-to-last and `?2` = last, then working leftward.

**`crossmap`** `template listlist` — like `map`, but takes all possible *combinations* of members from the data inputs rather than parallel members. Data inputs need not be the same length.

**`cascade`** `endtest template startvalue` — repeatedly applies a one-slot template, starting from startvalue, until endtest is satisfied. `#` counts from 1. Multi-variable form: `(cascade endtest tmpl1 sv1 tmpl2 sv2 ... finaltemplate)`.

**`cascade.2`** — `cascade` with a default of five inputs instead of three.

**`transfer`** `endtest template inbasket` — evaluates the template once for each member of the inbasket list. Maintains an *outbasket* (initially empty); after each evaluation, the result becomes the new outbasket. `?in` = current inbasket member; `?out` = current outbasket.

---

## Data Structure Primitives

### Constructors

| Primitive | Signature | Description |
|-----------|-----------|-------------|
| `word` | `word1 word2` | Concatenates words. Variadic: `(word w1 w2 w3 ...)` |
| `list` | `thing1 thing2` | Outputs a list of its inputs (not flattened). Variadic. |
| `sentence` / `se` | `thing1 thing2` | Outputs a flat list. Flattens list inputs. Variadic. |
| `fput` | `thing list` | Prepends thing to list |
| `lput` | `thing list` | Appends thing to list |
| `array` | `size` | Outputs an array of empty members. `(array size origin)` to set start index. |
| `mdarray` | `sizelist` | Multi-dimensional array. `(mdarray sizelist origin)` |
| `listtoarray` | `list` | Converts list to array. `(listtoarray list origin)` |
| `arraytolist` | `array` | Converts array to list |
| `combine` | `thing1 thing2` | If thing2 is a word → `word`; if thing2 is a list → `fput`. *(library)* |
| `reverse` | `list` | Reverses a list. *(library)* |
| `gensym` | *(no inputs)* | Outputs a unique word (G1, G2, etc.). *(library)* |

### Selectors

| Primitive | Signature | Description |
|-----------|-----------|-------------|
| `first` | `thing` | First character (word), first member (list), origin index (array) |
| `last` | `wordorlist` | Last character (word) or last member (list) |
| `butfirst` / `bf` | `wordorlist` | All but first |
| `butlast` / `bl` | `wordorlist` | All but last |
| `firsts` | `list` | `first` of each member of the list |
| `butfirsts` / `bfs` | `list` | `butfirst` of each member of the list |
| `item` | `index thing` | The *index*-th element (1-based for words and lists) |
| `mditem` | `indexlist array` | Multi-dimensional array access |
| `pick` | `list` | Random member. *(library)* |
| `remove` | `thing list` | Copy of list with all members `equalp` to thing removed. *(library)* |
| `remdup` | `list` | Copy of list with duplicate members removed (rightmost kept). *(library)* |
| `quoted` | `thing` | If a list, outputs as-is; if a word, outputs with `"` prepended. *(library)* |

### Mutators

| Primitive | Signature | Description |
|-----------|-----------|-------------|
| `setitem` | `index array value` | Replace the *index*-th member of array. Checks for circularity. |
| `mdsetitem` | `indexlist array value` | Multi-dimensional version of setitem. *(library)* |
| `.setfirst` | `list value` | **Dangerous.** Changes first member of list in place. |
| `.setbf` | `list value` | **Dangerous.** Changes butfirst of list in place. |
| `.setitem` | `index array value` | Like `setitem` but without circularity check. **Dangerous.** |

> **Warning**: Primitives whose names start with a period are dangerous. `.setfirst` and `.setbf` can create circular list structures, cause infinite loops, and crash Logo.

### Predicates

| Primitive | Alias | Description |
|-----------|-------|-------------|
| `wordp` | `word?` | true if input is a word |
| `listp` | `list?` | true if input is a list |
| `arrayp` | `array?` | true if input is an array |
| `emptyp` | `empty?` | true if input is empty word or empty list |
| `equalp` | `equal?` | true if inputs are equal. Also available as infix `=`. Two numbers are equal if they have the same numeric value. Case-sensitivity controlled by `caseignoredp`. Arrays are only equal to themselves. |
| `notequalp` | `notequal?` | true if inputs are not equal. Also: `<>` |
| `beforep` | `before?` | true if word1 comes before word2 in ASCII collating sequence |
| `.eq` | | true if inputs are the **same datum** (identity, not equality). **Dangerous.** |
| `memberp` | `member?` | true if thing1 is a member of list/array, or a character of word |
| `substringp` | `substring?` | true if word1 is a substring of word2, or thing1 is `equalp` to a member of list/array |
| `numberp` | `number?` | true if input is a number |
| `backslashedp` | `backslashed?` | true if input character was entered with backslash or vertical bars |

### Queries

| Primitive | Description |
|-----------|-------------|
| `count` | Number of characters (word), members (list), or last-index (array) |
| `ascii` | ASCII code of first character. Interprets control characters as representing backslashed punctuation. |
| `rawascii` | ASCII code, treating control characters as themselves |
| `char` | Character with the given ASCII code (0–255) |
| `member` | If thing2 is a word/list: outputs the portion from the first instance of thing1 to the end. Outputs empty word/list if not found. |
| `lowercase` | All uppercase → lowercase |
| `uppercase` | All lowercase → uppercase |
| `standout` | Word that prints in standout mode (boldface/reverse video) |
| `parse` | Outputs the list that `readlist` would produce from the word |
| `runparse` | Outputs the list that would result from reparsing the word as an instruction line |

---

## Stack and Queue Operations *(library procedures)*

```logo
push "stackname thing      ; prepend to the list variable named stackname
pop "stackname              ; remove and output the most recently pushed member
queue "queuename thing      ; append to the list variable named queuename
dequeue "queuename          ; remove and output the least recently queued member
```

---

## Communication

### Transmitters

| Primitive | Signature | Description |
|-----------|-----------|-------------|
| `print` / `pr` | `thing` | Prints to the current write stream, newline at end. Lists printed without brackets. Variadic: `(print thing1 thing2 ...)` — separated by spaces. |
| `type` | `thing` | Like `print` but no newline and multiple inputs are not separated by spaces. Line-buffered. |
| `show` | `thing` | Like `print` but lists are printed *with* brackets. Variadic. |

**`printdepthlimit`**: if this variable exists with a nonneg integer value, complex structures are printed only to that depth. Members past the limit show as `...`.

**`printwidthlimit`**: if this variable exists with a nonneg integer value, only that many members of a list/array are printed. A single `...` replaces all missing data.

### Receivers

| Primitive | Alias | Description |
|-----------|-------|-------------|
| `readlist` | `rl` | Reads a line from the read stream, outputs as a list. At EOF, outputs the empty word. Processes backslash, vertical bar, tilde. Does not treat `;` as comment. |
| `readword` | `rw` | Reads a line, outputs as a single word (including spaces). At EOF, outputs the empty list. Preserves tildes and newlines from line continuation. |
| `readchar` | `rc` | Reads a single character. Turns off echoing until next `readlist`/`readword` or Logo prompt. |
| `readchars` | `rcs` | `readchars num` — reads *num* characters as a word. |
| `shell` | | Unix only. `shell "command` outputs the result of running a shell command as a list of lines. `(shell "command "true)` outputs each line as a word instead. |

### Terminal Access

| Primitive | Alias | Description |
|-----------|-------|-------------|
| `keyp` | `key?` | true if characters are waiting in the read stream |
| `cleartext` | `ct` | Clears the text screen |
| `setcursor` | | `setcursor [x y]` — moves the screen cursor (origin is upper-left, positive direction is southeast) |
| `cursor` | | Outputs `[x y]` of current cursor position |
| `setmargins` | | `setmargins [x y]` — shifts all further printing by x columns and y rows |

### File Access

| Primitive | Description |
|-----------|-------------|
| `openread "filename` | Open file for reading (read position at beginning) |
| `openwrite "filename` | Open file for writing (creates new / truncates existing) |
| `openappend "filename` | Open file for writing (appends to existing) |
| `openupdate "filename` | Open file for reading and writing (at end of existing file) |
| `close "filename` | Close the named file |
| `allopen` | Outputs list of names of all open files |
| `closeall` | Closes all open files. *(library)* |
| `erasefile "filename` / `erf` | Deletes the named file (must not be open) |
| `dribble "filename` | Records everything read from keyboard or written to terminal to the named file |
| `nodribble` | Stops dribble recording |
| `setread "filename` | Makes the named file the current read stream. `setread []` returns to terminal. |
| `setwrite "filename` | Makes the named file the current write stream. `setwrite []` returns to terminal. |
| `reader` | Outputs the name of the current read stream file (or `[]` if terminal) |
| `writer` | Outputs the name of the current write stream file (or `[]` if terminal) |
| `setreadpos charpos` | Seek within the read stream file (0-based) |
| `setwritepos charpos` | Seek within the write stream file (0-based) |
| `readpos` | Outputs current read position |
| `writepos` | Outputs current write position |
| `eofp` / `eof?` | true if no more characters to read from the read stream file |

Each open file has a single position used for both reading and writing. If a file opened for update is both `reader` and `writer`, `setreadpos` and `setwritepos` affect each other.

---

## Arithmetic

### Numeric Operations

| Primitive | Infix | Description |
|-----------|-------|-------------|
| `sum` | `+` | Sum of inputs. Variadic: `(sum 1 2 3 4)` |
| `difference` | `-` | Difference of two inputs. |
| `minus` | unary `-` | Negation. `-` means unary minus when preceded by something expecting an input, or preceded by a space and followed by a nonspace. `minus 3+4` = `-(3+4)` but `- 3+4` = `(-3)+4`. |
| `product` | `*` | Product of inputs. Variadic. |
| `quotient` | `/` | Division. Integer result iff both inputs are integers and divisor divides evenly. `quotient 5 2` = 2.5. `(quotient num)` = `1/num` (reciprocal). |
| `remainder` | | Remainder. Both must be integers. Result has same sign as *dividend* (num1). |
| `modulo` | | Remainder. Both must be integers. Result has same sign as *divisor* (num2). |
| `int` | | Truncates toward zero. Always outputs integer format. |
| `round` | | Rounds to nearest integer. |
| `sqrt` | | Square root. Input must be nonneg. |
| `power` | | `power base exp`. If base is negative, exp must be an integer. |
| `exp` | | e^n |
| `log10` | | Common (base-10) logarithm |
| `ln` | | Natural logarithm |

### Trigonometry

| Primitive | Description |
|-----------|-------------|
| `sin` *degrees* | Sine (input in degrees) |
| `cos` *degrees* | Cosine (input in degrees) |
| `arctan` *num* | Arctangent in degrees. `(arctan y x)` for two-argument form. |
| `radsin` *radians* | Sine (input in radians) |
| `radcos` *radians* | Cosine (input in radians) |
| `radarctan` *num* | Arctangent in radians. `(radarctan y x)` for two-argument form. |

To get π: `2 * (radarctan 0 1)`

### Numeric Predicates

| Primitive | Alias | Infix | Description |
|-----------|-------|-------|-------------|
| `lessp` | `less?` | `<` | true if num1 < num2 |
| `greaterp` | `greater?` | `>` | true if num1 > num2 |

### Random Numbers

```logo
random 10                    ; random nonneg integer < 10
rerandom                     ; make random reproducible (reseeds)
(rerandom seed)               ; specific seed for repeatable sequence
```

### Print Formatting

```logo
form num width precision
; outputs a word with at least width characters, exactly precision digits after the decimal
; form 3.14159 10 2  →  "      3.14

; Debugging: (form num -1 format) uses C printf format string
```

### Bitwise Operations

| Primitive | Description |
|-----------|-------------|
| `bitand` | Bitwise AND. Variadic. |
| `bitor` | Bitwise OR. Variadic. |
| `bitxor` | Bitwise XOR. Variadic. |
| `bitnot` | Bitwise NOT. |
| `ashift` | Arithmetic shift left by *num2* bits (right shift if negative, with sign extension). |
| `lshift` | Logical shift left by *num2* bits (right shift if negative, with zero fill). |

---

## Logical Operations

```logo
and tf1 tf2             ; true if all inputs are true. Variadic.
or  tf1 tf2             ; true if any input is true. Variadic.
not tf                  ; true if input is false
```

All inputs must be the words `true` or `false` (case-insensitive). UCBLogo's `and`/`or` evaluate all arguments — they do **not** short-circuit.

---

## Graphics

Berkeley Logo provides traditional Logo turtle graphics with **one turtle**. Multiple turtles, dynamic turtles, and collision detection are not supported.

### Coordinate System

- Center of the graphics window is turtle location `[0 0]`
- Positive X is to the right; positive Y is up
- Headings are in **degrees clockwise from the positive Y axis** (0 = north/up, 90 = east/right)
- Logo attempts to scale the screen so that `[-100 -100]` and `[100 100]` fit in the graphics window with a 1:1 aspect ratio

### Colors

Logo interprets color numbers 0–7 uniformly:

| 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|
| black | blue | green | cyan | red | magenta | yellow | white |

Colors 8–15 (if available): 8=brown, 9=tan, 10=forest, 11=aqua, 12=salmon, 13=purple, 14=orange, 15=grey.

Logo begins with a black background and white pen.

### Turtle Motion

| Primitive | Alias | Description |
|-----------|-------|-------------|
| `forward` | `fd` | Move forward by *dist* turtle steps |
| `back` | `bk` | Move backward by *dist* (heading unchanged) |
| `left` | `lt` | Turn counterclockwise by *degrees* |
| `right` | `rt` | Turn clockwise by *degrees* |
| `setpos` | | Move to absolute `[x y]` position |
| `setxy` | | `setxy xcor ycor` — two-argument form of setpos |
| `setx` | | Move horizontally to new X coordinate |
| `sety` | | Move vertically to new Y coordinate |
| `home` | | Move to `[0 0]` heading 0. Equivalent to `setpos [0 0]`. |
| `setheading` | `seth` | Set absolute heading (degrees clockwise from north) |
| `arc` | | `arc angle radius` — draw an arc with turtle at center, clockwise from current heading |

### Turtle Motion Queries

| Primitive | Description |
|-----------|-------------|
| `pos` | Outputs `[x y]` — current position |
| `xcor` | Outputs X coordinate *(library)* |
| `ycor` | Outputs Y coordinate *(library)* |
| `heading` | Outputs current heading in degrees |
| `towards` | `towards [x y]` — heading to face the given position |
| `scrunch` | Outputs `[xscale yscale]` — current scrunch factors |

### Turtle and Window Control

| Primitive | Alias | Description |
|-----------|-------|-------------|
| `showturtle` | `st` | Make turtle visible |
| `hideturtle` | `ht` | Make turtle invisible (speeds up drawing) |
| `clean` | | Erase all drawings, keep turtle state |
| `clearscreen` | `cs` | Erase all drawings + `home` (like `home` + `clean`) |
| `wrap` | | Turtle wraps around screen edges (default mode) |
| `window` | | Turtle can move off-screen (infinite plane) |
| `fence` | | Turtle stops at screen edges with "out of bounds" error |
| `fill` | | Flood-fill the region containing the turtle |
| `label` | | `label "text` — print text on the graphics window at turtle position |
| `textscreen` | `ts` | Maximize text window |
| `fullscreen` | `fs` | Maximize graphics window |
| `splitscreen` | `ss` | Show both text and graphics windows |
| `setscrunch` | | `setscrunch xscale yscale` — adjust aspect ratio/scaling |
| `refresh` | | Remember turtle motions for window overlay reconstruction |
| `norefresh` | | Don't remember (faster, but overlayed graphics lost) |

### Turtle and Window Queries

| Primitive | Alias | Description |
|-----------|-------|-------------|
| `shownp` | `shown?` | true if turtle is visible |

### Pen and Background Control

The pen can be **up** or **down**. When down, it operates in one of three modes: **paint** (draw lines), **erase** (erase lines), or **reverse** (invert).

| Primitive | Alias | Description |
|-----------|-------|-------------|
| `pendown` | `pd` | Set pen down (don't change mode) |
| `penup` | `pu` | Set pen up (don't change mode) |
| `penpaint` | `ppt` | Pen down, mode = paint |
| `penerase` | `pe` | Pen down, mode = erase |
| `penreverse` | `px` | Pen down, mode = reverse |
| `setpencolor` | `setpc` | Set pen color (by number) |
| `setpalette` | | `setpalette colornumber rgblist` — define a custom color |
| `setpensize` | | Set pen size |
| `setpenpattern` | | Set pen pattern (hardware-dependent) |
| `setpen` | | `setpen list` — restore pen state from a previous `pen` output *(library)* |
| `setbackground` | `setbg` | Set background color |

### Pen Queries

| Primitive | Alias | Description |
|-----------|-------|-------------|
| `pendownp` | `pendown?` | true if pen is down |
| `penmode` | | Outputs `paint`, `erase`, or `reverse` |
| `pencolor` | `pc` | Outputs current pen color number |
| `palette` | | `palette colornumber` → `[r g b]` (0–65535 each) |
| `pensize` | | Outputs pen size |
| `penpattern` | | Outputs pen pattern |
| `pen` | | Outputs list of pen position, mode, and hardware info *(library)* |
| `background` | `bg` | Outputs background color number |

---

## Property Lists

A property list is a named collection of name-value pairs. Names of property lists are always case-insensitive. Names of individual properties are case-sensitive or case-insensitive depending on `caseignoredp` (default: case-insensitive).

```logo
pprop "myobject "color "red       ; set property
gprop "myobject "color             ; get property → "red (or [] if none)
remprop "myobject "color           ; remove property
plist "myobject                    ; outputs flat list [name1 val1 name2 val2 ...]
```

Property lists persist in the workspace alongside procedures and variables.

---

## Workspace Management

### Variable Definition

| Primitive | Description |
|-----------|-------------|
| `make "var value` | Assign value to variable. If the name already exists as a local, changes it; otherwise creates/updates global. |
| `name value "var` | Same as `make` with arguments reversed. *(library)* |
| `local "var` | Declare local variable in current procedure (dynamic scope). `local [var1 var2]` for multiple. Variables created by `local` have no initial value. |
| `localmake "var value` | `local` + `make` in one step. *(library)* |
| `thing "var` | Outputs the value of the named variable. `:var` is sugar for `thing "var`. |

### Workspace Predicates

| Primitive | Alias | Description |
|-----------|-------|-------------|
| `procedurep` | `procedure?` | true if input names a procedure |
| `primitivep` | `primitive?` | true if input names a primitive (built-in) procedure |
| `definedp` | `defined?` | true if input names a user-defined procedure |
| `namep` | `name?` | true if input names a variable |
| `macrop` | `macro?` | true if input names a macro |

### Contents Lists

A **contents list** is a list of three lists: `[procedures variables plists]`.

| Primitive | Description |
|-----------|-------------|
| `contents` | All unburied named items |
| `buried` | All buried named items |
| `procedures` | List of all unburied procedure names (a list of names, not a contents list) |
| `names` | Contents list with only variables |
| `plists` | Contents list with only property lists |
| `namelist "var` | Contents list for one or more variable names *(library)* |
| `pllist "pname` | Contents list for one or more property list names *(library)* |

### Inspection

| Primitive | Description |
|-----------|-------------|
| `po contentslist` | Print definitions of the named items |
| `poall` | Print all unburied definitions *(library)* |
| `pops` | Print all procedure definitions *(library)* |
| `pons` | Print all variable definitions *(library)* |
| `popls` | Print all property list contents *(library)* |
| `pon "var` / `pon [vars]` | Print named variables *(library)* |
| `popl "pname` / `popl [pnames]` | Print named property lists *(library)* |
| `pot contentslist` | Print title lines (signatures) of procedures and property lists |
| `pots` | Print title lines of all procedures *(library)* |

### Workspace Control

| Primitive | Description |
|-----------|-------------|
| `erase` / `er` | Erase procedures, variables, and/or property lists named in the contents list |
| `erall` | Erase all unburied items *(library)* |
| `erps` | Erase all unburied procedures *(library)* |
| `erns` | Erase all unburied variables *(library)* |
| `erpls` | Erase all unburied property lists *(library)* |
| `ern "var` | Erase the named variable(s) *(library)* |
| `erpl "pname` | Erase the named property list(s) *(library)* |
| `bury contentslist` | Hide items from `contents`, `pops`, `save`, etc. |
| `buryall` | Bury all items *(library)* |
| `buryname "var` | Bury the named variable(s) *(library)* |
| `unbury contentslist` | Un-hide items |
| `unburyall` | Unbury all *(library)* |
| `unburyname "var` | Unbury the named variable(s) *(library)* |

### Debugging

| Primitive | Description |
|-----------|-------------|
| `trace contentslist` | Print entry/exit/assignments for named items |
| `untrace contentslist` | Turn off tracing |
| `step contentslist` | Pause before each instruction line when named procedures run |
| `unstep contentslist` | Turn off stepping |

### Editing and Files

| Primitive | Description |
|-----------|-------------|
| `edit` / `ed` | Edit definitions using the EDITOR environment variable. `(edit)` re-edits the previous temp file. |
| `edall`, `edps`, `edns`, `edpls` | Edit all / procedures / variables / property lists *(library)* |
| `edn "var`, `edpl "pname` | Edit named variables / property lists *(library)* |
| `save "filename` | Save all unburied definitions to file |
| `savel contentslist "filename` | Save specific items to file *(library)* |
| `load "filename` | Read and execute instructions from file. If `startup` variable is set, runs it after loading. |
| `help "name` | Print reference manual info about a primitive. `(help)` lists all. |

---

## Macros

A macro is a special kind of procedure whose output is evaluated as Logo instructions **in the context of the macro's caller** — not in the macro's own context. This makes macros suitable for defining new control structures.

```logo
.macro my.repeat :num :instructions
  if :num = 0 [output []]
  output sentence :instructions ~
                  (list "my.repeat :num - 1 :instructions)
end
```

Every macro is an operation — it must always output something. Even in the base case, it outputs an empty instruction list `[]`.

| Primitive | Description |
|-----------|-------------|
| `.macro` | Like `to`, but defines a macro instead of a procedure |
| `.defmacro` | Like `define`, but for macros |
| `macrop` / `macro?` | true if the input names a macro |
| `macroexpand` | Takes a Logo expression that invokes a macro, outputs the expanded instruction list *(library)* |

> **Note**: Logo macros are *not* special forms. The inputs to the macro are evaluated normally; only the *output* from the macro is handled unusually.

### Backquote

The backquote (`` ` ``) is a library procedure that outputs a list equal to its input but with certain substitutions:

- If a member of the input list is the word `,` (comma), the following member should be an instructionlist that produces an output when `run`. The comma and the instructionlist are replaced by that output.
- If a member is `,@` (comma-atsign), the following member should produce a list when `run`. The `,@` and the instructionlist are replaced by the *members* of that list (splice).

```logo
show `[foo baz ,[bf [a b c]] garply ,@[bf [a b c]]]
; [foo baz [b c] garply b c]
```

---

## Special Variables

Logo takes special action if any of the following variable names exist. They follow normal scoping rules — a procedure can `local` one to limit its effect.

| Variable | Default | Description |
|----------|---------|-------------|
| `caseignoredp` | `true` (buried) | If true, `equalp`, `beforep`, `memberp`, etc. treat upper and lower case as equal |
| `erract` | *(unbound)* | Instructionlist run on error. Typically `[pause]` for interactive debugging. |
| `loadnoisily` | *(unbound)* | If true, prints procedure names when loading from a file |
| `printdepthlimit` | *(unbound)* | Max depth of sublist structure printed by `print` etc. |
| `printwidthlimit` | *(unbound)* | Max number of members printed per list by `print` etc. |
| `redefp` | *(unbound)* | If true, allows primitives to be erased or redefined (via `copydef`) |
| `startup` | *(unbound)* | If set in a file loaded with `load`, run as an instructionlist after loading |

---

## Key Idioms and Patterns

### Recursive List Processing

```logo
to mysum :list
  if emptyp :list [output 0]
  output (first :list) + mysum butfirst :list
end

to myfilter :pred :list
  if emptyp :list [output []]
  if invoke :pred first :list
    [output fput first :list myfilter :pred butfirst :list]
  output myfilter :pred butfirst :list
end
```

### Building Up a Result Word

```logo
to codeword :word :code
  if emptyp :word [output "]
  output word (codelet first :word :code) (codeword butfirst :word :code)
end
```

### Anonymous Templates

```logo
map [? * ?] [1 2 3 4]
filter [? > 0] :numbers
map.se [list ? (* ? ?)] [1 2 3]
```

### `run` for Instruction Lists

```logo
run [forward 50 right 90]
run :some.list
```

### `invoke` for Operations

```logo
invoke "square 5              ; calls square with input 5, outputs 25
(invoke :op :arg1 :arg2)
```

---

## Commonly Overlooked Details

- **`print` vs `show`**: `print` strips list brackets; `show` shows them. Always use `show` when displaying lists for debugging.
- **Empty word**: `"` is the empty word; `[]` is the empty list. `emptyp "` → true.
- **`=` vs `equalp`**: `=` is infix and works for numbers and words; `equalp` works for any datum.
- **`bf`/`bl`**: abbreviations for `butfirst`/`butlast` — used constantly in practice.
- **`se`**: abbreviation for `sentence`.
- **`op`**: abbreviation for `output`.
- **`:var` in procedures**: dynamic scope means a called procedure can access variables of its caller. This is intentional in Berkeley Logo.
- **`#` in templates**: yields position (1-based), not value. `map [#] [a b c d]` → `[1 2 3 4]`.
- **Array indexing**: starts at 1 by default; `(array 5 0)` makes it start at 0.
- **`remainder` vs `modulo`**: `remainder` gives the sign of the dividend; `modulo` gives the sign of the divisor.
- **`lput` vs `fput`**: `fput` prepends (efficient for lists); `lput` appends (less efficient). Prefer `fput` in recursion, `reverse` at the end.
- **`map.se` vs `map`**: `map.se` flattens one level — each element's result is `sentence`d into the accumulator.
- **`reduce`**: works right-to-left (starts with last two elements), not left-to-right. Requires at least 1 element.
- **Infix arithmetic precedence**: standard math precedence (`*` before `+`).
- **`test` state is per call frame**: `test` stores a boolean local to the procedure in which it is used.
- **`make` in a procedure creates a global** unless `local` was declared first.

---

## Error-Prone Situations

1. **Missing stop rule** → `butfirst doesn't like [] as input`
2. **Forgetting to quote a word**: `print hello` tries to call procedure `hello`
3. **Forgetting list brackets**: `if x > 0 stop` → `stop` is not in an instruction list
4. **`and`/`or` evaluate all arguments**: UCBLogo has no short-circuit for prefix `and`/`or`
5. **`for` step direction**: `for [i 5 1]` doesn't count down unless step is `-1`; use `for [i 5 1 -1]`
6. **`make` in a procedure creates a global** unless `local` was declared first
7. **`item` is 1-indexed** for both lists and arrays (unless array was created with a different origin)
8. **Minus sign ambiguity**: `- 3+4` = `(-3)+4` but `minus 3+4` = `-(3+4)` — spacing matters

---

## Design Philosophy

Berkeley Logo is designed to teach **functional thinking** and **recursion** through concrete examples. Key principles reflected in the design:

- **No assignment as default**: prefer recursion and higher-order functions over mutation
- **Procedures are data**: procedures can be passed by name (`"myprocedure`) or as templates (`[? * 2]`)
- **Dynamic scope** enables clean decomposition (utility procedures can access their callers' state)
- **Interactive development**: define procedures, test them, redefine — no compile step
- **Everything is a procedure call**: uniform syntax; `to`/`end`/`make` look like special forms but are processed by the parser, not the evaluator
- **No static types**: all data is words, lists, or arrays; numeric operations coerce as needed
- **Call by binding**: Logo parameters are neither call-by-value nor call-by-reference — the called procedure gets a new local variable initialized to the value of the argument expression (Brian Harvey's term "call by binding" from Vol. 3, Ch. 4)
