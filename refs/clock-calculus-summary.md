# The clock calculus — a summary for Iklo

> **Provenance.** Distilled from *"A Type-based Clock Calculus"*, lecture
> slides by **Marc Pouzet** (ENS Paris / Inria), MPRI course, October 2022.
> The source PDF is a reference document kept locally at
> `refs/clock-calculus.pdf` and is **not committed** to this repository
> (copyright — slides, not open content). This file is an original summary
> and analysis, not a reproduction of the slides' text, written to connect
> the concept to Iklo's design (see [§8](#8-relevance-to-iklo) below).
>
> Written 2026-07-20, prompted by [ADR-0003](decisions/ADR-0003-queue-evaluation-exploration.md)
> (queue/stream evaluation exploration), which referenced "clock calculus"
> and "N-synchrony" without a citable summary in the repo. That gap is what
> this file fills.

## 1. The problem: what does it mean for two streams to be "in sync"?

Kahn Process Networks (KPNs) are the classical model of stream computation:
processes communicate through unbounded FIFO buffers, with no notion of
rate or speed at all. KPNs are always deterministic and never need to reject
a program, but they buy that generality by allowing (in principle) unbounded
buffering.

**Synchronous languages** (Lustre, Signal, Esterel, Lucid Synchrone) want
something stronger and cheaper: a network is *synchronous* if it can be
executed **with no buffering mechanism at all** — every signal is defined
according to one global clock (a set of totally ordered instants), and
composition is lock-step, like a synchronous digital circuit where every
gate reads one input and produces one output per cycle.

The catch: not every KPN can be executed this way. **The clock calculus is
the static check that tells you which ones can** — a type system, in the
proper technical sense, where the "types" describe *when* a stream produces
values rather than *what kind* of values it produces.

## 2. What a clock is, formally

For a stream `x` defined over a domain of instants `D`, its equation
`z = x + y` really means `∀t ∈ D. z(t) = x(t) + y(t)` — nothing requires `D`
to be discrete, or even known in advance.

A stream can be **partial**: defined only at *some* instants (its own
sub-domain). The formal device for this is an explicit **absent** value:
extend every type `T` to `T_abs = T + {abs}`, then every stream has an
associated boolean **clock** — a stream of `true`/`false` saying "present at
this instant" / "absent at this instant". Two operators change clocks in
opposite directions:

- **`when`** — samples a stream only when a boolean condition holds, moving
  to a strictly slower **sub-clock**. (`s on e` — "the sub-clock of `s` where
  `e` is true".)
- **`merge`** — takes two streams defined on *complementary* clocks (`c` and
  `not c`) and recombines them onto the union clock. It is `when`'s exact
  dual.

This is the important refinement over the loose "end-of-stream marker" framing
in ADR-0003 §"the open problem": **the mechanism is not a marker for where a
stream ends — it is a per-instant presence bit, carried by every stream, at
every point in time.** That is strictly more general: it composes across many
concurrently-running streams at different rates, not just one stream with one
boundary.

## 3. A small clocked stream language

The calculus is presented over a minimal core:

```
e ::=  e e | let x = e in e | x | i
    |  e fby e         -- register / unit delay ("followed by")
    |  e -> e          -- initialisation ("first e1, then e2")
    |  e when e        -- downsample onto a sub-clock
    |  merge e e e     -- recombine complementary sub-clocks
    |  rec x.e | λx.e
```

`fby` and `->` are Lucid's own primitives (Wadge & Ashcroft) — the same
family already cited in ADR-0003 — so this is not a competing lineage, it is
the *typed* continuation of it. `when`/`merge` are the clock-manipulation
pair that Lucid's untyped `fby` alone doesn't give you.

## 4. Clocks as types

The type system assigns a **clock type** `cl` to every expression:

```
σ  ::=  ∀α1,...,αn. cl              -- clock scheme (ML-style polymorphism)
cl ::=  ∀x:cl.cl | cl × cl | s       -- dependent function, product, or sort
s  ::=  s on e | α                   -- a clock refined by a condition, or a variable
```

Typing is syntax-directed (Const, Var, Op, Abs, App, Rec, Let rules, closely
mirroring Hindley–Milner) with one twist: it is a lightweight **dependent**
type system — a clock `s on e` can depend on the *value* of a boolean stream
`e`, not just on other types. The core primitives get exactly the signatures
you'd expect once you see it:

```
pre    : ∀α. α → α                              -- delay preserves the clock
->     : ∀α. α → α → α                          -- init preserves the clock
when   : ∀α. α → ∀x:α. α on x                   -- downsamples
merge  : ∀α. ∀x:α. (α on x) → (α on not x) → α  -- recombines
```

Any strict binary operator requires **both operands to share the same clock**
— which is the formal version of the informal problem the deck opens with:
`(e1 when c1) + (e2 when c2)` is only synchronous when `c1` and `c2` are
provably the same clock.

## 5. Why this is hard, and why practical systems cheat (on purpose)

Checking clock equality in general is exactly as hard as you'd fear:

| Clock language | Complexity of equality |
|---|---|
| Plain booleans | NP-complete |
| Booleans + registers (state) | PSPACE-complete |
| Unbounded arithmetic | Undecidable |

So every real implementation deliberately gives up completeness for a
**decidable, syntax-directed inference algorithm** — structural equality plus
first-order unification, in the style of ML's Algorithm W — the same
trade-off Hindley–Milner makes for ordinary type inference, and it costs the
same kind of thing: some semantically-equal-but-syntactically-different
programs get rejected, and polymorphism is restricted to prenex (rank-1)
form. This is named directly, not hand-waved, in the source material.

Three named strategies exist in the wild: clock **equality** (structural, in
Lucid Synchrone; boolean-equivalence, in Signal), clock **inference** (Signal,
Lucid Synchrone derive your clocks for you), and clock **verification**
(Lustre — you write clock annotations, the compiler only checks them).

## 6. The payoff: a correctness theorem that erases the "absent" value

**Theorem (Correctness).** A well-clocked program can be executed
synchronously — with the abstract "absent" value removed entirely, replaced
by ordinary guarded/conditional execution at compile time.

Mechanically, this is a type-directed program transformation: every
expression `e : cl` is compiled to an annotated `e'` where each operation
carries an explicit enable bit, threading an abstraction over the clock at
every generalization point and an application at every instantiation point
— machinery, again, structurally identical to how a polymorphic-type
compiler inserts dictionary or type-application code.

One detail worth keeping: **clocks are only operationally useful for
*stateful* primitives** (`pre`, `->`, `fby` — anything with a register).
Purely combinational, stateless code needs *no* runtime clock dispatch at
all once it's been clock-checked; the "is this present?" branching only
survives compilation where there's a delay/register to gate.

## 7. Two escape hatches worth knowing about

- **Clock abstraction (existential types).** A function that samples on a
  condition computed *locally* — not visible to its caller — needs the
  condition packaged as part of an existential/dependent return type:
  `hide : ∀α. α → Σ(o:α). α on o`. Esterel's **valued signal** (a signal
  that carries both a value and a presence bit) is exactly this shape:
  `α sig = Σ(c:α). α on c`.
