# ADR-0010: Remove sitehub-mobile-api, use path-based client routing

Date: 2026-05-25
Status: Accepted. Supersedes part of ADR-0003.

## Context

The original 8-package layout (ADR-0003) included `sitehub-mobile-api` as a separate driving adapter for the mobile app. During product discovery, mobile was deprioritized ("not a priority"). The web and mobile clients consume the same school content (news, staff, pages, gallery) — the differences are in pagination, payload size, and authentication, not in the data itself.

Professional experience from the author's day job confirmed that separate BFFs for web and mobile tend to diverge in unhelpful ways, but a single API with no client awareness also causes friction when clients evolve at different speeds.

## Decision

Remove the `sitehub-mobile-api` package. Instead, use path-based client routing within `sitehub-public-api`:

- `/web/v1/{resource}` — optimized for Astro SSG builds (full content, all items)
- `/mobile/v1/{resource}` — optimized for mobile app (paginated, lighter payloads)

Both route prefixes call the same use cases in `sitehub-app`. The handlers can diverge independently when needed.

## Alternatives considered

- **Keep `sitehub-mobile-api` as a separate package** — adds a package boundary for an API that doesn't exist yet. Premature separation.
- **Single `/v1/` prefix with query parameters for client differences** — no isolation between client concerns. When mobile needs a different response shape, the handler becomes a mess of conditionals.
- **Keep the package as an empty skeleton** — zero cost but misleading. Suggests mobile is a first-class deployment target when it isn't.

## Consequences

- **Positive**: 7 packages instead of 8. Clear client routing within one package. Mobile and web can evolve their response shapes independently via separate handler modules.
- **Costs accepted**: if mobile eventually needs fundamentally different infrastructure (e.g., WebSocket push), we may need to extract it. The path-based separation makes this extraction straightforward.
- **Mitigations**: the `/mobile/v1/` prefix is a placeholder. It can start as an alias for the web handlers and diverge only when real mobile requirements emerge.

## Revisit when

When the mobile app is actively developed and its API needs diverge significantly from the web API (different auth model, push notifications, offline sync).
