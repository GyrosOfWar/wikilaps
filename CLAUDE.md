# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

Wikilaps is a two-part app for voting on how F1 race sessions should be covered:
a Rust/axum backend (`backend/`) backed by Postgres, and a SvelteKit 5 frontend
(`frontend/`). Users get an anonymous, cookie-based identity and vote on whether
each session is worth watching as a `FullRace`, `RaceIn30` (race in 30 min), or
just `Highlights`.

The frontend is generated against the backend's OpenAPI spec (oazapfts) and
proxies `/api` to the backend in dev — the two halves are coupled through that
contract, so **changing a backend route means regenerating the frontend client**
(see "OpenAPI contract" below).

## Repo orchestration (`just` + `mprocs`)

The root `justfile` uses **nushell** (`set shell := ["nu", "-c"]`) and delegates to
per-directory justfiles in `backend/` and `frontend/`. From the repo root:

```bash
just start       # run backend + frontend together via mprocs (etc/services.yaml)
just check       # backend: fmt --check + cargo check;  frontend: pnpm check + lint
just format      # cargo fmt + pnpm format across both
just generate    # regenerate the frontend API client (see OpenAPI contract)
just backend <cmd>   # run <cmd> in backend/
just frontend <cmd>  # run <cmd> in frontend/
```

`just start` runs the backend on `http://localhost:13252` and the SvelteKit dev
server (which proxies `/api` → `:13252`). The frontend package manager is **pnpm**
(not npm), pinned in CI to v11.

## Backend (`backend/`)

Rust, edition 2024: axum 0.8, sqlx 0.9 (Postgres, compile-time-checked
`query_as!`/`query!`), tokio, jiff + jiff-sqlx for dates/times, utoipa +
utoipa-swagger-ui for OpenAPI.

Commands (run from `backend/`):
```bash
cargo build
cargo run                 # serves on :13252, Swagger UI at /swagger-ui
cargo check
cargo test
cargo test <name>         # run a single test by substring
```

### Environment & database

`DATABASE_URL` must be set (via `backend/.env`, loaded with `dotenvy`) or the binary
panics at startup. `COOKIE_SECRET` (HMAC key for signing identity cookies) and
`COOKIE_SECURE` (`true` to set the cookie `Secure` attribute over HTTPS, default
`false`) are optional; if `COOKIE_SECRET` is unset, a random one is generated at
startup with a warning — fine for dev, but it invalidates existing cookies on every
restart, so set it in prod. `SERVER_HOST`/`SERVER_PORT` override the bind address
(default `localhost:13252`). All config lives in `config.rs` (`AppConfig::default()`).

sqlx macros check queries against a live database at compile time, so a **prepared
`.sqlx` cache exists** in `backend/.sqlx` and is committed. With it you can build
offline via `SQLX_OFFLINE=true` (CI does this). **When you add or change a `query!`/
`query_as!`, regenerate the cache** with `cargo sqlx prepare` (needs `sqlx-cli` and a
live `DATABASE_URL`) or the offline build/CI will fail.

Migrations live in `backend/migrations/` (sqlx-cli naming: `<timestamp>_<name>.sql`)
and run automatically at startup via `sqlx::migrate!("./migrations").run(&db)` in
`Database::new` (`database.rs`). Add new ones with `sqlx migrate add <name>`, or hand-
write the timestamped file following the existing pattern.

### Seeding

