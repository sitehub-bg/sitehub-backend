# ADR-0006: `Membership(user, school, role)` relation instead of `school_id` on `User`

Date: 2026-05-20
Status: Accepted

## Context

In phase 1, every admin user manages exactly one school. The simplest data
model would put `school_id` directly on the `User` entity. However, two
plausible future requirements would invalidate that model:

1. A user managing multiple schools (e.g., a person who works as the
   secretary at two schools, or a district-level account).
2. Differentiated roles within a school (principal, teacher, secretary,
   read-only auditor).

Refactoring from `school_id`-on-`User` to a relation later requires touching
every query, every JWT claim, every authorization check, and the database
schema. Adopting the relational model upfront is cheap; retrofitting it is
expensive.

## Decision

Model school affiliation as a separate **`Membership(user_id, school_id, role)`**
entity from day one.

- `User` has no `school_id` field.
- A membership join links one user to one school with one role.
- Phase 1 invariant: at most one `Membership` per `user_id`. This is
  enforced at the application layer (validated on creation) but the schema
  permits more, so lifting the invariant later requires no migration.
- JWT claims include the *current* `school_id` and `role` from the active
  membership; the auth layer is the single place that resolves "which
  school is this user acting on behalf of."

## Alternatives considered

- **`school_id` directly on `User`.** Rejected — locks the data model to a
  single-school assumption that is plausibly wrong within 12 months. The
  refactor cost outweighs the savings.
- **Single embedded membership inside `User`.** Marginal improvement over
  `school_id`; still couples user identity to school affiliation. Rejected.
- **Separate `User` types per role.** Rejected — explodes the type space
  and the same person can have different roles at different schools.

## Consequences

- **Positive**: multi-school support is free when needed (lift the
  application-layer invariant). Role information is in one place. The
  audit trail of "who acted as what role at which school" is natural.
- **Costs accepted**: every "load the user's school" path goes through one
  extra entity (`Membership`) today, for a feature we don't yet use. Slightly
  more verbose queries.
- **Mitigations**: the join is constant-time (indexed by `user_id`), and
  the JWT carries the resolved membership so most request paths don't
  re-fetch it. A `Resolver` in `sitehub-app` centralizes the
  "user → active membership" lookup.

## Revisit when

- We decide to commit to single-school-per-user permanently (unlikely).
- Membership grows additional fields large enough to justify splitting
  (e.g., per-membership preferences, custom permissions).
