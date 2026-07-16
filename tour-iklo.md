# Iklo Language Sketch

*Iklo* is a shell, functional, data and DSL focused language.
It's also yet-another-attempt of making a Lisp *wwith less parentheses*...

It's meant to be used as quick-but-not-so-dirty way of integrating systems.
A "systems scripting language", it also aims to serve varied areas such as devops, TUI prototyping,
and agentic integration & implementation.

Iklo also aims to be a fun way of learning about programming!
It is *intentionally* similar to 80's Logo.
(It even provides its own implementation of Turtle graphics as a loadable package.

But it's not Logo: that was a [very loose programming language standard]() which unfortunately was plagued with many
small inconsistencies, undeterminism, and undefined behaviour...

In this sense, Iklo could not be more distinct: in spite of being essentially a dynamically typed language,
it allows you to be as strict as you want (by, for example, providing optional static typing or even compilation.)

Strictness profile is selectable per module:

- **default**: all syntax enabled.
- **strict**: comma operator is only valid in map literals (as pair separators). Outside maps, `,` is a syntax error.

Its design was also influenced by other languages, such as Clojure, Lisp, Dylan, Ruby, Elixir, Scala, and Haskell.
Its name comes from *ikke Logo* (Danish for "not Logo"!)


## Glossary:

- *sugar*: same as *syntax sugar*, usually a *shorter or simpler alternative* to a built-in literal, operator or form.
- *token*: the canonical symbolic unit in iklo source/runtime (identifier-like atom), used for bindings and form heads.

## Terms

- Tokens, bindings, operators, expressions.

## Concepts

- Values, forms, patterns, types, comments, compute.

## Core / Prologue / Primitive / Built-In

```
#!/usr/bin/env iklo
# comments start with `#`

```

### Types & Literals

| Name         | Description                          | Short Literal              | Literal Constructor            | Form-based Constructor                              |
|--------------|--------------------------------------|----------------------------|--------------------------------|-----------------------------------------------------|
| char         | Character type                       | `\a; \newline`             | `char%{ a }; char%{ newline }` | `char 160; char "a"`                                |
| int8..128    | Integer types (8 to 128 bits)        | `42`                       | `int16%{ -5 }`                 | `int32 9`                                           |
| flo8-128     | Float types (8 to 128 bits)          | `3.14`                     | `flo64%{ -2.5} `               | `flo32 8`                                           |
| number       | A Big Decimal-like type              | `1_234_313_324`            | `number%{ 3_778_383 }`         | `number 42; number "1_878_797_932_949_898_999_823"` |
| token        | Token type, used for bindings        | `foo`                      | `token%{ token with spaces }`  | `token 'foo; token "bar"; token " token with spaces "`  |
| option       | Global static symbol                 | `-foo=bar; foo->bar`       | `option%{ foo bar }`           | `option 'foo 'bar`                                  |
| string       | String type                          | `"hello"`                  | `string%{ world }`             | `string \a \b \c          # "abc"`                  |
| list         | Linked list                          | `[ 1 2 3 ]`                | `list%{ 1 2 3 }`               | `list 1 2 3`                                        |
| syntax-quote | List templates; used in macros       | `` `[ a ~:b _:c d ] ``     | `quote%{ a ~:b _:c d }`        | `quote 'a ~:b _:c 'd      # see 1)`                 |
| vector       | Dense vector                         | `<[ 1 2 3 ]>  # see 2)`    | `vector%{ 1 2 3 }`             | `vector 1 2 3`                                      |
| set          | Set of values                        | `{ a b c }`                | `set%{ a b c }`                | `set 'a 'b 'c`                                      |
| map          | Key-value map                        | `{ a->1, -b, c->2 }`       | `map%{ a->1, -b, c->2 }`       | `map -a 1 -b -nil -c 2    # see 3)`                 |
| stream       | Lazy stream                          | `'( a b c ); ( 'a 'b 'c )` | `stream%{ a b c }`             | `stream 'a 'b 'c`                                   |
| data         | Algebraic Data Type                  | —                          | `data%{ :a=1, :b, :c=2 }`      | `data 'a 1 'b -nil 'c 2`                            |

#### Notes

1) `~:b` and `_:c` are read as *ordinary tokens*.
   `~` and `_` are *simple, token-valid characters* outside of syntax-quote or string-interpolation.