`src/bin/seed_db.rs` (behind the `seed` cargo feature) populates `race_weekend`/
`session` for a season by fetching the latest [f1db](https://github.com/f1db/f1db)
JSON release at runtime (nothing vendored):

```bash
cargo run --features seed --bin seed_db [year]   # defaults to current year
```

### Architecture

- `main.rs` — builds the axum router by registering handlers on an `OpenApiRouter`
  (`routes!(...)`), splits out the OpenAPI spec, and mounts Swagger UI at
  `/swagger-ui` (`/apidoc/openapi.json`). Thin: config load + DB init + serve.
- `routes.rs` — all HTTP handlers, the `AppState { db, cookie_secret, cookie_secure }`,
  and the API response types (`RaceWeekendResponse`, `SessionResponse`, `VoteCounts`,
  …) with `#[utoipa::path]` annotations that drive the OpenAPI spec. Response types are
  deliberately separate from DB row types, converted via `From` impls. `VoteCounts`
  omits `race_in_30` for non-`Race` sessions.
- `database.rs` — `Database` wraps a `PgPool` and exposes one method per query
  (`find_last_weekend`, `list_weekends`, `list_voted_sessions_for_user`, `insert_vote`,
  `upsert_race_weekend`, `upsert_session`). DB row structs (`RaceWeekend`,
  `SessionWithVotes`, `VoteCounts`) and the `SessionType`/`VoteType` sqlx enums live
  here. Runs migrations on construction.
- `auth.rs` — anonymous browser identity. `init_session` (`GET /api/session`) issues a
  random token in an HMAC-signed cookie (`wl_uid`, value `<token>.<base64 hmac>`) if the
  browser lacks a valid one; `verify_token` checks the signature with a constant-time
  compare. The `UserId` extractor (`FromRequestParts<AppState>`) pulls the verified token
  off the cookie and rejects with `401` if missing/invalid — the token is the opaque
  `user_identifier`. Client IP (`X-Forwarded-For`) and user-agent are logged only, never
  part of identity, so a vote identity survives network changes.
- `config.rs` — `AppConfig`, all env parsing.
- `pagination.rs` — generic `PageParameters` (page/size/sort/dir) and `Page<T>` envelope
  (`content`, `totalItems`, `pageNumber`, `pageSize`, `totalPages`). Scaffolding; wire it
  into list endpoints as they grow.
- `error.rs` — single `AppError` enum implementing `IntoResponse`, with `From` impls for
  `sqlx::Error`/`MigrateError` so `?` works in handlers returning `error::Result<T>`. All
  variants currently map to `500` with the debug-formatted error as the JSON body — no
  per-variant status mapping yet.
- `docs.rs` — the utoipa `ApiDocs` marker struct.
- `lib.rs` — module root; `main.rs`/`seed_db.rs` consume the crate as `wikilaps::*`.

### Domain model

Schema (`migrations/20251114160944_init_database.sql`):
- `race_weekend` — a weekend at a circuit in a given year, unique on `(year, round)`.
- `session` — belongs to a `race_weekend`; typed via the `session_type` enum
  (`sprint_qualifying`, `sprint_race`, `qualifying`, `race`).
- `votes` — a per-session vote from a `user_identifier`, typed via `vote_type`
  (`FullRace`, `RaceIn30`, `Highlights`). `UNIQUE (user_identifier, session_id)` means a
  browser's **first vote for a session wins**; later votes are ignored.

Note the enum casing differs: `session_type` values are snake_case in the DB,
`vote_type` values are PascalCase — the sqlx `#[sqlx(rename_all = ...)]` attributes
encode this, keep them in sync with the migration.

## Frontend (`frontend/`)

SvelteKit 5 (runes mode forced on for all non-`node_modules` files, see `vite.config.ts`)
+ TypeScript, Tailwind CSS v4 (`@tailwindcss/vite`), Skeleton UI
(`@skeletonlabs/skeleton-svelte`), Lucide icons, `@sveltejs/adapter-auto`.

Commands (run from `frontend/`, package manager is **pnpm**):
```bash
pnpm dev              # dev server, proxies /api → http://localhost:13252
pnpm build
pnpm preview
pnpm check            # svelte-kit sync + svelte-check (type checking)
pnpm lint             # prettier --check + eslint
pnpm format           # prettier --write
pnpm test             # vitest (client browser project + server node project) once
pnpm test:unit        # vitest watch
pnpm openapi:generate # regenerate src/lib/api.ts from the running backend
```

Tests use vitest with two projects (`vite.config.ts`): a `client` project running in a
real Chromium via `@vitest/browser-playwright` for `*.svelte.{test,spec}.ts`, and a
`server` node project for other `*.{test,spec}.ts`. There are no tests written yet.

### OpenAPI contract (backend ↔ frontend)

`src/lib/api.ts` is **generated, not hand-edited** — it's an oazapfts client built from
the backend's OpenAPI spec. To regenerate after a backend route/type change:

```bash
# backend must be running on :13252
just generate         # == pnpm openapi:generate + prettier --write on api.ts
```

The client uses `baseUrl: "/"` and hits `/api/...`, which the Vite dev proxy (and, in
prod, whatever reverse proxy) routes to the backend. `src/lib/client.ts` wraps generated
calls with app logic (e.g. `submitVote` posts a vote then `invalidateAll()`s to refresh
loaders). Route `load` functions call the generated API directly with SvelteKit's `fetch`
(`+layout.ts` calls `initSession` on every load; `+page.ts` loads the latest weekend).

### i18n (Paraglide)

Localization uses `@inlang/paraglide-js`. Message catalogs live in
`frontend/messages/{en,de}.json` (base locale `en`, config in
`project.inlang/settings.json`); the Vite plugin compiles them into
`src/lib/paraglide/` (generated — don't edit). **Add/change copy by editing the JSON
catalogs**, then import from `$lib/paraglide/messages`. Locale is resolved per-request via
`hooks.server.ts` (paraglide middleware) and URLs are de-localized for routing in
`hooks.ts`. `src/lib/i18n.ts` holds app-specific label helpers (e.g. `sessionTypeLabel`);
`LanguageSwitcher.svelte` toggles locale.

### Frontend structure

- `src/routes/+page.svelte` (+ `+page.ts`) — home, shows the latest elapsed session.
- `src/routes/weekends/[year]/` — all weekends for a season.
- `src/lib/components/` — `RaceWeekendCard`, `VoteResults`, `LanguageSwitcher`.
- `src/lib/date-time.ts` — date/time formatting (uses `temporal-polyfill`).
- `flag-icons` renders country flags via `country_key`.

## CI

GitHub Actions (`.github/workflows/`): `backend.yml` runs `cargo fmt --check`,
`cargo check --all-targets` (with `SQLX_OFFLINE=true`), and `cargo test` against a
Postgres 18 service; `frontend.yml` runs prettier, eslint, and `pnpm test` (installs
Chromium for the browser test project). Both are path-filtered to their subdirectory.
