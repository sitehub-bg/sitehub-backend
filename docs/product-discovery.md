# sitehub — Product Discovery

Date: 2026-05-25
Status: Complete

## What is sitehub

A multi-tenant platform for Bulgarian school and kindergarten websites. Schools get modern, accessible, professionally designed websites with zero upfront cost. The platform handles hosting, compliance, and maintenance — schools just manage their content.

## Vision

Replace the fragmented, abandoned, and outdated school website landscape in Bulgaria with a unified, open-source platform that municipalities and schools can trust.

## Project purpose

1. **Business** — real SaaS product, revenue, customers. Top priority.
2. **Master's thesis** — comes naturally from building the business.
3. **Job interview portfolio** — bonus, not a driver of decisions.

## Market

### Size

| Segment | Count |
|---|---|
| Schools in Bulgaria | ~2,500 |
| Kindergartens (phase 2) | ~1,500 |
| **Total addressable market** | **~4,000** |

### Research

103 school websites analyzed across 5 cities:

| City | Schools analyzed | Quality (good/adequate/basic) |
|---|---|---|
| Pleven | 17 | 10% / 50% / 40% |
| Burgas | 31 | 15% / 45% / 40% |
| Kavarna | 4 (all in municipality) | 25% / 50% / 25% |
| Varna | 24 | 33% / 33% / 33% |
| Sofia | 27 | 40% / 40% / 20% |

School types covered: НУ, ОУ, СУ, МГ, ППМГ, АЕГ, НЕГ, ПГРЕ, ПГЧЕ, ПГ, НУИ, НБУ, ОбУ, ЧОУ, Спортно.

### School pain points

1. **Reliability** (primary) — schools were burned by unprofessional developers who abandoned them, extorted money after promising free service. They want someone predictable and reliable.
2. **Quality** — sites look embarrassing, outdated, inconsistent. Parents judge.
3. **Difficulty** — WordPress is confusing. Updates depend on "the IT teacher" who may leave.
4. **Compliance** — missing legally required sections (budget, GDPR, anti-bullying).

### Competition

| Competitor | Model | Weakness |
|---|---|---|
| daskalo.com | Free WordPress hosting, ad-supported, 5000+ schools | Terrible quality, ads, no school-specific structure, abandoned feel |
| Local freelancers | 500-3,000 lv creation + 68-320 lv/mo maintenance | Unreliable, inconsistent, expensive, schools get abandoned |
| Agencies | 3,000-8,000 lv creation + expensive maintenance | Way too expensive for public schools |

**sitehub's advantage:** purpose-built for schools, open source (no lock-in), municipal-branded domains, 4-18x cheaper than alternatives, compliance built-in, one person to trust.

## Content sections (derived from 103 schools)

### Must-have (80%+ of schools)

| Section | Bulgarian | Frequency |
|---|---|---|
| Home | Начало | ~98% |
| Admissions | Прием | ~85% |
| News | Новини | ~78% |
| Contacts | Контакти | ~80% |
| About | За нас | ~72% |
| Documents | Документи | ~70% |

### Should-have (40-70%)

Staff directory (~55%), Projects (~45%), Gallery (~35%), Schedules (~38%), History (~35%), Curriculum (~30%)

### Nice-to-have (<30%)

Budget, Scholarships, Olympiads, e-Diary link, Public Council, Innovative School badge, STEM, Buyer Profile, Data Protection, School Meals

### Type-specific

- **Professional gymnasiums (ПГ):** Specialties section
- **Language gymnasiums:** Certifications (DELF, DSD)
- **Sports schools:** Champions/Achievements
- **Private schools:** FAQ, Careers, State Funding

## Architecture

### Tech stack

```
Rust        → backend (API, domain logic, auth)
SurrealDB   → database (multi-tenant, feature config)
Astro       → everything frontend (school sites + admin panel)
React       → interactive islands inside Astro (forms, editors)
Cloudflare  → hosting (Pages, R2, CDN)
```

### System diagram

```
┌─────────────────────────────────────────────┐
│  admin.sitehub.bg         (Admin Panel)      │
│  auth.sitehub.bg          (Login/JWT)        │
└──────────────┬──────────────────────────────┘
               │ JWT-authenticated
               ▼
┌─────────────────────────────────────────────┐
│  sitehub-backend (Rust, hexagonal arch)      │
│  ├── sitehub-app     (domain + use cases)    │
│  ├── sitehub-storage (SurrealDB)             │
│  ├── sitehub-jwt     (token issuer)          │
│  ├── sitehub-admin-api                       │
│  ├── sitehub-auth-api                        │
│  ├── sitehub-mobile-api  (placeholder)       │
│  └── sitehub-public-api ◄── Astro fetches    │
└──────────────┬──────────────────────────────┘
               │ Build-time fetch
               ▼
┌─────────────────────────────────────────────┐
│  Astro SSG (per school)                      │
│  school1.pleven.bg   ──► Cloudflare CDN      │
│  school2.pleven.bg   ──► Cloudflare CDN      │
│  school3.burgas.bg   ──► Cloudflare CDN      │
└─────────────────────────────────────────────┘
```