2) **No spaces allowed** between `<` and `[`, or `]` and `>` (no operators nor delimiters allow that).

3) For maps, keys **must** be options.
   Also, since `a->b` is a *global* option syntax, you can also do `map a->1 (-b) c->2`, or `map a->1, -b, c->2`.
   Still, for maps (and in general), the *short literal* or *literal constructor* syntaxes are preferred.
   - Note: there might be other "map-like" types that accept keys of other types, but there won't be specific "sugar" for them.

Literal constructor rules:

- A literal constructor receives exactly one token stream argument (raw, not pre-evaluated).
- It may use a dedicated parser, but must return either:
  - a valid value, or
  - a deterministic syntax error with source location.
- Literal constructors are pure (no side effects).
- Short literals are reader-level syntax sugar that desugar to constructor calls before macro expansion.

Token representation:

- `&:binding.asString`: canonical textual representation of the lexical token itself.
- `:binding.asBoundString`: contextual string generated by the current binding (engine-specific).

#### Number Literals

```text
[+-]?9+[(\.)?9+]?  # number literal, pseudo-regex
```

Depending on the input, the parsed type may be `int*`, `flo*`, or `number`.


#### String and List Literals

```iklo

#
### string
#

"roger senna"           

# interpolation
`"~:val ~$var _:lst"`   

# multiline
"""
hey
ho """                  

#
### lists
#

l%{ a b c d } == [ a b c d ] == (list 'a 'b 'c 'd )

#
### vectors
#

vec%{ a b c d } == { a b c d } == (vector 'a 'b 'c 'd)

#
### options
#

# the option `-foo`
-foo == option%{ foo } == option 'foo

# the option `-foo = bar`
-foo = bar == foo -> bar == option%{ foo bar } == option 'foo 'bar

# the "empty" option
-- == option%{} == (option)

#
### maps
#

%{ a -> 1, b -> 2 } == map a->1 b->2 == map (option 'a 1) (option 'b 2) == map -a 1 -b 2

#
### sets
#

%( a b c d ) == set%{ a b c d } == (set 'a 'b 'c 'd)

#
### stream (potentially infinite literal)
#

%[ a b c d ] == (stream 'a 'b 'c 'd)

# note: full-scan stream comparisons are only allowed for *finite* streams
#       (and even those should generally be avoided)

# quote-syntax
`[ a ~:b _:c d ] == quote%{ a ~:b _:c d }      



#
### Assignments
#

# untyped val assignment
let :record be [ :a :b :c :d ]

# typed val assignment
let ^bool :x be -true

# note: you may put the left-side declaration between parentheses,
#       in case that helps diferentiating "typed val assignments" from "type assignments"
let (^bool :y) be -false



#
### Algebraic Data Types - Type Assignments
#

# Note: **Everything is a value**
#       So there's no separate token for type assignment (type declaration + definition).
#       Meaning `let` can *always* be used to bind an expression <expr> to a token.
#       Regardless if <expr> is a regular value, a function, a type, or something else entirely...

# simple enum types - similar to sets
let ^bool be %d{ -true, -false }

# equivalent values, with different type-checking behavior:
let :x be -true       # a "free" -true should always be parsed as boolean?
let ^bool :x be -true
let :x be ^bool -true

# record-like enum types - similar to maps
let ^maybe be d%{ -left = :value, -right = :value }

# default construction happens by applying bound type as a function:
let :val be (^maybe "value1")
let (^maybe :val) be ^maybe "value2"

# simple recort types use slots, not options
let ^my-record be d%{ :field1 = :value, :field2 = :value, :field3 = d% { -true, -false, -unknown = :unknown-value } }

# describe returns generated constructor + field metadata
let (^my-record :my-record) be ^my-record "my-value" "my-unknown-value"
let :my-record be ^my-record :value = "my-value", :unknown-value = "my-unknown-value" # named arguments syntax

# named arguments are valid for any form that declares named slots in its interface

# graph transaction semantics:
# 1) top-level `let` on graph bindings runs in an implicit transaction
# 2) nested graph updates require explicit `graph.begin` ... `graph.commit`
# 3) on uncaught error, graph transaction always rolls back
# 4) macros can emit graph transactions, but cannot commit a transaction they did not open

