---
**Document:** Iklo Language Reference
**Purpose:** Specification of the "Iklo Lisp-Like Machine", documenting language features and implementation status.
**Audience:** Language designers, interpreter implementers, AI agents reasoning about language semantics.
**Status:** Actively maintained. Much of the material below is aspirational (**TBI**/**TBD**/**BET**); see [AGENTS.md](AGENTS.md) for what's actually implemented today.
**Cross-refs:**
- Repo/workflow spec — [spec/SPEC.md](spec/SPEC.md)
- Agent operating guide — [AGENTS.md](AGENTS.md)
- Design decisions — [`spec/decisions/`](spec/decisions/)
- Bugs — filed as [GitHub Issues](https://github.com/rsenna/iklo/issues) (label `bug`); when you notice an unrelated defect mid-task, open an issue instead of losing it.
---
# Iklo Language Reference

*Iklo* is a functional, data and DSL focused programming language.
It is *also* a shell.
*And* an in-process database system – based on [Turso](https://turso.tech/) [GitHub](https://github.com/tursodatabase/turso).
*And* yet-another-attempt of making a Lisp *with fewer parentheses*...

It's meant to be used as quick-but-not-so-dirty way of integrating systems.
A "systems scripting language", it also aims to serve varied areas such as devops, app prototyping,
and agentic integration & implementation.

Iklo also aims to be a fun way of learning about programming!
It is *intentionally* similar to 80's Logo, taking inspiration particularly from UCBLogo.
In fact, sometimes UCBLogo will be used as a *comparison baseline* for Iklo: we will mention in which ways Iklo is
similar to, or different from UCBLogo.

(Iklo even provides its own implementation of Turtle graphics as a loadable package.)

*But Iklo is not Logo.*

(Even though it *could*; after all, Logo was such a *loosely defined language*, that *anything* can be a Logo language.)

In spite of being essentially a dynamically typed language, Iklo allows you to be as "formal" as you want.

For example, you can add (optional) static type declarations, or even compile Iklo scripts into binaries.

Its design was also influenced by other languages, such as Clojure, Lisp, Dylan, Ruby, Elixir, Scala, and Haskell.

Its name comes from *ikke Logo* (Danish for "not Logo"!)


## Glossary:

- *sugar*: same as *syntax sugar*, usually a *shorter or simpler alternative* to a built-in literal, operator or form.
- *token*: the canonical symbolic unit in iklo source/runtime (identifier-like atom), used for bindings and form heads.

- **TBI**: To Be Implemented: means the related feature has not yet been implemented.
- **TBD**: To Be Defined: means the related feature is planned but not yet defined.
- **BET**: means the related feature is neither implementer nor planned, it's still just a possibility or a
  recommendation.

- **Forms** are the basic building blocks of the language. They can either be functions, macros or "special-forms"
  (i.e. what other languages might call "keywords").

- Tokens, bindings, operators, expressions. (TODO)

- Values, forms, patterns, types, comments, compute. (TODO)

## Core / Prologue / Primitive / Built-In

```
#!/usr/bin/env iklo
# comments start with `#`

```

### Types & Literals

| Name         | Description                    | Short Literal              | Literal Constructor            | Form-based Constructor                                 |
|--------------|--------------------------------|----------------------------|--------------------------------|--------------------------------------------------------|
| char         | Character type                 | `\a; \newline`             | `char%{ a }; char%{ newline }` | `char 160; char "a"`                                   |
| int8..128    | Integer types (8 to 128 bits)  | `42`                       | `int16%{ -5 }`                 | `int32 9`                                              |
| flo8-128     | Float types (8 to 128 bits)    | `3.14`                     | `flo64%{ -2.5} `               | `flo32 8`                                              |
| number       | A Big Decimal-like type        | `1_234_313_324`            | `number%{ 3_778_383 }`         | `number 42; number "1_878_797_932_949_898_999_823"`    |
| token        | Token type, used for bindings  | `foo`                      | `token%{ token with spaces }`  | `token 'foo; token "bar"; token " token with spaces "` |
| option       | Global static symbol           | `-foo=bar; foo->bar`       | `option%{ foo bar }`           | `option 'foo 'bar`                                     |
| string       | String type                    | `"hello"`                  | `string%{ world }`             | `string \a \b \c          # "abc"`                     |
| list         | Linked list                    | `[ 1 2 3 ]`                | `list%{ 1 2 3 }`               | `list 1 2 3`                                           |
| syntax-quote | List templates; used in macros | `` `[ a ~:b _:c d ] ``     | `quote%{ a ~:b _:c d }`        | `quote 'a ~:b _:c 'd      # see 1)`                    |
| vector       | Dense vector                   | `<[ 1 2 3 ]>  # see 2)`    | `vector%{ 1 2 3 }`             | `vector 1 2 3`                                         |
| set          | Set of values                  | `{ a b c }`                | `set%{ a b c }`                | `set 'a 'b 'c`                                         |
| map          | Key-value map                  | `{ a->1, -b, c->2 }`       | `map%{ a->1, -b, c->2 }`       | `map -a 1 -b -nil -c 2    # see 3)`                    |
| stream       | Lazy stream                    | `'( a b c ); ( 'a 'b 'c )` | `stream%{ a b c }`             | `stream 'a 'b 'c`                                      |
| data         | Algebraic Data Type            | —                          | `data%{ :a=1, :b, :c=2 }`      | `data 'a 1 'b -nil 'c 2`                               |

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
#       in case that helps differentiating "typed val assignments" from "type assignments"
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

# simple record types use slots, not options
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

| Operator                | Sugar   | Description                                                                                               |
|-------------------------|---------|-----------------------------------------------------------------------------------------------------------|
| *newline*               |         | Line-breaks terminate an expression iff a) it is valid and b) the next line cannot continue it.           |
| `%`                     |         | Dispatch operator: operator implementations; bindings for tokens; basic literal syntax                    |
| `%deref expr`           | `*expr` | Deref operator; realizes protected or promise-based values such as futures or laziness.                   |
| `%ref expr`             | `&expr` | Ref operator; references the token itself. Required when working with the default (form) binding.         |
| `+`, `-`                |         | Unary signal operator.                                                                                    |
| `+`, `-`, `*`, `/`, `^` |         | Binary arithmetic operators.                                                                              |
| `( ... )`               |         | Par-expr. Overloaded as grouping expression, function apply, and part of some literals syntax.            |
| `;`                     |         | Strict line-break. Forces the end of the current line expression and requires the expression to be valid. |
| `,`                     |         | Expression separator. Forces the end of the last bottom expression.                                       |

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

