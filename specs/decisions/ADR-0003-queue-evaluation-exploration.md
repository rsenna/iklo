# ADR-0003 — Queue-based evaluation as an alternative to stack-based VDBE (exploratory)

- **Status:** Proposed — exploratory, not decided. Nothing in this ADR changes
  current behaviour or authorises implementation work.
- **Date:** 2026-07
- **Deciders:** @rsenna (with Claude as sounding board)
- **Supersedes:** —
- **Superseded by:** —

## Decision (one sentence)

**We are not deciding anything yet — this records a design idea worth keeping
on file: evaluate FIFO (queue) discipline as an alternative or complement to
the stack discipline [LANGUAGE.md](../../LANGUAGE.md)'s VDBE section currently assumes, because
Iklo already has three places (VDBE, the shell, `stream` literals) where a
queue-shaped machine might unify concepts that are currently separate.**

This is a research note, not a commitment. See [Constitution §VI](../../.specify/memory/constitution.md): lightweight
`Decision (date)` notes are fine until they harden; this one hasn't.

## Context

A conversation about how Forth is constructed ended up asking: Forth's data
stack is the *runtime* discipline that lets it delete the *parser* — postfix
notation is the human-writable linearisation of a stack machine's execution
order. What if the runtime discipline were a queue instead?

The core finding: **stack code is the post-order traversal of the expression
tree; queue code is the level-order (breadth-first) traversal.** A stack
machine finishes each subtree before moving to the next; a queue machine
executes the tree in generations, with every operand of the current generation
sitting in the queue simultaneously. This is not exotic — it is the classical
"queue processor" architecture explored in the dataflow-machine literature
(Preiss et al.) as a way to expose instruction-level parallelism without
register renaming, since independent operations end up adjacent in the queue
rather than buried under stack depth.

Level-order traversal is not hand-writable at depth (sibling subtrees
interleave across generations), so a queue machine *requires* a real compiler
in front of it — which Iklo already has (`grammar.lalrpop` → `Spanned<Expr>`).
That is: the objection that sank pure queue-Forth ("you'd need a parser") is
not an objection for Iklo.

This connects to three places in Iklo — two of them real today, one still
hypothetical:

1. **A future pipe-based shell would be a Kahn network — but that alone
   wouldn't prove the point.** Iklo has no shell pipeline today: `iklo-lexer`
   has no pipe lexeme, and `iklo-cli` only implements the REPL and file
   runner (`AGENTS.md`, "What is actually implemented today"). If a
   Unix-style `cat | grep | sort` pipeline is eventually built, FIFO
   transport between processes makes it a Kahn network by construction —
   but that is process-level IPC, not evidence that *expression evaluation*
   inside Iklo uses queue/level-order discipline. Treat this as a plausible
   future direction, not existing support for the thesis.
2. **`stream` is already a first-class literal** (`LANGUAGE.md` §Types &
   Literals, `%[ a b c d ]` / `(stream 'a 'b 'c 'd)`), explicitly allowed to be
   infinite, with lazy semantics already specified elsewhere in the language
   (thunks, `forced`/`failed` states, call-by-need).
3. **VDBE is currently specified as a register + stack hybrid** ([LANGUAGE.md](../../LANGUAGE.md)
   §VDBE, "Operand stack for expression composition"). That commitment
   predates this exploration and is **not changed by it** — see Non-decisions
   below.

The unifying move (borrowed from Lucid, Wadge & Ashcroft): **treat a scalar as
a singleton stream.** Then `1 + 2` is `zip-with(+)` over two one-element
streams, a shell pipeline stage is the same operator over longer streams, and
"language mode" vs. "shell mode" stop being two semantics needing two
evaluators — they become one machine at two stream lengths. That is a
candidate mechanism for the "one grammar, three faces" claim in [AGENTS.md](../../AGENTS.md),
not yet a design.

Recursion in a pure FIFO model doesn't use call/return (that's intrinsically
LIFO — hence Forth's *separate* return stack even in an otherwise stack-pure
design); a "call" becomes enqueuing onto the callee's input stream, "return"
is the callee's output stream, and recursion is a feedback edge. This is
exactly **Kahn process networks** — deterministic dataflow with a well-studied
formal theory, not a novelty.

### The open problem: grouping doesn't disappear, it relocates