```

### Built-in Operators

| Operator                | Sugar    | Description                                                                                               |
| ----------------------- | -------- | --------------------------------------------------------------------------------------------------------- |
| *newline*               |          | Line-breaks terminate an expression iff a) it is valid and b) the next line cannot continue it.           |
| `%`                     |          | Dispatch operator: operator implementations; bindings for tokens; basic literal syntax                    |
| `%deref expr`           | `*expr`  | Deref operator; realizes protected or promise-based values such as futures or laziness.                   |
| `%ref expr`             | `&expr`  | Ref operator; references the token itself. Required when working with the default (form) binding.         |
| `+`, `-`                |          | Unary signal operator.                                                                                    |
| `+`, `-`, `*`, `/`, `^` |          | Binary arithmetic operators.                                                                              |
| `( ... )`               |          | Par-expr. Overloaded as grouping expression, function apply, and part of some literals syntax.            |
| `;`                     |          | Strict line-break. Forces the end of the current line expression and requires the expression to be valid. |
| `,`                     |          | Expression separator. Forces the end of the last bottom expression.                                       |

#### Notes:

- operators *may* be separated from neighbour tokens by spaces/line-breaks (or not)
- space itself is *not* a valid operator character though (unless escaped)

So

  - Valid: `1**2 == 1 ** 2` is `-true`
  - Invalid: `1 * * 2` is a syntax error

#### Par-expr behavior

The rough algorithm is:

```text
( tokens ) = if tokens[0] is a known form, then (token[0] (rest tokens))
             else if tokens is valid expression, then (eval tokens)
             else (stream tokens)
```

#### Comma and Semicolons

##### `,` divides the stream into two parts:

  - Inside a line, or inside most literals
  - Exception: `map` short or construct literals

If the left or right part contains 0 tokens:

  - Then that part is ignored.

If the left or right part contains 1 single token

  - Then that item is used directly.

If the left of right part has 2+ items:

- The parser checks whether *part* is a valid expression:
  - `true`: the whole (evaluated?) expression is returned.
  - `false`: the parser returns a stream of those items.

> [!WARN]
>
> Adding commas to an expression might cause unexpected consequences.
>
> Comma is **very magic**: it should generally be *avoided*,
>
> But they might sometimes be useful:
>
> - When you are prototyping, and later you will replace them with parentheses.
> - When you **really** know what you are doing.
>
> Exception: **map literals** (here they are basically white space.)

##### `;` acts like a more strict line-break

`;` at the end of a line is **not** required.
But semicolons *may* generally be used without concern, if that makes the code clearer.

#### Examples:

Operator examples:

```iklo
# deref and ref
let :future be async (compute-heavy)
* :future
&some-form

# strict break and separators
form-a 1; form-b 2
map %{ -a -> 1, -b -> 2 }

# unary and binary
-1 + +2
2 * 3 ^ 2

# explicit grouping
(1 + 2) * 3

(form :a :b :n)     # like lisp
form (:a :b :n)     # same as (form (stream :a :b :n))
form (:a, :b, :n)   # also (form (stream :a :b :n))
form (:a :b, :n)    # (form (stream (stream :a :b)) :n)
form [:a :b, :n]    # (form [[:a :b], :n])

[ 1, 2, 3, 4 ]  # same as [ 1 2 3 4 ]
[ 1 2 3 4, 5 ]  # same as [ [ 1 2 3 4 ] 5 ]
[ 1 2 3, 4 5 ]  # same as [ [ 1 2 3 ] [ 4 5 ] ]
[ 1 + 2, 3 ]    # same as [ (1 + 2) 3 ] == [ 3 3 ]
[ 1 + 2 + 3 ]   # no expression resolution, same as [ '1 '+ '2 '+ '3 ] => 5 itens
[ 1 + 2 + 3, ]  # same as [ ( 1 + 2 + 3 ) ] == [ 6 ]