Both commas `,` and semicolons `;` are used to *separate expressions*, but their usage differs slightly.
Still, sometimes both can be used with the same behaviour. Example:

##### `,` exactly like a line-break

- Adding a comma into a stream of tokens *asks* the parser to treat the previous and the next sequence of tokens as 
  complete expressions.
- If both sequences are *valid* expressions, then that's exactly what the parser does.
- If either of them are not valid, but the stream without commas was valid, then the comma is *ignored*.

- Also, if both sequences are part of a parent expression, the parent expression is *always* considered incomplete.
  - So adding a comma *requires* the parent expression to have at least one more complete sub-expression *after*
    the comma.
  - This *also* means that a comma is *never* valid as the *last token in any expression or code block*.

- Also, For `map` values, `,` may specifically be used to separate option pairs.

##### `;` acts as a *stronger* line-break

- Adding a semicolon into a stream of tokens *forces* the parser to treat the previous and next sequence of tokens as
  complete expressions.
- If both sequences are *valid* expressions, then the `;` placement is valid.
- Otherwise, adding `;` is a syntax error.

- Like commas, if both sequences are part of a parent expression, the parent expression is *always* considered 
  incomplete.
  - So adding a semicolon *requires* the parent expression to have at least one more complete sub-expression *after*
      the semicolon.
  - This *also* means that a comma is *never* valid as the *last token in any expression or code block*.

- *Also*, `;` has an extra function for `do` expressions, being required at the end of lines

##### Notes:

- One *could* add `;` at the end of every line in a lexical scope - *except the last line*.
- But `;` at the end of a line is **neither** required nor a recommended Iklo style.

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

| Binding      | Sugar    | Engine       | Description                                                                                                                                                                                  |
|--------------|----------|--------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `gra%token`  | `^token` | graph        | **Transactional Memory binding**: modify *many* related identities, as a *single* transaction, at the *same* time. Internally used for *types*, *patterns*/*interfaces*, and *computations*. |
| `fm%token`   | `token`  | graph        | **Form binding**: associates a token with a *form* (*function* or *macro*).                                                                                                                  |
| `if%token`   |          | graph        | **Interface binding**: describes the *signature* or *interface* for the *form* and *computation* bindings.                                                                                   |
| `cp%token`   |          | graph        | **Computation binding**: describes the *compute*/*body* for *form* and *interface* bindings.                                                                                                 |
| `key%token`  | `~token` | static       | **Keyword** or **Option binding**: *self-bound*, *global*, and *static*. **Cannot be rebound**.                                                                                              |
| `val%token`  | `:token` | lexical      | **Lexical binding**: *Usually immutable* in iklo. Used for *locals*, *function arguments*, etc.                                                                                              |
| `var%token`  | `$token` | dynamic      | **Variable binding**: *thread-local* identities with a *shared default*. Like clojure **vars**.                                                                                              |
| `rx%token`   |          | reactive     | **Reactive binding**: *event-sourced*, *reactive* binding. Like Clojure **agents**.                                                                                                          |
| `sync%token` |          | synchronised | **Entity binding**: *synchronous* and *uncoordinated*. Like Clojure **atoms**.                                                                                                               |

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
- Parsing uses Pratt precedence climbing.
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

