# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

Wikilaps is a two-part app: a Rust/axum backend (`backend/`) backed by Postgres, and a SvelteKit 5 frontend (`frontend/`). The project is very early-stage — the backend exposes a single stub endpoint and the frontend is an unmodified SvelteKit starter.

## Backend (`backend/`)

Rust, edition 2024, using axum, sqlx (Postgres, compile-time checked queries via `query_as!`/`query!`), tokio, jiff for dates/times.

Commands (run from `backend/`):
```bash
cargo build
cargo run
cargo check
cargo test
```

`DATABASE_URL` must be set (via `backend/.env`, loaded with `dotenvy`) for the binary to start — it panics if unset. `COOKIE_SECRET` (HMAC key for signing identity cookies) and `COOKIE_SECURE` (`true` to set the cookie `Secure` attribute over HTTPS, default `false`) are optional: if `COOKIE_SECRET` is unset a random one is generated at startup with a warning (fine for dev, but invalidates existing cookies on every restart — set it in prod). Since `sqlx::query_as!`/`query!` macros check queries against a live database at compile time, `DATABASE_URL` must also be valid when running `cargo build`/`cargo check`, or `SQLX_OFFLINE=true` with a prepared `.sqlx` cache must be used instead (no `.sqlx` cache exists yet in this repo).

Migrations live in `backend/migrations/` (sqlx-cli naming: `<timestamp>_<name>.sql`) and are applied automatically at startup via `sqlx::migrate!("./migrations").run(&db)` in `Database::new` (`backend/src/database.rs`). Add new migrations with `sqlx migrate add <name>` if `sqlx-cli` is installed, otherwise create the timestamped file by hand following the existing naming pattern.

### Architecture

- `main.rs` — builds the axum `Router`, wires routes to handlers, holds `AppState { db: Database }`. Handlers currently live directly in `main.rs` (e.g. `list_weekends`) rather than a separate `routes`/`handlers` module — check here first when adding endpoints, and consider splitting into modules once the number of routes grows.
- `database.rs` — `Database` wraps a `PgPool` and exposes one method per query (e.g. `list_weekends`). Runs migrations on construction. Row structs (e.g. `RaceWeekend`) are defined here and currently deserialized directly from `query_as!` — there's a `TODO don't directly serialize database structs` marking that API response types should eventually be separated from DB row types.
- `auth.rs` — browser identity. `POST /api/session` (`init_session`) issues a random token in an HMAC-signed cookie (`wl_uid`, value `<token>.<base64 hmac>`); `verify_token` checks the signature with a constant-time compare. The `UserId` extractor (`FromRequestParts<AppState>`) pulls the verified token off the cookie and rejects with `401` if missing/invalid — the token is used verbatim as the opaque `user_identifier`. Client IP (from `X-Forwarded-For`) and user-agent are logged only, never part of identity, so a vote identity survives network changes.
- `error.rs` — a single `AppError` enum implementing `IntoResponse`, with `From` impls for `sqlx::Error` and `sqlx::migrate::MigrateError` so `?` works in handlers returning `error::Result<T>`. All errors currently map to `500 Internal Server Error` with the debug-formatted error message as the JSON body — there is no per-variant status mapping yet.

### Domain model

The schema (`backend/migrations/20251114160944_init_database.sql`) models F1-style race weekends:
- `race_weekend` — a weekend at a location/circuit in a given year.
- `session` — belongs to a `race_weekend`; typed via the `session_type` enum (`FreePractice`, `SprintQualification`, `SprintRace`, `Qualifying`, `Race`).
- `votes` — a per-session vote from a `user_identifier`, typed via the `vote_type` enum (`FullRace`, `RaceIn30`, `Highlights`) — this is presumably the "wiki laps" voting concept the app is named for (users voting on how a race session should be covered/highlighted).

## Frontend (`frontend/`)

SvelteKit 5 + TypeScript, Tailwind CSS v4 (via `@tailwindcss/vite`), `@sveltejs/adapter-node`.

Commands (run from `frontend/`):
```bash
npm run dev          # dev server
npm run build        # production build
npm run preview      # preview production build
npm run check        # svelte-kit sync + svelte-check (type checking)
npm run check:watch  # same, in watch mode
npm run lint         # prettier --check + eslint
npm run format       # prettier --write
```

There is no test setup yet. The frontend currently has no meaningful routes/components beyond the SvelteKit scaffold (`src/routes/+layout.svelte`, `src/routes/+page.svelte`) — it does not yet call the backend API.