[ 1; 2; 3; 4 ]  # syntax-error, `[ 1` is **not** valid
[ 1 2 3; 4 ]    # syntax-error, `[ 1 2 3` is **not** a valid expression
```



## Bindings

| Binding      | Sugar    | Engine       | Description                                                                                                |
| ------------ | -------- | ------------ | ---------------------------------------------------------------------------------------------------------- |
|  `gra%token` | `^token` | graph        | **Transactional Memory binding**: modify *many* related identities, as a *single* transaction, at the *same* time. Internally used for *types*, *patterns*/*interfaces*, and *computations*. |
|   `fm%token` |  `token` | graph        | **Form binding**: associates a token with a *form* (*function* or *macro*).                               |
|   `if%token` |          | graph        | **Interface binding**: describes the *signature* or *interface* for the *form* and *computation* bindings. |
|   `cp%token` |          | graph        | **Computation binding**: describes the *compute*/*body* for *form* and *interface* bindings.              |
|  `key%token` | `~token` | static       | **Keyword** or **Option binding**: *self-bound*, *global*, and *static*. **Cannot be rebound**.          |
|  `val%token` | `:token` | lexical      | **Lexical binding**: *Usually immutable* in iklo. Used for *locals*, *function arguments*, etc.          |
|  `var%token` | `$token` | dynamic      | **Variable binding**: *thread-local* identities with a *shared default*. Like clojure **vars**.          |
|   `rx%token` |          | reactive     | **Reactive binding**: *event-sourced*, *reactive* binding. Like Clojure **agents**.                       |
| `sync%token` |          | synchronised | **Entity binding**: *synchronous* and *uncoordinated*. Like Clojure **atoms**.                            |

### Notes

- *Lexical* vs. *dynamic* scope:
  - **dynamic**: *mutable global with quirks* (e.g. they can be temporarily overriden).
  - **lexical**: available inside any lexical block; scope used by most modern language when resolving variable references.
- *Concurrent* vs. *non-concurrent*:
  - **Non-concurrent binding** disallows simultaneous access from distinct threads.
  - **Concurrent binding** allows concurrent access, It must be either:
    - **Synchronous** or **Asynchronous**.
    - **Coordinated** or **Uncoordinated**.
- `graph`, `dynamic`, `reactive` and `synchronized` are always mutable, by definition.
- `lexical` values are *usually* constant, but can be declared mutable with `set`.



## Assignment

```iklo
# assign an expression to some binding
let <bound-token> be <expression>    
```



## Expression Parsing

```text
let :x be 1 + 2 * 3 ^ 4 * 5 + 6

=> 1 +(2 * ((3 ^ 4) * 5) + 6)
```

Expression parser definition:

- `2 * 3` is always syntactic sugar for `(* 2 3)`.
- Parsing uses Pratt precedence climbing (not parenthesis insertion rewrite).
- Operator declaration must include:
  - precedence number,
  - associativity (`left` or `right`),
  - fixity (`prefix`, `infix`, `postfix`),
  - implementation form.
- Built-in precedence (high to low):
  1. postfix calls / indexing
  2. prefix unary (`+`, `-`, `%ref`, `%deref`)
  3. exponentiation `^` (right-associative)
  4. multiplicative `*` `/`
  5. additive `+` `-`
  6. separators `,` and `;` (statement-level, not arithmetic)

### Form application

Forms apply left to right, eagerly consuming arguments and expressions.

```text
form :a :b :c ... :n
form :a :b + 1 :c       # same as (form :a (:b + 1) :c)
form :a :b + 1 + :c     # same as (form :a (:b + 1 + :c))
```

### Examples

```iklo
form :a :b :c ... :n 
```

- `form` applies the form.
- both arguments AND expressions are eagerly taken.
- `form` takes as many arguments as it can.
- each argument must be a valid expression
- each expression *also* takes as much tokens as it can

```iklo
# valid parsing could be either `(form-a 1 'form-b 2)` or `(form-a 1) (form-b 2)`.
# depends on forms pattern and arity
form-a 1 form-b 2`

# should be avoided
# behaves like `(stream (form-a 1) (form-b 2))`, which can be surprising.
form-a 1, form-b 2

# two separate expressions, like `(form-a 1) (form-b 2)`.
# if `form-a 1` returns a value, it will be lost.
form-a 1; form-b 2

# 3 arguments - same as any Lisp:
(form :a :b :n)     

# 3 arguments - same as `(form :a (:b + 1) :c)`
form :a :b + 1 :c

# 2 arguments - same as `(form :a (:b + 1 + :c))`
form :a :b + 1 + :c     

# 1 single argument (probably not what most would expect) - same as `(form (stream :a :b :n))`:
form (:a :b :n)     
form (:a, :b, :n)   

# 1 argument - same as `(form (stream (stream :a :b)) :n)`:
form (:a :b, :n)    

# 1 argument - same as `(form [[:a :b], :n]):
form [:a :b, :n]    

