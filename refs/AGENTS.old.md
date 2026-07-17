---
**Document:** Iklo Shell & Language Reference
**Purpose:** Specification of the Iklo language, documenting language features and implementation status.
**Audience:** Language designers, interpreter implementers, AI agents reasoning about language semantics.
**Last Updated:** 2026-07-17
**Status:** Actively maintained
**Cross-refs:** 
  - Bugs — filed as [GitHub Issues](https:#github.com/rsenna/iklo/issues) (label `bug`)
    - **when you notice an unrelated defect mid-task, open an issue** instead of losing it
---

# Iklo

Takes inspiration from UCBLogo, but is not a compatibility target.
Sometimes UCBLogo will be used as a *comparison baseline* for Iklo: we will mention in which ways Iklo differs from
UCBLogo.

- **TBI**: To Be Implemented: means the related feature has not yet been implemented.
- **TBD**: To Be Defined: means the related feature is planned but not yet defined.
- **BET**: means the related feature is neither implementer nor planned, it's still just a possibility or a
  recommendation.

- **Forms** are the basic building blocks of the language. They can either be functions, macros or "special-forms"
  (i.e. what other languages might call "keywords").

---

# Iklo Language Reference

- **Style**: **TBD**
  - Indentation: four spaces.
  - Line continuation: eight spaces.

- **Shell/Python Style Comments**: starting with `# ` (hash and space).

- **Unary operators**: `!` (logical not), `~` (bitwise not)

- **Binary operators**: `||`, `&&`, `==`, `!=`, `<`, `>`, `<=`, `>=`, ... (supporting *short-circuit* evaluation).
  Similar to bash, a sequence of `:a && :b && :c` would return the value returned by evaluating `c` if (and only if)
  `:a` and `:b` are truthy. That means logical operators are also a valid way of "combining" expressions.

  - `->` if a "option-with-value constructor". Used e.g. with map literals.
    ```iklo
    (-a = 20) == (-a 20) == a->20
    ```

- **Assignment**: **TBI**

  - Primitive assignment expressions: support different *kinds* of assignment (lexical, dynamic, global, etc.)
    with different sigils or keywords. For instance:

    ```Iklo
    ### Dynamic
    
    set $a to 5
    set $b to 6
    
    # prints "5 6"
    print $a $b
    
    # prints "500 6"
    let $a be 500 [
        print $a $b
    ]
    
    # prints "5 6"
    print $a $b

    ### Lexical
    
    let :x be 20
    
    # prints "20"
    print :x
    ```

- **Closure syntax**: **TBI**

  ```Iklo
  to some-proc :a :b do
      # Note `some-lambda` has no prefix - this is the "form" binding
      # Same as `let some-lambda [fn :x :y [return :x + :y]]`:
      let some-lambda do
          fn :x :y do
              return :x + :y
          end
      end

      return some-lambda 7 8
  end
  ```

- **Convention**: use '?', '!' and '?!' suffixes:
  - `?` replaces the 'p' suffix (so `equal?`, instead of `equalp`)
  - `!` should be used to indicate a procedure is "dangerous" (similar to UCBLogo's `.` prefix).

- **Errors**: should *not* be implemented like standard OOP-like exceptions, but use a simpler mechanism based on 
  returning sum types (e.g. `^int or ^error`).

- **Function Purity**: **BET** how to track and enforce side-effect discipline. The goal is similar in spirit to
  Haskell but without the monad cognitive load. Procedures that perform any IO should be flagged as such, with the
  *exact kind* of IO (terminal? graphical? file?). 

  - **Regular Form Declaration/Definition** **TBI**

    ```iklo
    to <proc-name> [ <call-pattern> ] [ -> ^some-type ] [ where <condition> ] do
     ...
    end
    ```

    `<call-pattern>` is a sequence of *formal parameters* and *literal tokens*.

      - A formal parameter is a token prefixed with `:` (a lexical binding).
      - A literal token is a bare word that must appear in the call site.
  
    One restriction: `<call-pattern>` **must** start with a *formal parameter*.
    (Meaning if the form name is made of multiple English words, then whitespace cannot separate those words.) 

    The parser uses the pattern to match calls to the procedure.

    ```iklo
    # Examples
    to buy-food :a apples :b steaks :c soft drinks do ... end
    buy-food 4 apples 3 steaks 3 soft drinks
  
    to proc2 ^int :a, ^string :b, ^list :c do ... end
    proc2 4 "bread" [1 2 3]
  
    to proc3 :a :b :c -> ^int or ^error
            where :a > :b > :c
  
        # better if it was a `where` condition, this is just an example
        if :a - :b < :c do
            return ^error "Invalid arguments"
        end
  
        :a - :b - :c  
    end
    ```
---

## Evaluation Model

- There should be *no parsing ambivalence*.
  - There must be only one possible way that a block of code might be parsed.
  - And no exceptions, or subtle different ways were certain forms could be evaluated.
  - Evaluation Model must be **uniform** and as simple as possible.

- Procedures *must* declare its *default cardinality* in case they accept a variable amount of arguments.

- **Commas as whitespaces**: **BET**: commas are *simply ignored*, and can be used as whitespaces too
---

## Annotations

Similar to Java and Clojure, metadata can be assigned to code through annotations.

Annotation calls are made using the `#!` prefix:

```iklo
#!SomeAnnotation
to my-procedure :n do
  :n
end
```

## Bindings

```iklo
to make-adder :n do
    fn do
        :n + 1    # :n is captured from the enclosing definition
    end
end
```

Like UBCLogo and many Lisps, dynamic scope exists but must be opted into explicitly.
Currently the only way of doing that is through `set` and bind-prefixes:

```iklo
# `$` is the prefix for dynamic vars, and `set` assign values to vars
set $a to 5
```

## Block Syntax: `do ... end`

UCBLogo uses `to ... end` for procedure definitions and `[...]` for inline instruction lists.

Iklo tightens this:
  - Every construct introducing a block **must** open it with `do`, and close it with `end`
  - Every block ending with `end` **must** begin with `do`

### Procedure definitions

```logo
to square :side
  repeat 4 [forward :side right 90]
end
```

```iklo
to square :side do
    repeat 4 do
        forward :side
        right 90
    end
end
```

`do ... end` was borrowed from Elixir.

## Conditional Syntax

UCBLogo conditionals use quoted lists as branches:

```logo
if :x > 0 [print "positive]
ifelse :x > 0 [print "positive] [print "non-positive]
```

In Iklo we prefer to use keyword-delimited blocks (though quoted lists are also valid).

```iklo
if :y > 0 [ print 'positive ] else [ print 'non-positive ]

if :x > 0 do
    print 'positive
else-if :x == 0 do
    print 'zero
else
    print 'negative
end
```

Notes:

- `ifelse` is not a primitive in Iklo.

- `do ... [else-if ...] else ... end` is an `if`-specific extension to the `do ... end` block syntax.
  - **NOT** a general-purpose block form.
  
- `else` (and `else-if`) are **required tokens** for the `if` form, even for the quoted-list overload.

---

## Procedure Patterns

Iklo accepts function calls using **patterns**.

A procedure declaration can specify how the parser should recognise a call, including:

- **Prefix** `name :arg1 :arg2`

- **Infix** `:arg1 name :arg2` (e.g. arithmetic operators, `+`, `-`, etc.)

- **Suffix** `:arg1 name`

- **Bare tokens required between or after arguments**: e.g. `if <cond> do <body> end` requires the literal tokens 
  `do` and `end` to be present in the call site.

- **Default Cardinality**: **TBD** We need some way of indicating how many arguments a procedure wants *by default*,
  specially when the same procedure can receive a *variable amount of arguments*
  - e.g. `sum`, which accepts 2 arguments *by default*, but can actually receive any number of arguments.
  - The parser needs to know how many arguments to expect before it can parse the call.

### Declaring patterns

The syntax for pattern declaration is **TBD** and will be specified here.

### Built-in examples

```
if <:cond> do <:body> end
if <:cond> do <:then-body> else <:else-body> end
to <name> <:args>... as <:body> end
```

`else` is not a standalone procedure, but a **required bare token** in the `if` patterns.

The parser enforces their presence.

**Why patterns**:

- This makes the grammar extensible and self-describing.
- New control structures can be added in library code using the same mechanism as built-ins, without parser changes.
- Great for DSLs

---

- [ ] TODO: From here on document has NOT been properly updated for Iklo yet
  - Syntax, semantics, and examples still follow deprecated LogoScript implementation.
  - *Some* decisions might be valid for Iklo, others not.

---

## Strings

Iklo supports regular strings — not just sequences of words. Strings use C-style double-quoted format and may contain spaces and escape sequences.

```iklo
setq greeting "Hello, world!"
setq path "C:\\users\\logo"
print "this is a string with spaces"
```

Strings are distinct from word literals (which use `'`). A word is a single token; a string can contain whitespace and is delimited by `""` (see Delimiters).

TBD: there should be *some* extra sigils/delimiters for distinct string literal parsing
- **the backquote sigil for string-interpolation**: `` `"some interpolable string using variables :var1 and :var2, or 
even an expression such as $(1 + 2 + 3)"``
- **the `"""` delimiter** (raw / multi-line strings):

  ``````
  letq my-var """
  a "docstring" that accepts all characters as-is,
  including double-quotes, line-breaks and \ backslashes
  """
  ``````

---

## Namespaces and Packages

**Iklo** adds a namespace system inspired by *Clojure*. All names (procedures and variables) belong to a namespace; the default is `user` (or similar — TBD). Names can be namespace-qualified: `geometry/square`, `math/pi`. A `require` form (or equivalent — TBD) loads a package and makes its public names available. A package is a collection of namespaced definitions that can be distributed and loaded independently.

UCBLogo has a flat global namespace, which becomes unmanageable for large programs or library ecosystems. Clojure's namespace model is a proven, simple approach: namespaces are first-class, aliasing is explicit, and there is no implicit importing.

Specifics (syntax for `ns`, `require`, aliasing, visibility modifiers) will be documented here as they are decided.

---

## Execution Contexts: Module Mode vs REPL Mode

Iklo runs in one of two distinct execution contexts. Some language forms behave differently depending on which context is active. This distinction must be tracked carefully in documentation and tests.

### Module Mode

A source file is treated as a **compilation unit**. All top-level `to` declarations are processed in a single pass before execution begins. Within a module:

### REPL Mode

The REPL is an **interactive session**, not a compilation unit. Forms are evaluated one at a time as they are entered.

- `to` declarations are **live updates** to the global function table: redefining a procedure replaces the previous definition immediately.
- This is intentional and necessary for iterative development — fixing a buggy procedure should not require restarting the session.
- `&my-proc` in REPL mode behaves as a **late-binding reference**: if you save a reference and then redefine the procedure, calls through the reference see the new definition. This is the expected behavior for interactive work.

### Documentation Convention

Whenever a language form behaves differently between the two contexts, document both behaviors explicitly using the markers `[MODULE]` and `[REPL]`:

```
[MODULE]  to redeclaration → compile-time error
[REPL]    to redeclaration → replaces previous definition
```

A form with no such markers behaves identically in both contexts.

### Testing Convention

Maintain two distinct test groups:

- `tests/module/` — exercises module-mode behavior (static compilation, redeclaration errors, fixed function table).
- `tests/repl/` — exercises REPL-mode behavior (live redefinition, late-binding references, session state).

Any behavior that differs between contexts **must** have a test in both groups, with the expected outcome explicitly different. Behavior identical in both contexts lives in `tests/shared/` and is run under both harness modes.

---

## Primitive Reference

This section is the canonical record of every built-in special form, keyword, and native procedure in Iklo. Add an entry here whenever a new primitive is defined (in the interpreter, parser, or lexer). Each entry documents the call signature, semantics, Iklo deviations from UCBLogo (if any), and implementation status.

**Status tags**

| Tag | Meaning |
|-----|---------|
| `[DONE]` | Implemented and tested |
| `[TBI]` | To be implemented |
| `[PARTIAL]` | Implemented but incomplete or untested |

**Kind tags** — what the runtime must do to evaluate the form

| Tag | Meaning |
|-----|---------|
| `[SPECIAL]` | Special form: arguments are NOT evaluated before dispatch (the evaluator handles them directly) |
| `[NATIVE]` | Native function: arguments are evaluated before the Rust fn receives them |
| `[KEYWORD]` | Pure syntax — consumed by the parser, produces no value at runtime |

---

### Control flow

#### `if` … `do` / `else` … `end` — [SPECIAL] [DONE]

```iklo
if <cond> do <body> end
if <cond> do <then-body> else <else-body> end

# UCBLogo bracket form also accepted:
if <cond> [<body>]
ifelse <cond> [<then>] [<else>]
```

Evaluates `<cond>`. If truthy, evaluates `<then-body>`; otherwise evaluates `<else-body>` (if present). `then`, 
`else`, and `end` are required literal tokens in the keyword-block form. `do` is used (instead of `then`) in 
the body-opening position.

---

#### `to` … `do` / `end` — [SPECIAL] [DONE]

```iklo
to <name> <:param>... do
  <body>
end

# Pattern form (mixfix):
to <name> <:param | literal-word>... do <body> end
```

Defines a named procedure. Redeclaration replaces the previous definition. `is` marks the boundary between the 
header and body; `end` closes the body.

---

#### `return`

```iklo
return <expr>
```

Immediately returns `<expr>` as the value of the enclosing procedure. No further statements in the procedure body 
are evaluated.

---

#### `stop`

```iklo
stop
```

Returns from the enclosing procedure with no value (equivalent to returning `-nil`). No further statements are 
evaluated.

---

#### `run`

```iklo
run <list-or-block>
```

Evaluates the given instruction list (or block value) as a sequence of statements in the current scope. Used to invoke a dynamically-computed body.

---

#### `apply`

```iklo
apply <func> <arg1> <arg2> ...
(apply :func, :arg1, :arg2)
```

Calls `<func>` (which must be a `Value::Function` or a native function value) with the given arguments. Unlike calling a function by name, `apply` can invoke a value held in a variable. Must be a special form because evaluating it requires access to `call_inner`.

*Note*: `(apply double 21)` parses as `(apply 'double 21)`
  - Parentheses apply the first argument, and **only** the first argument (like Lisp).
  - To avoid automatic apply, use `'(apply double 21)` (also like Lisp).
  - Other collection delimiters (`[]`, `{}`) **NEVER** apply functions.
    - So `[apply double 21] == '[apply double 21] == ['apply 'double 21]`

---

#### `forever` — [SPECIAL] [DONE]

```iklo
forever do
  <body>
end

# UCBLogo form:
forever [<body>]
```

Evaluates `<body>` in an infinite loop. The only way to exit is via `stop` (exits the enclosing procedure), `throw` (TBI), or an error. `forever` itself never outputs a value.

*Iklo note*: the `do … end` block is the preferred body form; the UCBLogo bracket form is also accepted.

*Implementation*: `crates/iklo-interp/src/lib.rs:409`

---

#### `repeat` — [SPECIAL] [DONE]

```iklo
repeat <n> do
  <body>
end

# UCBLogo form:
repeat <n> [<body>]
```

Evaluates `<body>` exactly `<n>` times. `<n>` must evaluate to a non-negative integer. Does not bind a loop variable. Returns `#nil`.

UCBLogo makes the current iteration count available inside the body as `repcount`; Iklo will follow this convention (TBI — see spec/backlog.md §3).

*Implementation*: `crates/iklo-interp/src/lib.rs:429`

---

#### `for` — [SPECIAL] [PARTIAL]

```iklo
for [<var> <start> <end>] do
  <body>
end

for [<var> <start> <end> <step>] do
  <body>
end

# UCBLogo forms (also accepted):
for [i 1 10]    [<body>]
for [i 1 10 2]  [<body>]    # step 2
for [i 10 1 -1] [<body>]    # count down
```

Binds `<var>` to `<start>`, then evaluates `<body>` repeatedly, incrementing `<var>` by `<step>` (default `1` if start ≤ end, `-1` if start > end) after each iteration, stopping when `<var>` passes `<end>`. The binding is lexical inside the body. Returns `#nil`.

*Iklo deviation*: the control list `[var start end]` / `[var start end step]` uses the UCBLogo bracket-list syntax for the header; an alternative keyword form may be added (TBI).

*Parsing note*: A negative step literal like `[i 5 1 -1]` works via prefix-vs-infix dispatch (Grammar 2.0 §13e): a *prefix-shaped* `-` (whitespace before, glued to what follows) starts a new expression, so `1 -1` is end=1, step=−1, while `1 - 1` and `1-1` are still subtraction.

*Implementation*: `crates/iklo-interp/src/lib.rs:467`. See spec/backlog.md §3 for known issues (cleaner keyword form).

*CONCERNS*:
  - Bare words as indices
    - In UCBLogo's `for [i 1 10]` example, `i` is a bare word *implicitly* declared as a dynamic variable.
    - In Iklo, ideally `i` should be `'i` instead, and always be introduced as a lexical variable (like `let` 
      does)
  - How to parse `<body>` in Iklo?
    - In Iklo, a code block should also be a list.
    - But, maybe different from other lists, its elements should not ever be resolved automatically.
    - A possible solution might be making `for` a type of macro, that automatically quotes its `<body>` argument.
      - That's similar to what `letq x 3` does with `x`
      - **Note (2026-07-09):** there is no macro system yet; `for` is a hardcoded parser arm
        (`parser:801`) dispatched by name. Migrating it (and `if`, `run`, `output`, `apply`) from
        hardcoded special forms to real macros is explicit follow-on work in
        [ADR-0004](design/decisions/ADR-0004-macros-and-bounded-reader-extension.md), and can be
        done incrementally rather than big-bang.

---

#### `foreach` — [SPECIAL] [DONE]

```iklo
foreach <list> do
  # current element bound to `?` (template convention)
  <body>
end

# UCBLogo forms (also accepted):
foreach [a b c] [print ?]    # ? = current element
foreach [a b c] "print        # pass procedure name directly
```

Iterates over each element of `<list>`, evaluating `<body>` once per element. The current element is accessible as `?` inside template lists (UCBLogo convention). Returns `#nil`.

*Iklo note*: the `?` template variable convention is inherited from UCBLogo. A cleaner Iklo syntax for naming the iteration variable is TBI (possibly `foreach :x in <list> do … end`). Also note: `foreach` may be renamed to `for-each` per spec/backlog.md §1 naming convention.

*Implementation*: `crates/iklo-interp/src/lib.rs:512`

---

### Binding and scope

#### `make` / `set` — [SPECIAL] [DONE]

```iklo
make 'name <value>
set  'name <value>
setq  name <value>   # TBI — quoted form (like letq, but dynamic scope)
```

Writes `<value>` into the **dynamic** (global/caller) scope under `name`. `set` is a Iklo alias for `make`; both produce the same `Expr::Make` AST node. `setq` (the quoted sugar form) is planned but not yet implemented in the parser.

*UCBLogo deviation*: in UCBLogo `make` always writes to the dynamic scope. In Iklo, `make`/`set` retain that behaviour; use `let`/`letq` for lexical binding.

---

#### `let` / `letq` — [SPECIAL] [DONE]

```iklo
let 'name <value>
letq name <value>
```

Creates a new **lexical** binding for `name` in the current scope. `letq` is syntactic sugar that removes the need for the `'` quote.

---

#### `local` — [NATIVE] [TBI]

```iklo
local 'name
local [name1 name2 ...]
```

UCBLogo compatibility: declares `name` as a local variable in the current procedure's dynamic scope frame. In Iklo, procedure parameters are already lexically scoped, so `local` is mainly for UCBLogo compatibility mode.

---

#### `value` / `thing` — [NATIVE] [DONE]

```iklo
value 'name
thing 'name
```

Returns the value bound to `name` in the current dynamic scope. `thing` is the UCBLogo name; `value` is a Iklo alias.

---

### List and word primitives

#### `first` — [NATIVE] [DONE]

```iklo
first <list-or-word>
```

Returns the first element of a list, or the first character of a word. Error on empty input.

---

#### `last` — [NATIVE] [DONE]

```iklo
last <list-or-word>
```

Returns the last element of a list, or the last character of a word. Error on empty input.

---

#### `but-first` / `butfirst` / `rest` / `bf` — [NATIVE] [DONE]

```iklo
but-first <list-or-word>   # Iklo canonical name
butfirst  <list-or-word>   # UCBLogo alias
rest      <list-or-word>   # conventional alias
bf        <list-or-word>   # UCBLogo abbreviation
```

Returns everything after the first element. Returns an empty list/word if the input has one or zero elements.

---

#### `but-last` / `butlast` / `bl` — [NATIVE] [DONE]

```iklo
but-last <list-or-word>   # Iklo canonical name
butlast  <list-or-word>   # UCBLogo alias
bl       <list-or-word>   # UCBLogo abbreviation
```

Returns everything before the last element. Returns an empty list/word if the input has one or zero elements.

---

#### `fput` — [NATIVE] [DONE]

```iklo
fput <element> <list>
```

Returns a new list with `<element>` prepended.

---

#### `lput` — [NATIVE] [DONE]

```iklo
lput <element> <list>
```

Returns a new list with `<element>` appended.

---

#### `count` / `len` — [NATIVE] [DONE]

```iklo
count <list-or-word>
len   <list-or-word>   # Iklo alias
```

Returns the number of elements in a list, or the number of characters in a word.

---

#### `empty?` / `emptyp` — [NATIVE] [DONE]

```iklo
empty? <list-or-word>
emptyp <list-or-word>   # UCBLogo name
```

Returns `#true` if the argument is an empty list or empty word.

---

#### `list?` / `listp` — [NATIVE] [DONE]

```iklo
list? <value>
listp <value>
```

Returns `#true` if the argument is a list.

---

#### `word?` / `wordp` — [NATIVE] [DONE]

```iklo
word? <value>
wordp <value>
```

Returns `#true` if the argument is a word.

---

#### `number?` / `numberp` — [NATIVE] [DONE]

```iklo
number? <value>
numberp <value>
```

Returns `#true` if the argument is a number.

---

#### `sentence` / `se` — [NATIVE] [DONE]

```iklo
sentence <a> <b>
se <a> <b>
```

Returns a flat list formed by concatenating the members of `<a>` and `<b>`. Words are treated as single-element lists.

---

#### `list` — [NATIVE] [DONE]

```iklo
list <a> <b>
```

Returns a two-element list `[<a> <b>]`. Unlike `sentence`, does not flatten.

---

#### `word` — [NATIVE] [DONE]

```iklo
word <a> <b>
```

Concatenates two words (or numbers coerced to words) into a single word.

---

#### `item` — [NATIVE] [DONE]

```iklo
item <n> <list-or-word>
```

Returns the `<n>`-th element (1-indexed) of a list, or the `<n>`-th character of a word. Error if `<n>` is out of bounds.

*Implementation*: crates/iklo-interp/src/lib.rs:1209

---

#### `member` / `member?` / `memberp` — [NATIVE] [DONE]

```iklo
member  <element> <list-or-word>    # returns suffix starting at first match, or []
member? <element> <list-or-word>    # returns #true/#false
memberp <element> <list-or-word>    # UCBLogo alias for member?
```

When `<list-or-word>` is a word or string, searches for `<element>` as a substring. Numbers are coerced to their string representation before the search. This is a Iklo extension: UCBLogo's `memberp` on a word accepts only a single-character needle.

*Implementation*: crates/iklo-interp/src/lib.rs:1234 (`member`), :1262 (`member?`)

---

#### `pick` — [NATIVE] [DONE]

```iklo
pick <list>
```

Returns a randomly chosen element from `<list>`. Uses the same Xorshift64 PRNG as `random`.

*Implementation*: crates/iklo-interp/src/lib.rs:1283

---

#### `reverse` — [NATIVE] [DONE]

```iklo
reverse <list-or-word>
```

Returns a new list (or word) with elements in reverse order.

*Implementation*: crates/iklo-interp/src/lib.rs:1296

---

#### `rem-dup` / `remdup` — [NATIVE] [DONE]

```iklo
rem-dup <list>   # Iklo canonical name
remdup  <list>   # UCBLogo alias
```

Returns `<list>` with duplicate members removed (first occurrence kept).

*Implementation*: crates/iklo-interp/src/lib.rs:1309

---

#### `ascii` — [NATIVE] [DONE]

```iklo
ascii <word>
```

Returns the code point of the first character of `<word>` (ASCII values for ASCII chars; full Unicode otherwise).

*Implementation*: crates/iklo-interp/src/lib.rs:1349

---

#### `char` — [NATIVE] [DONE]

```iklo
char <n>
```

Returns the one-character word whose Unicode code point is `<n>`.

*Implementation*: crates/iklo-interp/src/lib.rs:1364

---

#### `upper-case` / `lower-case` — [NATIVE] [DONE]

```iklo
upper-case <word>   # Iklo canonical name
lower-case <word>   # Iklo canonical name
uppercase  <word>   # UCBLogo alias
lowercase  <word>   # UCBLogo alias
```

Returns the word with all characters converted to upper or lower case.

*Implementation*: crates/iklo-interp/src/lib.rs:1327 (`upper-case`), :1338 (`lower-case`)

---

#### `before?` / `beforep` — [NATIVE] [DONE]

```iklo
before?  <word1> <word2>
beforep  <word1> <word2>   # UCBLogo alias
```

Returns `#true` if `<word1>` comes before `<word2>` in lexicographic order (case-sensitive). Numbers are coerced to their string representation before comparison.

*Implementation*: crates/iklo-interp/src/lib.rs:1382

---

### Arithmetic

#### `+`, `-`, `*`, `/` — [SPECIAL] [DONE]

Infix binary operators. No operator precedence — use parentheses for explicit grouping.

---

#### `==`, `!=`, `<`, `>`, `<=`, `>=` — [SPECIAL] [DONE]

Infix comparison operators. Return `#true` or `#false`.

---

#### `&&`, `||` — [SPECIAL] [DONE]

Infix logical operators with short-circuit evaluation. Return the value of the last evaluated operand (bash-style), not necessarily a boolean.

---

#### `sum` / `difference` / `product` / `quotient` / `remainder` / `modulo` — [NATIVE] [DONE]

```iklo
sum        <a> <b> …   # variadic
difference <a> <b>
product    <a> <b> …   # variadic
quotient   <a> <b>
quotient   <a>          # returns 1/<a>
remainder  <a> <b>
modulo     <a> <b>
```

UCBLogo named arithmetic procedures. `sum` and `product` accept a variable number of arguments (default arity 2). `quotient` with one argument returns `1/<a>` (UCBLogo convention). `modulo` result has the sign of the divisor; `remainder` result has the sign of the dividend.

*Iklo note*: paren-free prefix calls currently consume exactly 1 argument. Use explicit-call form for multiple arguments: `sum(2 3 4)`, not `sum 2 3 4`. See spec/backlog.md §1 (default cardinality).

*Implementation*: crates/iklo-interp/src/lib.rs:1155 (`sum`), :1168 (`difference`), :1177 (`product`), :1190 (`quotient`), :1125 (`remainder`), :1211 (`modulo`)

---

#### `sqrt` — [NATIVE] [DONE]

```iklo
sqrt <n>
```

Returns the square root of `<n>`. Error if `<n>` is negative.

*Implementation*: crates/iklo-interp/src/lib.rs:1223

---

#### `abs` — [NATIVE] [DONE]

```iklo
abs <n>
```

Returns the absolute value of `<n>`.

*Implementation*: crates/iklo-interp/src/lib.rs:1235

---

#### `int` / `round` — [NATIVE] [DONE]

```iklo
int   <n>    # truncates toward zero
round <n>    # rounds to nearest integer
```

*Implementation*: crates/iklo-interp/src/lib.rs:1244 (`int`), :1253 (`round`)

---

#### `random` — [NATIVE] [DONE]

```iklo
random <n>
```

Returns a random non-negative integer less than `<n>`. Uses a Xorshift64 PRNG seeded from the system clock on first call.

*Implementation*: crates/iklo-interp/src/lib.rs:1262

---

#### `sin` / `cos` / `atan` — [NATIVE] [DONE]

```iklo
sin  <degrees>
cos  <degrees>
atan <x>         # single-argument arctangent
atan <x> <y>     # 2-argument form: outputs atan(y/x) in degrees
```

Arguments and results are in degrees (UCBLogo convention). The 2-argument form follows UCBLogo order: `atan x y` outputs the arctangent of `y/x`, equivalent to `atan2(y, x)`.

*Implementation*: crates/iklo-interp/src/lib.rs:1276 (`sin`), :1285 (`cos`), :1294 (`atan`)

---

#### `exp` / `log` / `power` — [NATIVE] [DONE]

```iklo
exp   <n>           # e^n
log   <n>           # natural log; error if n ≤ 0
power <base> <exp>  # base^exp
```

*Implementation*: crates/iklo-interp/src/lib.rs:1309 (`exp`), :1318 (`log`), :1330 (`power`)

---

#### `min` / `max` — [NATIVE] [DONE]

```iklo
min <a> <b> …   # variadic
max <a> <b> …   # variadic
```

*Iklo note*: use explicit-call form for multiple arguments: `min(1 2 3)`. See spec/backlog.md §1.

*Implementation*: crates/iklo-interp/src/lib.rs:1339 (`min`), :1355 (`max`)

---

### Higher-order functions

#### `map` — [SPECIAL] [DONE]

```iklo
map <template-or-proc> <list>
```

Returns a new list formed by applying `<template-or-proc>` to each element of `<list>`. `?` inside a template refers to the current element. A `stop` in the body ends iteration early, yielding the list built so far.

The body may be a template block (`[…]`), a quoted procedure name (`'proc`), or a function value (`:proc`) — the same three forms `foreach` accepts.

*Not implemented:* the `#` 1-based index variable. `#` is the line-comment sigil (§13b), so it cannot appear as an identifier inside a template. TBI pending an index-variable spelling that does not collide with comments.

---

#### `filter` — [SPECIAL] [DONE]

```iklo
filter <pred> <list>
```

Returns a new list containing only the elements for which `<pred>` returns truthy. The *original* element is kept, not the predicate's result. `<pred>` takes the same three forms as `map`'s body (template `?`, `'proc`, or function value).

---

#### `reduce` — [SPECIAL] [DONE]

```iklo
reduce <proc> <list>
```

Left-folds `<list>` using the binary `<proc>`, seeding the accumulator with the first element. Requires at least one element (raises otherwise). Template blocks bind the accumulator to `?1` and the current element to `?2` (`reduce([?1 + ?2], list(1 2 3 4))` → `10`); procedure/function bodies are called with `(acc item)`. Unlike the loop primitives, `reduce` produces a value and does not catch `stop`; a `stop`/`output` in the body propagates to the enclosing procedure.

---

#### `find` — [SPECIAL] [DONE]

```iklo
find <pred> <list>
```

Returns the first element for which `<pred>` returns truthy, or `[]` if none. `<pred>` takes the same three forms as `map`'s body.

---

#### `map-se` — [NATIVE] [TBI]

```iklo
map-se <template-or-proc> <list>
```

Like `map`, but flattens one level (each result concatenated via `sentence`).

---

#### `cascade` — [NATIVE] [TBI]

```iklo
cascade <count> <template> <initial>
cascade <count> <tmpl1> <init1> <tmpl2> <init2> ...
```

Applies `<template>` `<count>` times starting from `<initial>`. `?` holds the current accumulated value. The multi-variable form maintains multiple accumulators (`?1`, `?2`, …). Used for compact numeric recursions.

---

#### `invoke` — [NATIVE] [TBI]

```iklo
invoke <proc> <arg1> <arg2> ...
```

UCBLogo higher-order alias for `apply`. TBI — decide whether `apply` already covers all cases.

---

### I/O

#### `print` / `pr` — [NATIVE] [DONE]

```iklo
print <value>
pr    <value>
```

Outputs `<value>` followed by a newline. Lists are printed without surrounding brackets. `pr` is a UCBLogo abbreviation.

---

#### `type` — [NATIVE] [TBI]

```iklo
type <value>
```

Like `print` but without the trailing newline.

---

#### `show` — [NATIVE] [DONE]

```iklo
show <value>
```

Like `print` but lists are printed *with* surrounding brackets.

*Implementation*: `crates/iklo-interp/src/lib.rs:958`

---

#### `readword` / `readline` — [NATIVE] [TBI]

```iklo
readword    # reads one whitespace-delimited word from stdin
readline    # reads one line from stdin as a word
```

---

### Miscellaneous

#### `not` — [NATIVE] [DONE]

```iklo
not <bool>
```

Returns `#true` if `<bool>` is falsy, `#false` otherwise.

---

#### `and` / `or` — [NATIVE] [PARTIAL]

```iklo
and <a> <b>
or  <a> <b>
```

UCBLogo prefix-style logical operators (non-short-circuit). Iklo prefers `&&`/`||` (short-circuit); these are registered as natives but not yet available in prefix form due to parser limitations (they are only parsed as infix operators in `parse_led`, not prefix in `parse_nud`).

*Status note*: Native functions exist but prefix calls fail to parse. See spec/backlog.md for a note on parser work needed to enable prefix syntax.

*Implementation*: `crates/iklo-interp/src/lib.rs` — `and` `:1149`, `or` `:1155` (registered but not prefix-callable)

---

#### `equal?` / `equalp` — [NATIVE] [DONE]

```iklo
equal? <a> <b>
equalp <a> <b>
```

Returns `#true` if `<a>` and `<b>` are equal. `equal?` is the Iklo convention (replaces UCBLogo's `equalp`).

---

#### `not-equal?` — [NATIVE] [DONE]

```iklo
not-equal? <a> <b>
```

Returns `#true` if `<a>` and `<b>` are not equal; `#false` otherwise. Equivalent to `not (equal? :a :b)`.

*Implementation*: `crates/iklo-interp/src/lib.rs:1143`

---

#### `type-of` — [NATIVE] [DONE]

```iklo
type-of <value>
```

Returns a word identifying the type of `<value>`: `number`, `string`, `word`, `keyword`, `list`, `code`, `function`, or `native-function`.

*Note*: Unlike the documented signature which suggests keywords, the implementation returns quoted words (e.g., `'number` not `#number`), and callable values are reported as `'function` or `'native-function` rather than a unified `#procedure`.

*Implementation*: `crates/iklo-interp/src/lib.rs:1115`

---

#### `procedure?` — [NATIVE] [DONE]

```iklo
procedure? <value>
```

Returns `#true` if `<value>` is a callable procedure (user-defined function or native function); `#false` otherwise.

*Implementation*: `crates/iklo-interp/src/lib.rs:1105`

---

#### `keyword?` — [NATIVE] [DONE]

```iklo
keyword? <value>
```

Returns `#true` if `<value>` is a keyword (e.g. `#true`, `#false`, `#nil`, or a user-defined keyword); `#false` otherwise.

*Implementation*: `crates/iklo-interp/src/lib.rs:1110`

---

#### `str` — [NATIVE] [DONE]

```iklo
str <value>
```

Converts `<value>` to its string representation. Similar to `type` but returns the printed form as a string instead of printing to stdout.

*Implementation*: `crates/iklo-interp/src/lib.rs:981`

---

#### `range` — [NATIVE] [DONE]

```iklo
range <start> to <end>
range <start> to <end> step <step>
```

Returns a list of numbers from `<start>` to `<end>` (inclusive), stepping by `<step>` (default 1). Sugar for compact numeric sequences.

*Example*: `range 1 to 5` returns `[1 2 3 4 5]`; `range 10 to 1 step -1` returns `[10 9 8 7 6 5 4 3 2 1]`.

*Implementation*: `crates/iklo-interp/src/lib.rs:1163`. Note: parser requires literal `to` and `step` keywords.

---

#### `error` — [NATIVE] [TBI]

```iklo
error <message>
```

**[TBI]** — documented here but **not implemented** (`error "boom"` → `undefined form 'error'`, verified 2026-07-10).

Raises a runtime error with the given message. Per [ADR-0009](design/decisions/ADR-0009-errors-are-values-no-exceptions.md) this is the **bug / contract-violation tier** — it is not recoverable from within Iklo, because there is no `catch`. It unwinds to the driver, which reports it (the REPL survives because the *driver* catches, not the language).

Expected, recoverable failure is **not** `error`: a failable procedure **returns an error value**, which the caller may inspect, ignore, or propagate with `try`. So this primitive is the analogue of Rust's `panic!`, not of `Err`.

The *representation* of an error value is still open (ADR-0009 §Open: a tagged `[--ok v]`/`[--err e]` pair, or a sealed `^error` runtime kind). **The name is open too** — if errors are values, `error` most naturally *constructs* one, and this raise-primitive wants a different name (`panic`, `fail`).

---

#### `ignore` — [NATIVE] [TBI]

```iklo
ignore <value>
```

Evaluates and discards `<value>`. Useful for calling a procedure that outputs a value when you don't need it.

---

## Appendix

Comments, ideas, possible features, undecided behaviour

### Extensions

These are areas where Iklo is likely to extend UCBLogo but no design decisions have been made yet:

- **Cursors and Surfaces**: a way of thinking of IO in a more abstract way.
    - A *surface* is an object that is fundamentally used for **IO**, and it has two characteristics:
        - A *state* that can change through time.
        - 1+ *cursors*.
    - A *cursor* is an object that points to a *current position* within a *surface*.
        - It supports reading, writing, and moving in both absolute and relative manners.
        - Essentially what the turtle is in UCBLogo, generalised.
    - This abstraction allows *Turtle Graphics* and *File IO* to be treated in a very similar manner:
      *Both are surfaces with cursors* that move around, write/draw, and read.
    - Surfaces *can only be changed through cursors*
        - **Every call** involving surfaces, *but not mentioning any cursor*, should **never** change surface state!
    - 2 fundamental kinds of Surfaces:
        - A *destructible surface* (like a normal terminal screen) allows overwriting at any
          moment, losing previous state.
        - A *non-destructible surface* would not lose previous state — it would be reactive and aware of its history.
        - This distinction matters because, even though non-destructible surfaces involve external state, that state is
          **deterministic**, so procedures operating over them could still be considered *pure* in a meaningful sense.
        - IDEA: Look at reactive frameworks like *React JS* for the simplest viable model of "pure non-destructible IO
          surfaces".
    - Surface categories by dimensionality:
        - *1D* surfaces support forward/backward movement (e.g. iterators, files).
        - *2D* surfaces support X/Y movement (e.g. terminal and graphical screens).
        - *3D* surfaces extend to 3D graphics.
        - *nD* surfaces generalise to arbitrary dimensions (e.g. a view over an N-dimensional array).
        - Note: *time* can also be tracked, but only for *non-destructive surfaces* (by definition).
    - IDEA: **Surfaces for Dependency Injection**
        - Each *DI surface instance* would work as a *DI container*.
        - Plus sigil and annotation support, for binding and resolving dependencies automatically.
- **Mutability**: variables should be **immutable** under many (most?) conditions, but *how* that would happen is still unclear.
- **Multithreading and async**: concurrent turtles, async I/O
- **Multiple cursor**: more than one cursor (or turtle) on screen simultaneously
- **3D turtle graphics**: extending the 2D turtle model to 3D space
- **Extensive standard library**: data structures, string processing, math, I/O beyond UCBLogo primitives
- **Module/package distribution**: how packages are published and consumed
  - Ideally, an `import` should only bring all defines from the referenced package into scope.
  - Instead, a programmer could fully qualify the namespace path they want, without imports.
  ```iklo
  import some/long/namespace in
    print --some-keyword    # declared in some/long/namespace
  end
  
  print some/long/namespace->some-keyword    # equivalent to above block
  ```

### Interpolable Strings & Quasi-Quotes

Syntax should be similar between them two.

- Interpolable Strings
  - `` `":kept-as-is, ~:var1, ~(:var2 + :var3) and _:xs" ``
  - `` `"also works with ~*dynamic-binding (and possibly other bindings too)" ``
- Quasi-quote
  ```iklo
  print `word    # output: 'word
  let 'x 42
  let 'xs [ 1   2    3  ]
  let 'ys ['a 'b]
  print `['word1 :kept-as-is ~:x _:xs ~:ys]    # output: ['word1 :kept-as-is 42 1 2 3 ['a 'b]]
  ```

### Syntax Sugar

- Make `'[a b c :d]` the same as `['a 'b 'c :d]`?
  - Then forms such as `for [`

### Word Bindings

Note: `->` means the left side "is" the same as the right side.
In other words, their internal AST representation should be the same.
  
- Lexical Value
  - `:word` -> `l%word`
- Dynamic Value
  - `$word` -> `d%word`
- Static Procedure
  - `word` (when it is the default binding) -> `s%word`
- Type
  - Note: by convention, types are named in `PascalCase` (not `kebab-case`)
  - Naming Types: `%TypeName`
  - Real: Type Name = `%Float`
    - Values: `3.0`
  - Integer: Type Name = `%Integer`
    - Values: `3`
  - Literal String
    - Values: `"hello"` -> `s"hello"`
  - Quoted word
    - `'word`, `q%word`
  - Keyword
    - Values: `-k` -> `k%k`, `--keyword` -> `k%keyword`
  - Record
    - Types: `%RecordName` -> `R%RecordName`
  - Enum
    - Types: `%EnumName` -> `E%EnumName`
    - `%EnumName/--keyword` -> `%EnumName->keyword`
    - `%EnumName/-k` -> `%EnumName->k`
  - Trait
    - Types: `%Trait` -> `T%Trait`
    - Trait Members:
      - `%Trait/:data-member`
      - `%Trait/method`
      - `:trait-instance.:data-member`
      - `:trait-instance.method`
      - Note: we should be able to declare dependency of traits over dynamic state in some way
  - Surface
    - Types: `%Surface` -> `S%Surface`
    - Surface Members: like traits
  - Impl (?) (in case they are possibly named?)
    - Types: `%ImplName` -> `I%ImplName`
    - Impl Members: like Traits
  - Namespaces
    - Note: by convention, namespaces are `kebab-case`, just like words
    - `p%package-name`
    - `some/package/based/path` -> `p%some/p%package/p%based/p%path`

Note: the **default** binding is 'Static Procedure', meaning a bare word should
be parsed as a static procedure call.
- Author would like for that default to be changeable, but they don't know if
  complexity would increase too dramatically in that case