Pointwise stream operators (`+`, `map`) consume fixed-size prefixes and need
no grouping. But any operator that must consume *a whole stream* (`sum`,
`sort`, `reverse`) needs to know where the stream **ends** — an
end-of-stream marker, typed rates, or delimited segments. **Lisp's
parentheses group in space; streams group in time — the bookkeeping is
conserved, not eliminated.** Synchronous-dataflow languages (Lustre and kin)
formalise this as a *clock calculus*: which words may consume how much of
which stream, playing the same role Forth's `( a b -- c )` stack-effect
comment plays for a stack machine. Iklo would need an equivalent — plausibly
something the `stream` literal's existing "full-scan stream comparisons ...
should generally be avoided" caveat is already circling.

See [`refs/clock-calculus-summary.md`](../../refs/clock-calculus-summary.md)
for the worked-out version of this, with a correction to the framing above:
the actual mechanism is not an end-of-stream marker but a **per-instant
presence bit** carried by every stream — strictly more general, since it
composes across many concurrently-running streams at different rates rather
than describing one stream's boundary. That file also covers the correctness
theorem, the decidability trade-offs real implementations make, and a
dedicated Iklo-relevance section.

## Non-decisions (explicit)

To keep this from being mistaken for a plan:

- **[LANGUAGE.md](../../LANGUAGE.md)'s VDBE section is unchanged.** It still specifies a
  register + stack hybrid machine. This ADR does not propose replacing it,
  only flags that a queue/stream model is a real alternative worth
  evaluating *if* VDBE work ever starts — which [ADR-0001](ADR-0001-substrate-boundary.md)
  gates behind the full sequence it already commits to: the `Substrate`
  epic shipping and being exercised, *then* a Turso-backed `Substrate`
  landing and proving out, *then* VDBE-as-compilation-target earning its
  own separate ADR. None of that sequence is shortened or reordered here.
- **No dependency, crate, or grammar change is authorised by this ADR.**
- **The shell/language unification (scalar = singleton stream) is a hypothesis,
  not a design.** It has not been checked against Iklo's actual effect model
  (`^action`, strict effect boundaries) or against `let`/`set` semantics.

## Alternatives considered

Not applicable in the usual ADR sense — this note doesn't select an
alternative, it registers one (queue/stream evaluation) alongside the
already-chosen one (stack-based VDBE) for future comparison.

## Consequences of leaving this on record (vs. not writing it down)

- **Positive:** the idea, and the specific mechanism connecting it to
  `stream` literals, the shell, and Kahn networks, is discoverable instead of
  re-derived from scratch next time VDBE or shell-grammar work starts.
- **Negative:** none — this authorises nothing and blocks nothing.

## Open questions (for if/when this gets promoted toward a real decision)

- Does a level-order (queue) VDBE actually pay for its complexity in Iklo's
  case, given VDBE is *also* constrained by whatever Turso exposes (see
  [ADR-0001](ADR-0001-substrate-boundary.md)'s 2026-07 status note) — or is "stack VDBE, queue shell, stream
  literals bridge them at the value level" the more realistic target?
  - Fair to say the machine-model question (queue vs stack VDBE) is not just
    an Iklo research question but a genuinely open one in dataflow-architecture
    literature — expect this to remain unresolved for a while, not because
    Iklo hasn't thought hard enough, but because nobody has closed it.
- What is Iklo's clock calculus (the EOS/grouping mechanism from the "open
  problem" above)? Does it reuse `stream`'s existing lazy/finite distinction?
- Does "scalar = singleton stream" survive contact with the effect lane
  (`^action`, strict effect boundaries) and the `let`/`set` split, or does it
  only work for the pure lane?
- Is a small token-threaded (bytecode) prototype — either stack or queue
  discipline — a cheap way to de-risk this before committing real VDBE effort?
  A ~30-opcode inner interpreter is a fraction of the cost of the full VDBE
  question and would surface literal-pooling, branch-patching, and
  calling-convention lessons directly.

## Follow-ups

- No action required. Revisit when shell-grammar work or VDBE work resumes.
- If either of those efforts wants to build on this, promote the relevant
  parts to a proper "Accepted" ADR (new number) that supersedes the specific
  commitments it changes, per [Constitution §VI](../../.specify/memory/constitution.md).
- Further reading: Lucid (Wadge & Ashcroft — every variable is a stream, `fby`
  for feedback), Kahn process networks, Morrison's *Flow-Based Programming*,
  Preiss et al. on queue-machine dataflow architectures, and — for the
  grouping problem specifically —
  [`refs/clock-calculus-summary.md`](../../refs/clock-calculus-summary.md),
  which also points to N-synchrony (Plateau, POPL'06) as the bounded-buffer
  middle ground if the strict zero-buffering calculus proves too restrictive.