# valid only if form interfaces can consume it unambiguously.
# otherwise parse error: "ambiguous form chain without separator".
form1 -a form2 form3 -b form4 -c -d form5
```

## Self-interpreting forms (reader + parser contract)

Iklo supports forms with "natural language-like" call shapes by splitting parse into two steps:

1. Reader emits a flat stream of tokens/literals/operators with source spans.
2. Parser asks candidate head forms to parse themselves via their interface grammar.

Each form exposes a parse contract:

- **head tokens**: tokens that can start this form.
- **slot grammar**: required/optional arguments and marker tokens between arguments.
- **lookahead bound**: max tokens parser may inspect for this form.
- **fallback**: whether parse failure is hard error or soft miss.

Example:

```iklo
copy :src to :dst if-missing
```

Could be defined as:

```text
form copy:
  slots: <value> "to" <value> ["if-missing"]
```

This is how tokens "interpret themselves" without ambiguous free-form parsing.

## Macro system

- Macros operate on syntax objects (token + source + hygiene context), not raw strings.
- Expansion is iterative until fixed point or `max-expansion-depth` is reached.
- Hygiene is default; explicit capture requires `capture` form.
- Macro expansion happens after literal desugaring and before type checking.

## Optional typing model

- Untyped bindings are allowed everywhere.
- Typed bindings (`^type`) enforce runtime checks by default.
- Module-level `@type-mode static` enables compile-time checks for resolvable forms.
- Type errors include source span and expected/actual type descriptions.

## IKVM (Iklo Virtual Machine) specification

### Core idea

Iklo code targets **IKVM bytecode**, and IKVM executes it on a register + stack hybrid machine:

- Registers for local fast values.
- Operand stack for expression composition.
- Heap for persistent values, closures, streams, and graph bindings.

### Runtime objects

- Immediate: small ints, booleans, nil, chars.
- Heap: strings, lists, vectors, maps, sets, streams, closures, syntax objects, type descriptors.
- References: lexical cell, dynamic var cell, graph node, sync/reactive handles.

### Instruction families

- `LOAD_*`, `STORE_*` for values and bindings.
- `CALL_FORM`, `CALL_NATIVE`, `TAIL_CALL`.
- `JUMP`, `JUMP_IF`, `MATCH_TYPE`.
- `MAKE_*` for literals and ADT values.
- `TX_BEGIN`, `TX_COMMIT`, `TX_ROLLBACK` for graph transactions.
- `MACRO_EXPAND` for compile-time pipeline steps.

### VM guarantees

- Deterministic evaluation order (left-to-right).
- Graph transaction atomicity.
- Tail-call optimization for self and mutual recursion.
- Precise source mapping for runtime errors.

## Feasibility proof (language and implementation)

### Language feasibility

Iklo is feasible because each "risky" feature is bounded by explicit contracts:

- Self-interpreting forms are bounded by interface grammars + lookahead limits.
- Comma semantics are constrained by strict mode.
- Optional typing is layered (runtime first, static where possible).
- Macro power is constrained by hygiene and expansion depth limits.

### Implementation feasibility

A minimal implementation is achievable in stages:

1. Reader + Pratt parser + core forms.
2. AST interpreter with bindings and transactions.
3. Bytecode compiler targeting IKVM.
4. IKVM runtime with FFI and shell integration.
5. Static checker and macro tooling.

If stage 2 works, the language is viable. If stage 3+4 run the same test corpus as stage 2, the VM implementation is proven viable.

## Laziness and effects (practical model)

Iklo uses a two-lane execution model:

- **Pure lane**: values and pure forms may be lazy.
- **Effect lane**: side-effects are strict and explicitly sequenced.

### Lazy semantics

- Lazy values are represented as thunks: `(code, env, state)`.
- Thunk states: `new`, `running`, `forced(value)`, `failed(error)`.
- Forcing uses call-by-need (memoized result).
- Re-entrant force of `running` thunk is a runtime error (`cyclic-force`).

### Effect semantics

- Effectful forms return `Action<T>` values (descriptions, not immediate execution).
- Building or passing an `Action<T>` is pure.
- Effects execute only in strict effect boundaries:
  - top-level program runner,
  - `run <action>`,
  - `do ...` block,
  - `;` sequencing when expressions are `Action`.

### Synchronous side-effects

- `do` executes actions synchronously in source order.
- If an action fails, subsequent actions in the same `do` are not executed.
- `then` chains actions sequentially and passes value forward.
- Forcing a thunk must not execute hidden effects. If thunk value is an `Action<T>`, execution still requires `run`/`do`.

### Interface-level strictness

Forms declare strictness per slot:

- `strict :x` => argument evaluated before call.
- `lazy :x` => argument passed as thunk.
- Default slot mode is `strict` unless form opts in to lazy parameters.

### Minimal syntax sketch

```iklo
let :x be delay (1 + 2)
force :x

