# ADR-0012: Role-based access in admin API, not separate admin surfaces

Date: 2026-05-25
Status: Accepted

## Context

The platform has three levels of administration:

1. **Super Admin** (platform operator) — creates schools, manages tenants, sees all schools.
2. **School Admin** (school director/staff) — manages their school's content, settings, users.
3. **Teacher** (future) — edits their own sections only.

The question was whether to create a separate `sitehub-superadmin-api` package for platform-level operations.

## Decision

Use a single `sitehub-admin-api` package with role-based authorization. All admin levels access `admin.sitehub.bg/v1/...`. The JWT claims include a role, and the authorization layer allows or denies operations based on that role.

## Alternatives considered

- **Separate `sitehub-superadmin-api` package** — would duplicate 90% of admin handlers just to add school-management endpoints. Violates DRY. Adds a package and subdomain for a handful of endpoints.
- **Separate subdomain (superadmin.sitehub.bg)** — unnecessary network boundary for what is fundamentally an authorization concern, not an architectural one.

## Consequences

- **Positive**: one admin API, one deployment, one set of handlers. Roles are enforced at the authorization layer, not the routing layer. Adding new roles (e.g., municipal admin) doesn't require new packages.
- **Costs accepted**: authorization middleware must be correct — a bug could expose super-admin endpoints to school admins. Requires careful testing.
- **Mitigations**: test role-based access in integration tests. Use extractors that fail closed (deny by default, explicitly allow per role).

## Revisit when

If the super-admin surface grows large enough to warrant its own team or deployment cadence. Unlikely for a solo-developer project.