- **Oversampling.** Lucid Synchrone (1998!) could type a function whose
  *output* clock depends on its own *past* output — a legitimate
  self-referential loop, safe because it only reads earlier instants of
  itself via `fby`, never the current one. Lustre, with its more restrictive
  single-clock-variable scheme (`∀α.cl`, one variable = "the base clock of
  the node"), cannot express this pattern at all.

## 8. Relevance to Iklo

This isn't background reading for its own sake — it lands on several places
Iklo already has open surface area:

- **It is the formal foundation `ADR-0003` was reaching for.** ADR-0003
  registers "queue/stream evaluation" as a research direction and names the
  open problem as "grouping relocates from space to time; needs a
  clock-calculus-like mechanism." This document *is* that mechanism, worked
  out in full, with a citable correctness theorem and known complexity
  bounds — not a hand-wave anymore.
- **`stream` is already a first-class Iklo literal** (`LANGUAGE.md` §Types &
  Literals, `%[ a b c d ]`), explicitly allowed to be infinite, with the
  caveat that "full-scan stream comparisons ... should generally be
  avoided." That caveat is exactly the shape of problem a clock type solves:
  it's the compiler, not the programmer, that should know whether a
  full-scan is even well-defined (finite clock) or dangerous (infinite/lazy
  clock).
- **The shell is already a Kahn network, not yet a synchronous one.**
  `AGENTS.md`'s "one grammar, three faces" claim and the shell's pipe
  composition are KPN-shaped (§1 above) today. If Iklo ever wants to
  statically guarantee a pipeline needs no buffering — or, conversely, to
  *know* when it does and size the buffer — this is the calculus that
  answers it, not a bigger runtime.
- **A live design fork, already visible in Iklo's effect model.** Iklo's
  lazy-thunk states (`new`, `running`, `forced(value)`, `failed(error)`,
  `LANGUAGE.md` §Laziness and effects) are an *ad hoc*, per-value notion of
  "is this ready yet" with no static discipline governing it. The clock
  calculus's presence bit is the same question, generalized and made
  static. Worth an explicit future question: does Iklo want its two-lane
  pure/effect model to eventually absorb a clock-like check, and if so,
  does it want **inference** (compiler derives readiness, more magic, closer
  to Iklo's existing type-layering plan of "runtime first, static where
  possible") or **verification** (programmer states it, compiler checks —
  arguably a better fit for a project that already gates grammar changes
  behind an ADR and prefers `grammar.lalrpop`-as-spec explicitness over
  inference)?
- **The "clocks are only useful for stateful ops" result bounds the cost of
  ever doing this.** If Iklo's `Substrate`/VDBE work (ADR-0001) ever meets
  stream semantics, this result says the compile-time and runtime cost is
  concentrated exactly where Iklo already has state — bindings, `let`/`set`,
  the transactional image — not spread across ordinary pure expressions.
  That is good news for keeping the pure lane cheap.
- **`Σ(c:α).α on c` (Esterel's valued signal) is a ready-made shape** for
  "a stream value with a presence bit" if/when Iklo wants first-class
  optional/sparse streams distinct from the `Option`-style `some`/`none`
  values `LANGUAGE.md` already has for ordinary values.

## 9. Where this sits historically (why it's not a toy)

This calculus first appeared at ICFP'96 and shipped in real industrial
compilers: the first ReLuC compiler (Esterel-Technologies) was based on it,
and its descendant is inside **SCADE 6** — a certified compiler used for
safety-critical avionics and industrial control software, exercised on
programs up to 100,000 lines of code. It has also been mechanically verified
sound inside Coq (Boulmé & Hamon, LPAR'01). Named further-research
extensions worth tracking if Iklo goes deeper here: **periodic clocks**
(Julien Forget's PhD) and **N-synchrony** (Florence Plateau's PhD, POPL'06)
— the latter is explicitly a *relaxed* model that reintroduces *bounded*
buffering for programs that don't type-check as perfectly zero-buffer
synchronous. That is the natural middle ground between pure Kahn (unbounded
buffering, always accepts) and pure synchronous (zero buffering, sometimes
rejects) — worth reading first if the strict calculus ever turns out too
restrictive for something Iklo actually wants to express.

## 10. Sources cited by the deck (for follow-up reading)

- Paul Caspi and Marc Pouzet, *Synchronous Kahn Networks*, ICFP 1996.
- Jean-Louis Colaço and Marc Pouzet, *Clocks as First Class Abstract Types*,
  EMSOFT 2003.
- Sylvain Boulmé and Grégoire Hamon, *Certifying Synchrony for Free*, LPAR
  2001 (Coq formalization).
- Albert Cohen, Marc Duranton, Christine Eisenbeis, Claire Pagetti, Florence
  Plateau, Marc Pouzet, *N-Synchronous Kahn Networks: a Relaxed Model of
  Synchrony for Real-Time Systems*, POPL 2006.
- E. A. Lee and D. G. Messerschmitt, *Static Scheduling of Synchronous Data
  Flow Programs for Digital Signal Processing*, IEEE Trans. on Computers,
  36(2), 1987 (the SDF model referenced as the precursor to Kahn-network
  synchrony).