let :copy be shell "cp a b"         # Action<int>
run :copy                           # executes now

do
  shell "echo start"
  shell "cp a b"
  shell "echo done"
```

### Design constraints

- No implicit side-effect execution during ordinary expression evaluation.
- Deterministic effect order for all synchronous actions.
- Lazy evaluation is an optimization/control feature for pure compute, not an effect scheduler.

## Implementation plan: laziness without losing practicality

1. **Phase 1 — explicit laziness**
   - Add `delay`/`force`, thunk runtime object, memoization.
   - Keep all forms strict by default.
   - Exit criterion: deterministic thunk behavior + cycle detection.

2. **Phase 2 — explicit effect values**
   - Add `Action<T>`, `run`, `do`, `then`.
   - Port shell/file/network built-ins to return `Action`.
   - Exit criterion: all side-effects occur only through strict boundaries.

3. **Phase 3 — per-slot laziness**
   - Add interface slot modes (`strict`, `lazy`).
   - Parser + call machinery create thunks automatically for lazy slots.
   - Exit criterion: selected forms get call-by-need without global semantic breakage.

4. **Phase 4 — VM support**
   - IKVM instructions for thunk create/force and action run/sequence.
   - Source-map tracing for force and effect boundaries.
   - Exit criterion: bytecode runtime matches interpreter behavior.

5. **Phase 5 — static guidance**
   - Add diagnostics for accidental force hotspots and unsafe effect mixing.
   - Optional strictness/type hints per module.
   - Exit criterion: users can tune performance predictably.

## Transactional IKVM and live image runtime

Iklo language semantics are transactional, so IKVM must be transactional too.

### Runtime as a persistent image

- REPL/eval is image-based: once code is read and committed, it becomes part of the live runtime image.
- The image contains bindings, compiled code, macro definitions, type descriptors, annotation graph, and metadata.
- Image snapshots can be saved/restored; failed transactions never mutate committed image state.

### Unified phase model (read/expand/compile/run)

Iklo does not enforce a hard wall between compile time and run time. Instead:

- reader, macro-expander, compiler, and evaluator all run against the same runtime image,
- phase-specific APIs are capability-scoped (what can be observed/modified), not process-separated,
- compile-time products (expanded syntax, inferred interfaces, bytecode) are first-class runtime values.

### Transaction contract

- Every top-level eval runs inside an implicit transaction.
- Explicit transaction forms are available for multi-step updates:
  - `tx.begin`, `tx.commit`, `tx.rollback`, `tx.retry`.
- Commits are atomic across all binding engines (`graph`, `lexical`, `dynamic`, `reactive`, `sync`).
- Reader/expander/compile side effects must participate in the same transaction boundary as runtime updates.

### Binding engines as runtime services

- Each binding engine is implemented as a VM runtime service with a shared transaction manager.
- Engine operations publish deterministic events for introspection/debugging.
- Cross-engine invariants are validated at commit time.

### Annotation model

Annotations are runtime objects, not syntax-only metadata:

- An annotation is a first-class entity with identity, payload, provenance, and target references.
- Targets may include forms, interfaces, arguments, types, modules, tokens, or runtime systems.
- Annotation resolution is query-based (`find annotations where target = ...`), enabling translation of concepts from other languages without copying their implementation model.

### Runtime data as database substrate

Because committed runtime state is transactional and queryable, Iklo runtime can back NoSQL-like workloads:

- graph/document-style structures,
- versioned snapshots,
- transaction-scoped updates with rollback,
- reactive projections from committed events.

This is an intentional capability, not a side effect of implementation.