### Key architectural decisions

| Decision | Choice | Rationale |
|---|---|---|
| Backend framework | Rust + Axum | Performance, correctness, portfolio value |
| Architecture | Hexagonal (ports & adapters) | Dependency isolation, testable domain, no I/O in core |
| Database | SurrealDB, single DB | Multi-tenant with tenant_id. DB-per-tenant rejected (migration burden at 5000 DBs) |
| Frontend framework | Astro v6.3 | Zero JS by default, SSG-first, content collections, islands for interactivity |
| Static generation | Rebuild on publish | Admin publishes → backend regenerates affected pages → pushes to CDN |
| Media storage | Cloudflare R2 | Zero egress fees, S3-compatible, $0.015/GB/month |
| Static hosting | Cloudflare Pages | Free, unlimited bandwidth, global CDN |
| Admin panel | Astro + React islands | Minimal tech stack — same framework for everything |
| Content editor | WYSIWYG (TipTap) | School admins expect Google Docs-like experience. TipTap is extensible, React-friendly |
| Multi-language | Built-in from day one | Retrofitting i18n is extremely painful. Language gymnasiums and Erasmus+ projects need it |
| Accessibility | WCAG 2.1 AA from start | European Accessibility Act compliance. Selling point. Easier to build right than retrofit |
| Ads | No ads on any tier | Destroys trust, negligible revenue, repeats daskalo.com's mistakes |

### Frontend model

- **Headless CMS approach** — backend serves content via API, frontend is decoupled
- **Each school has its own Astro project** — default template generated from shared codebase
- **Schools can customize or build their own frontend** against the public API
- **Considering open source** — addresses trust/lock-in concerns directly

### Feature registry

School configuration stored in SurrealDB:

```
school:mg-geo-milev {
  features: ["about", "news", "staff", "admissions", "education",
             "documents", "stem", "gallery", "contacts"],
  theme: { primary: "#1a5276" },
  domain: "mg-geo-milev.pleven.bg",
  locale: ["bg", "en"]
}
```

This drives the admin panel (what content types are available) and the SSG build (what pages to generate).

## Domain strategy

Municipal subdomain model — schools get subdomains under the city domain:

```
mg-geo-milev.pleven.bg       ← free, official-looking
su-stoyan-zaimov.pleven.bg   ← consistent branding
school.sitehub.bg            ← fallback without municipal partnership
custom-domain.bg             ← paid feature
```

Municipality provides wildcard DNS (*.pleven.bg → Cloudflare). One-time setup per municipality. Zero domain costs for schools.

## Onboarding & workflow

- **Self-service signup** + migration help
- **One-time setup per school** — configure FE project, enable sections, customize branding
- **Automated scraper** for migrating content from old school sites (bulk import)
- **Content workflow:** Draft → Review → Publish (SSG only builds published content)
- **Single admin role** for MVP (revisit roles later)
- **Municipal admin view** planned — dashboard showing all schools in a city

## Pricing

### Tiers

| | Free | Standard | Premium | Enterprise |
|---|---|---|---|---|
| **Price** | 0 EUR/mo | **20 EUR/mo** | **100 EUR/mo** | Contact us |
| **Annual** | 0 | 240 EUR (470 lv) | 1,200 EUR (2,350 lv) | Custom |
| **Sections** | Core 6 | All | All + calendar, alumni, virtual tour, forms, parent portal, newsletter | Custom development |
| **Admins** | 2 | 5 | Unlimited | Unlimited |
| **Storage** | 1 GB | 10 GB | Unlimited | Unlimited |
| **Languages** | BG | BG + EN | Any | Any |
| **Support** | Community | Email | Priority SLA (4hrs), phone, dedicated contact | Dedicated engineer, on-site training |
| **Domain** | school.sitehub.bg | Custom / municipal | Custom / municipal | White-label |
| **Branding** | "Powered by sitehub" | No branding | No branding | White-label |
| **Design** | Standard template | Standard template | Custom design work | Fully custom |
| **Workflow** | Publish immediately | Draft → Review → Publish | Draft → Review → Publish | Custom |
| **Analytics** | None | Cloudflare (basic) | Cloudflare (basic) | Custom |
| **Export** | None | ZIP + JSON | ZIP + JSON | ZIP + JSON |

### Migration pricing

| Scenario | Migration cost |
|---|---|
| Municipal contract (all schools) | **Free** |
| Individual school | **500 EUR** one-time |

### Additional features (all paid tiers)

- WCAG 2.1 AA accessibility
- SSL/CDN included
- Data export (static site ZIP + structured JSON)
- No ads, ever

## Competitive pricing analysis

