# ADR-0034: Secrets only from the environment, never from config files

Date: 2026-05-29
Status: Accepted

## Context

ADR-0032 and ADR-0033 established a layered configuration system: typed `Config` struct loaded by `figment` from a TOML file pointed to by `SITEHUB_CONFIG`, with `SITEHUB_`-prefixed env vars as the top override layer. Config files are committed to git (`config/dev.toml`, `config/test.toml`, `config/staging.toml`, `config/production.toml`).

Because TOML files are committed and baked into the runtime Docker image, anything placed in them is effectively public to anyone with code or image access. A reviewer or contributor unfamiliar with this distinction may treat the production TOML as a natural home for production credentials (DB password, JWT signing key, SMTP password, third-party API keys), creating a credential-leak vector that is trivial to introduce and painful to revoke once committed.

Separately, the project logs the loaded `Config` via `tracing::info!(?cfg, "loaded config")` at startup. Any sensitive field added to `Config` would silently flow into stdout, which Fly.io ships to its log aggregator and retains.

## Decision

**Secrets MUST come from the environment, never from a committed config file.**

Specifically:

1. **Where secrets live at rest**:
   - Production / staging: Fly.io app secrets (`flyctl secrets set ...`).
   - Local development: developer's local `.env` (gitignored) loaded by `dotenvy`, **only** outside production. The `.env.example` documents which keys are expected.
   - CI: GitHub Actions secrets, exposed to jobs via `env:` blocks.

2. **How they reach the application**:
   - Through environment variables, picked up by the figment `Env::prefixed("SITEHUB_")` layer, which sits on top of the TOML layer and therefore wins.
   - Secret-bearing fields on `Config` are typed with a redacting wrapper (e.g., `secrecy::SecretString` or a project-local equivalent) whose `Debug` impl prints `[REDACTED]`. This makes `tracing::info!(?cfg)` safe by default.

3. **What MUST NOT appear in any `config/*.toml`**:
   - Passwords, API keys, signing keys, OAuth client secrets, webhook secrets, SMTP credentials.
   - Anything you would not be willing to publish on the GitHub repo's front page.

4. **What MAY appear in `config/*.toml`**:
   - Non-secret operational parameters: hostnames, ports, timeouts, feature flags, log levels.
   - Public URLs, public identifiers (e.g., JWT issuer/audience strings).
   - Tunables that vary by environment and are not sensitive.

5. **Categorisation of borderline values**:
   - SurrealDB URL (host+port): non-secret, goes in TOML. Username/password: secret, env only.
   - JWT issuer/audience: non-secret. Signing key: secret, env only.
   - CORS origin list: non-secret. Webhook signing secret: env only.

6. **Tooling support**:
   - Each `config/*.toml` carries a header comment: `# No secrets in this file. Secrets come from SITEHUB_* env vars (Fly.io secrets in deployed envs). See ADR-0034.`
   - The `Config` struct's `Debug` impl is implemented so that any field added in the future without a redacting wrapper will be visibly logged in code review — making accidental leaks easier to catch.

## Alternatives considered

- **Store secrets in TOML, encrypt with sops/age/SOPS-encrypted commits** — common in GitOps stacks, but adds key-management infrastructure that's overkill for this project's scale and obscures the source of truth.
- **Keep secrets in env vars but allow them inside Config (no redacting wrapper)** — relies on every contributor remembering not to log Config. Brittle; we've already shipped a startup log line that prints the whole struct.
- **No structured config at all, only env vars** — rejected in ADR-0030 (factor 3 exception) for the same reasons ADR-0032 captured.

## Consequences

- **Positive**:
  - Clear, durable rule: contributors and reviewers don't have to re-derive the policy.
  - Type-system support via secret wrappers makes accidental log leaks loud at code review time.
  - Aligns with Fly.io's intended use of `flyctl secrets` and GitHub's intended use of Actions secrets.
- **Costs accepted**:
  - Each new secret field requires adding a redacting wrapper (small recurring cost).
  - Local development requires copying `.env.example` and filling in values, which is one extra step.
- **Mitigations**:
  - Header comments in every TOML reinforce the rule at the point of likely violation.
  - The Config Debug impl review checklist makes additions visible.

## Revisit when

- We adopt a secrets-management infrastructure (Vault, AWS Secrets Manager, SOPS-encrypted git) that changes where secrets live at rest. The rule "not in TOML" would still hold; the env-injection mechanism may change.
- The project grows to a scale where rotation tooling beyond `flyctl secrets set` is needed.
