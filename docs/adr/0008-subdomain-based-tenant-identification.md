# ADR-0008: Subdomain-based tenant identification for the public API

Date: 2026-05-20
Status: Accepted

## Context

The public API is consumed by per-tenant static websites (each school
publishes its own static site that fetches data from sitehub-backend). The
backend must identify which tenant a request is for. The decision affects:

- API URL design (what schools' websites embed in their fetch calls).
- CDN/cache configurability (per-tenant cache rules).
- TLS / DNS setup.
- Whether tenant identity can be cleanly separated from authentication
  (which the public API does not use).

The admin and mobile APIs identify their tenant differently (from a JWT
claim) — see [ADR-0007](0007-host-based-routing-for-single-binary.md).
This ADR is specifically about the **public** API.

## Decision

Identify the tenant by the request **subdomain**:

```
https://acme.sitehub.bg/api/v1/pages/home
                  ↑
                  tenant slug = "acme"
```

- Each tenant has a unique slug (e.g., `acme`, `sredno-uchilishte-petko`).
- The slug appears in the database as `TenantSlug` (a value object in
  `sitehub-app`) and maps 1:1 to `TenantId` via a lookup at request entry.
- Middleware in `sitehub-public-api` parses the subdomain, resolves the
  `TenantId`, builds a `TenantCtx`, and attaches it to the request as an
  extractor.
- The slug is part of the public contract: changing a school's slug is a
  breaking change for that school's website.

## Alternatives considered

- **Path-prefix tenancy** (`https://sitehub.bg/t/acme/api/v1/...`). Rejected
  — defeats the per-tenant CDN/cache story (a CDN sees all paths under one
  hostname and can't apply per-tenant rules without parsing paths). Uglier
  in school-side fetch URLs.
- **API key as tenant identifier** (`X-Tenant: acme` or `Authorization`
  header). Rejected — the public API has no authentication (see
  [ADR-0007](0007-host-based-routing-for-single-binary.md)) and adding an
  identifier-header just to name the tenant is unnecessary indirection.
- **Query parameter** (`?tenant=acme`). Rejected — query params interact
  poorly with HTTP caching (some caches key on URL only, treating the
  same path with different params as a cache hit). Confusing for clients.
- **Subdomain on a separate host** (`api.sitehub.bg/acme/...`). Rejected —
  combines the worst of path-prefix and subdomain approaches without the
  per-tenant DNS isolation benefit.

## Consequences

- **Positive**: tenant identity is a property of the request URL itself,
  which makes per-tenant CDN/cache rules, rate-limit ACLs, and (later)
  per-tenant TLS or analytics trivial. The school's website embeds a
  natural URL (`acme.sitehub.bg/api/...`) that humans can read.
- **Costs accepted**: requires wildcard DNS (`*.sitehub.bg`) and a
  wildcard TLS cert. The slug is part of the public API contract — slug
  renames are breaking changes. Subdomain length and character set
  limits constrain valid slugs (DNS label rules).
- **Mitigations**: wildcard Let's Encrypt cert is standard and free.
  Slug rules (lowercase, alphanumeric + dashes, length 3–63) enforced
  at creation time by a `TenantSlug::parse` constructor. Old slugs can
  be redirected via a separate alias table if rename support is needed.

## Revisit when

- A tenant requires a custom domain (`www.school.bg`) — this becomes an
  *additional* identification path (host → tenant alias), not a
  replacement of the subdomain scheme.
- DNS / cert provisioning for new tenants becomes a friction point that
  justifies path-based fallback.