| Approach | Year 1 cost | Annual cost | Quality |
|---|---|---|---|
| Cheap freelancer | 1,400 lv | 900 lv | Bad, unreliable |
| Professional WordPress | 3,900 lv | 1,500 lv | Decent, but you're dependent on one person |
| Agency | 6,900 lv | 2,500 lv | Good, but very expensive |
| **sitehub Standard** | **0 lv** | **470 lv** | **Professional, accessible, always maintained** |

**sitehub is 2-9x cheaper and better.**

### Municipal example (Pleven, 17 schools)

| | Traditional | sitehub |
|---|---|---|
| Year 1 | 59,500 lv (creation + maintenance) | 8,000 lv |
| Year 2+ | 25,500 lv/year | 8,000 lv/year |
| Consistency | 17 different designs, developers | Unified branding under *.pleven.bg |
| Compliance | Often missing | Built-in |
| Lock-in | Dependent on freelancer | Open source, full data export |

Municipality saves **~51,500 lv in year one, ~17,500 lv every year after.** All contracts stay under 20,000 lv/year direct-award threshold (no public tender needed).

## Revenue projections (realistic)

### Assumptions

- 20-30% of market becomes paying customers (800-1,200 schools)
- 50-60% stay on free tier (marketing via footer branding)
- 20-30% never adopt

### Revenue ramp

| Phase | Timeline | Schools | Paying | Revenue |
|---|---|---|---|---|
| Prove it | Months 1-6 | ~30 | ~5 | ~100 EUR/mo |
| First municipal deal | Months 6-12 | ~70 | ~25 | ~500 EUR/mo |
| Word spreads | Year 2 | ~200 | ~80 | ~1,600 EUR/mo |
| Real traction | Year 3 | ~500 | ~150 | ~3,000-4,000 EUR/mo |
| Market coverage | Year 4-5 | ~1,500 | ~400 | ~8,000-10,000 EUR/mo |

### Revenue mix at 500 schools

| Tier | Count | Monthly | Annual |
|---|---|---|---|
| Free | 350 | 0 | 0 |
| Standard (20 EUR) | 120 | 2,400 | 28,800 |
| Premium (100 EUR) | 25 | 2,500 | 30,000 |
| Enterprise (~300 EUR) | 5 | 1,500 | 18,000 |
| **Total** | **500** | **6,400** | **76,800** |

### Infrastructure cost

~50 EUR/month regardless of scale (Cloudflare free tiers + small VPS for backend). Break-even at 3 paying schools.

### Steady state (full market)

150,000 - 250,000 EUR/year. Profitable from month 1 of first municipal deal.

## Go-to-market

### Pilot cities

| City | Schools | Advantage | Strategy |
|---|---|---|---|
| **Pleven** | 17 | User lives here, worst sites, warm lead at МГ Гео Милев | Showcase site → municipality pitch |
| **Kavarna** | 4 | User has property, 100% coverage possible | Quick win, entire municipality |
| **Burgas** | 31 | Most innovative mayor | Municipal contract play |
| **Varna** | 24 | User has property, large city | Scale play after proving concept |
| **Sofia** | 27+ | Largest market | Long-term, hardest sell (sites already better) |

### Sequence

1. Build МГ "Гео Милев" Плевен as showcase (free, replicate all existing sections)
2. Headmistress tells other directors (word of mouth in tight community)
3. Pitch to Pleven municipality with live proof — ask for wildcard DNS first (easy yes)
4. Municipal contract → 17 schools at once
5. Take the Pleven success story to Kavarna (4 schools, quick win)
6. Pitch to Burgas mayor with two municipal success stories

### The trust pitch

> "Your school website will never be abandoned again. It's open source — your code and data are yours. You can leave anytime. We're always here because we make money by being so good you don't want to leave. And it costs you less than what you pay today."

## First customer

**МГ "Гео Милев" Плевен** — user's own school (alumnus). Headmistress is supportive and remembers him. Free showcase site that must replicate all 10 existing sections: Home, About, News (Olympiads), Forum, Education (Schedule, Timetables, Curriculum, Textbooks, Available Spots, Forms, Interest Clubs, National Programs, Graduates), Documents (Public Services, Parent/Student Info, Templates, Helpline, Budget), Admissions, STEM, Contacts, Gallery.

## Features not at MVP

- Mobile app (sitehub-mobile-api is a placeholder)
- Notifications (email, push, RSS — add later)
- Municipal admin dashboard (design later, keep in mind architecturally)
- Advanced roles (Admin/Editor/Viewer — single admin role for now)
- Forum (rare feature, only МГ Плевен has one)

## Open decisions

- Exact Astro template design
- SurrealDB schema for feature registry and content model
- i18n data model structure (translations map vs separate fields)
- TipTap editor configuration and custom blocks
- Automated scraper architecture for migration
- Build pipeline: how Astro rebuilds are triggered and deployed per school
- Municipal admin dashboard scope
