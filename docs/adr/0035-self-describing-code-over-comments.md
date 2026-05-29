# ADR-0035: Self-describing code over comments

Date: 2026-05-29
Status: Accepted

## Context

Comments in code carry three costs that grow with the codebase:

1. **Visual noise** — every comment is a line a reader has to parse before getting to the logic. Dense comment blocks slow down code review and obscure the structure.
2. **Drift** — comments rot when the code they describe changes. Stale comments are worse than no comments because they mislead the reader.
3. **Token cost** — when code is consumed by language models (Claude Code, copilots, code review tools), every comment is billed tokens. For a project intentionally collaborating with AI tooling, this is a real and recurring cost.

The corresponding benefit of comments is real but narrow: they make sense when *why* is non-obvious (hidden constraints, subtle invariants, deliberate workarounds, behavior that would surprise a reader). Comments that explain *what* the code does duplicate the code itself and add no value when identifiers are well chosen.

## Decision

**Self-describing code is the default. Comments are discouraged but not forbidden.**

Concretely:

1. **Choose names that describe intent.** A function called `verify_token_signature` does not need a comment saying "verifies the token signature." A variable called `shutdown_deadline` does not need a comment saying "this is the shutdown deadline."
2. **Decompose to express structure.** If a block needs a comment to explain what it does, extract it into a function whose name says what it does.
3. **Reach for a comment only when the *why* is non-obvious.** Hidden constraints (RFC requirements, regulatory deadlines), subtle invariants (this lock must be held), surprising workarounds (this exists because library X has bug Y), and deliberate trade-offs (we chose O(n²) here because n ≤ 8 and the constant is tiny) all qualify.
4. **Avoid:**
   - Comments restating what the code does (`// increment counter` next to `counter += 1`).
   - Header banners (`// ------ CONFIG SECTION ------`) — structure should come from modules/functions.
   - Author tags (`// added by Alice on 2024-01-15`) — git blame is authoritative.
   - TODO comments without an owner or date — track them in issues instead.
   - Commented-out code — delete it, git remembers.
5. **Doc comments (`///` and `//!`) follow the same rule with one carve-out:** public API surface that other crates consume may carry doc comments where they document contract, not mechanism.
6. **ADRs and CLAUDE.md remain the right home for project-level reasoning** — that's why these documents exist.

## Alternatives considered

- **Require comments on every public function** — common style guide, but encourages "comment to satisfy the rule" noise. Self-explanatory names give the same benefit at lower cost.
- **Forbid comments entirely** — too strict; some context genuinely cannot fit in identifiers.
- **Comment density target (e.g., 1 line of comment per 5 lines of code)** — measures the wrong thing. Quality of intent matters more than line count.

## Consequences

- **Positive**:
  - Smaller diffs, faster reviews.
  - Lower LLM token cost when sending code to AI tools.
  - Less drift to maintain over time.
  - Forces clearer naming and tighter functions, which improves code quality.
- **Costs accepted**:
  - Some readers expect heavy comments and may find the codebase terse initially.
  - The bar "is the *why* non-obvious?" is a judgement call and can produce disagreement in review.
- **Mitigations**:
  - When a reviewer asks "why does this do X?", the answer goes in a renamed identifier or an extracted function — not a new comment.
  - When a comment really is warranted, lead with **Why:** so the rationale is the load-bearing part.

## Revisit when

- The project grows to a size or contributor base where verbal communication can't fill the gap that absent comments leave.
- A specific class of bug emerges that would have been prevented by a comment convention we currently discourage.
