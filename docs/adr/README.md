# Architecture Decision Records

This directory holds the Architecture Decision Records (ADRs) for sitehub-backend. Each file captures **one architectural decision at the time it was made**: the context, the decision, the alternatives weighed, and the consequences accepted.

All ADRs are in `docs/adr/` as numbered markdown files (e.g., `0001-single-database-multi-tenancy.md`).

## Format

We follow the **Nygard ADR** template (`0000-template.md`). One decision per file, numbered with a 4-digit zero-padded prefix.

## Conventions

- ADRs are **immutable**. If a decision is reversed or refined, write a new ADR that supersedes the old one; do not edit the original.
- Update the `Status` field on a superseded ADR to `Superseded by ADR-XXXX`.
- Keep each ADR to roughly one page. If a decision needs more than that, you may be conflating two decisions.
- Write in the past tense from the moment of decision (`We chose X because Y`), not the future tense.

## Adding a new ADR

1. Copy `0000-template.md` to `NNNN-short-kebab-title.md` with the next number.
2. Fill in the sections.
3. Commit the ADR with the implementing change (or just before it).