Is defined internally as:

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

## More Examples

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

Note: `do ... end` enforcement was borrowed from Elixir.

## VDBE (Iklo Virtual Machine) specification

> **Status (2025):** design-only. Iklo currently runs on the tree-walking
> interpreter in `crates/iklo-runtime`. Adopting VDBE as a compilation target
> is deferred pending an `ImageStore` capability boundary that keeps semantics
> reversible — see [ADR-0001](spec/decisions/ADR-0001-turso-vdbe-image-store.md).

- [ ] TODO: Let's use **VDBE**
  - [If it is good enough for Doom](https://github.com/tursodatabase/turso-vdbe-doom-example), then it's good enough 
    for Iklo.

### Core idea

Iklo code targets **VDBE bytecode**, and VDBE executes it on a register + stack hybrid machine:

- Registers for local fast values.
- Operand stack for expression composition.
- Heap for persistent values, closures, streams, and graph bindings.

### VM guarantees

- Deterministic evaluation order (left-to-right).
- Graph transaction atomicity.
- Tail-call optimisation for self and mutual recursion. TODO: is it available in VDBE?
- Precise source mapping for runtime errors.

## Feasibility proof (language and implementation)

### Language feasibility

Iklo is possible because explicit contracts bound each "risky" feature:

- Self-interpreting forms are bounded by interface grammars plus lookahead limits.
- Optional typing is layered (runtime first, static where possible).
- Macro power is constrained by hygiene and expansion depth limits.

### Implementation feasibility

A minimal implementation is achievable in stages:

1. Reader + Pratt parser + core forms.
2. AST interpreter with bindings and transactions.
3. Bytecode compiler targeting VDBE.
4. VDBE runtime with FFI and shell integration.
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

- Effectful forms return `^action ^t` values (descriptions, not immediate execution).
- Building or passing an `^action ^t` is pure.
- Effects execute only in strict effect boundaries:
  - top-level program runner,
  - `run <action>`,
  - `do ...` block,
  - `;` sequencing when expressions are `^action`.

### Synchronous side-effects

- `do` executes actions synchronously in source order.
- If an action fails, later actions in the same `do` are not executed.
- `then` chains actions sequentially and passes value forward.
- Forcing a thunk must not execute hidden effects. If thunk value is an `^action ^t `, execution still requires `run`/`do`.

### Interface-level strictness

Forms declare strictness per slot:

- `strict :x` => argument evaluated before call.
- `lazy :x` => argument passed as thunk.
- Default slot mode is `strict` unless form opts in to lazy parameters.

### Minimal syntax sketch

- [ ] formalise that, in shell mode, unbounded forms are evaluated executable calls
  - So in `(vim start)`, `vim` must be either a known form or an executable call.

```iklo
let :x be lazy 1 + 2
strict :x

let :copy be cp a b         # ^action ^int 
run :copy                   # executes now

do
  echo start
  cp a b
  echo done
```

### Design constraints

- No implicit side-effect execution during ordinary expression evaluation.
- Deterministic effect order for all synchronous actions.
- Lazy evaluation is an optimisation/control feature for pure compute, not an effect scheduler.

## Implementation plan: laziness without losing practicality

1. **Phase 1 — explicit laziness**
   - Add `lazy`/`strict`, thunk runtime object, memoization.
   - Keep all forms strict by default.
   - Exit criterion: deterministic thunk behavior + cycle detection.

2. **Phase 2 — explicit effect values**
   - Add `^action ^t`, `run`, `do`, `then`.
   - Port shell/file/network built-ins to return `^action ^t`.
   - Exit criterion: all side-effects occur only through strict boundaries.

3. **Phase 3 — per-slot laziness**
   - Add interface slot modes (`strict`, `lazy`).
   - Parser + call machinery create thunks automatically for lazy slots.
   - Exit criterion: selected forms get call-by-need without global semantic breakage.

4. **Phase 4 — VM support**
   - VDBE instructions for thunk create/force and action run/sequence.
   - Source-map tracing for force and effect boundaries.
   - Exit criterion: bytecode runtime matches interpreter behavior.

5. **Phase 5 — static guidance**
   - Add diagnostics for accidental force hotspots and unsafe effect mixing.
   - Optional strictness/type hints per module.
   - Exit criterion: users can tune performance predictably.

## Transactional VDBE and live image runtime

Iklo language semantics are transactional, so VDBE must be (and indeed it is) transactional too.

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
