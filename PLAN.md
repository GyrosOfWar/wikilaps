# Wikilaps — Implementation Plan

A spoiler-free website where anonymous users vote on how much each Formula 1
session is worth watching. For every session of a race weekend a user picks one
of three options:

- **FullRace** — worth watching in full
- **RaceIn30** — worth a condensed / ~30-minute watch
- **Highlights** — highlights are enough

Aggregated **vote** results are shown right away — they're not a spoiler. What
*is* spoiler-gated is **race results** (winner, qualifying order, etc.): those
are hidden by default so browsing the site never reveals outcomes. Race-results
data doesn't exist in the DB yet but is planned.

---

## Guiding principles

- **Spoiler-free by default — for race results, not votes.** Vote distributions
  are shown immediately; they tell you *whether other fans think it's worth
  watching*, which is the whole point of the site and not an outcome spoiler.
  What must stay hidden by default is **actual race data**: winner, podium,
  qualifying order, etc. That data isn't stored yet, but the UI and API should be
  designed so that when it's added it's gated behind an explicit per-session (or
  global) "reveal results" action.
- **Low friction voting.** Anonymous first. No login required to vote. Make
  double-voting *harder*, not impossible — accept that a determined user can
  vote twice; just stop the trivial cases.
- **One vote per (user, session), changeable.** Re-voting updates the existing
  vote rather than adding a new one.
- **Ship thin vertical slices.** Get one session votable end-to-end before
  broadening.

---

## Current state (baseline)

- `race_weekend`, `session`, `votes` tables exist (`migrations/20251114160944_init_database.sql`).
- `session_type` enum: `FreePractice`, `SprintQualification`, `SprintRace`, `Qualifying`, `Race`.
- `vote_type` enum: `FullRace`, `RaceIn30`, `Highlights`.
- Backend: one stub endpoint `GET /api/race-weekends` whose handler is `todo!()`.
- `Database::list_weekends` works; DB row structs are serialized directly (marked TODO).
- `AppError` maps everything to 500.
- Frontend: unmodified SvelteKit scaffold, no API calls.

### Known gaps to address along the way

- `votes` has **no unique constraint** on `(user_identifier, session_id)` — double votes are currently possible at the DB level.
- No `round` / ordering column on `race_weekend` (needed to sort the season sensibly).
- No separation between DB row types and API response types (existing TODO).
- Error handling has no per-variant status mapping (400s for bad input, etc.).

---

## Phase 0 — Backend foundations & cleanup

Goal: a clean, honest skeleton before adding features.

- [ ] Split handlers out of `main.rs` into a `routes`/`handlers` module (per CLAUDE.md guidance, do this once >1 route exists).
- [ ] Introduce API response DTOs separate from DB row structs (resolve the `TODO don't directly serialize database structs`). e.g. `api::RaceWeekendResponse` built from `database::RaceWeekend`.
- [ ] Implement `list_weekends` for real: return `Json<Vec<RaceWeekendResponse>>`, ordered by `year`, then round/start_date.
- [ ] Add per-variant status mapping to `AppError` (e.g. `NotFound → 404`, `BadRequest/Validation → 400`, DB/migration → 500). Add variants as needed.
- [ ] Confirm `SQLX_OFFLINE` story: generate a `.sqlx` cache (`cargo sqlx prepare`) so builds don't require a live DB, and document it. (Currently none exists.)

## Phase 1 — Schema evolution (new migrations)

Each item is a new timestamped migration in `backend/migrations/` (never edit the applied init migration).

- [ ] **Unique vote per user+session:** `ALTER TABLE votes ADD CONSTRAINT votes_user_session_unique UNIQUE (user_identifier, session_id);` — enables upsert and prevents trivial double-votes.
- [ ] **Add `round` to `race_weekend`** (`INT`, the round number within the season) for stable ordering and display ("Round 12 — Silverstone").
- [ ] **Indexes:** index `session(weekend_id)` and `votes(session_id)` for the aggregation/listing queries.
- [ ] **(Optional) `created_at`/`updated_at`** on `votes` (`TIMESTAMPTZ DEFAULT now()`) for basic abuse analysis and vote-change tracking.
- [ ] **(Optional) session display name/number** if we want "FP1/FP2/FP3" distinctions (currently `FreePractice` can't distinguish practice sessions). Decide whether to add a `session_number SMALLINT` or richer enum (see Open questions).

## Phase 2 — Anonymous identity & anti-double-vote

Goal: give each browser a stable-ish `user_identifier` and make casual double-voting annoying.

- [ ] **Anonymous ID via signed cookie:** on first visit, backend issues a random UUID stored in an `HttpOnly`, `SameSite=Lax`, long-lived cookie. This becomes `user_identifier`. Use `tower-cookies` or set it in the handler.
- [ ] Enforce **one vote per (user, session)** at the API using upsert (`INSERT ... ON CONFLICT (user_identifier, session_id) DO UPDATE SET vote_type = ...`).
- [ ] **Layered friction (pick a subset, in order of value):**
  - Cookie identity (above) — stops the 90% case.
  - Optional IP + coarse rate limiting (e.g. `tower_governor`) to cap votes/minute per IP.
  - Optional: store a hash of `IP + User-Agent` as a secondary signal for later abuse analysis (be mindful of privacy; hash + salt, don't store raw IPs long-term).
  - Explicitly **out of scope for v1:** captchas, accounts, device fingerprinting libraries.
- [ ] Document the threat model briefly: this is "make it harder," not "make it impossible."

## Phase 3 — Seed the current season

Goal: populate `race_weekend` + `session` for the current F1 season (schedule only — no results).

- [ ] **Choose a data source** (see Open questions for the tradeoff):
  - **f1db** (https://github.com/f1db/f1db) — comprehensive, MIT/CC-licensed, downloadable as CSV/JSON/SQL/SQLite. Great for structured season + circuit data. Verify it carries per-**session** start times for the *current* season (its strength is historical/results data).
  - **Jolpica-F1** (Ergast successor, REST API) — good for current-season schedules incl. session times.
  - **OpenF1** (https://openf1.org) — live/session-oriented, good session timing detail.
- [ ] Write a **seeder** as a separate cargo binary (`backend/src/bin/seed.rs`) that:
  - Fetches/reads the chosen source (vendored JSON file checked into repo, or fetched at run time).
  - Maps source rows → `race_weekend` (year, round, location, circuit_name, country_key, start_date) and `session` (weekend_id, session_type, start_time, end_time).
  - Is **idempotent** (upsert on a natural key, e.g. `(year, round)` for weekends) so re-running updates rather than duplicates. May need a unique constraint on `race_weekend(year, round)`.
  - Maps source session names → our `session_type` enum; decide handling for FP1/2/3 collapsing into `FreePractice`.
- [ ] Prefer a **vendored snapshot** (commit the season JSON under `backend/data/`) over live-fetch, so seeding is reproducible and offline-friendly. Add a make/script target to refresh it.
- [ ] Populate `country_key` with something the frontend can turn into a flag (ISO country code or emoji-flag key).

## Phase 4 — Voting & results API

- [ ] `GET /api/race-weekends` → list of weekends (year, round, location, circuit, country_key, start_date). *No vote data.*
- [ ] `GET /api/race-weekends/:id/sessions` → sessions for a weekend (id, type, start/end time). *No vote data.*
- [ ] `POST /api/sessions/:id/vote` → body `{ "vote_type": "FullRace" | "RaceIn30" | "Highlights" }`. Uses cookie identity; upserts. Returns 204 or the user's current vote.
- [ ] `GET /api/sessions/:id/results` → aggregated vote counts per `vote_type` **+ the caller's own vote**. Public / shown by default (votes aren't a spoiler). Can be folded into the sessions list to avoid an extra round-trip.
- [ ] **(Future) `GET /api/sessions/:id/race-results`** → actual outcome data (winner, qualifying order, …) once that data exists. **This** is the spoiler-bearing endpoint — the frontend only calls it on explicit reveal.
- [ ] Validation: reject unknown `vote_type`, unknown session id (→ 400/404 via the new error variants).
- [ ] Aggregation query: `SELECT vote_type, count(*) FROM votes WHERE session_id = $1 GROUP BY vote_type`.

## Phase 5 — Frontend (SvelteKit 5)

Goal: spoiler-free browsing + voting UI.

- [ ] Set up API access: a typed client + either Vite dev proxy or CORS (match Phase 0 decision). Consider SvelteKit `load` functions / server routes.
- [ ] **Season view:** list race weekends (round, location, flag, date). No results shown.
- [ ] **Weekend view:** list sessions with their date/time. Each session shows the three vote buttons, the user's current selection, and the **vote distribution** (shown by default — not a spoiler).
- [ ] **Voting interaction:** click a vote option → `POST` vote → reflect selection and updated distribution. Allow changing the vote.
- [ ] **Spoiler-gated race results (future):** when race-results data exists, render it behind an explicit "Reveal results" action per session (blurred/collapsed until clicked), only fetching `/race-results` on reveal. Support a per-session and a global "reveal everything" toggle, persisted in localStorage. Build the UI so this slot exists even before the data does.
- [ ] **Time/spoiler care:** session status (upcoming/finished) is fine to show; vote distributions are fine to show; only actual outcome data is gated. Avoid UI that leaks outcomes (e.g. don't surface a winner's flag/name anywhere by default).
- [ ] Basic responsive styling with Tailwind v4 (already configured). Flag rendering from `country_key`.

## Phase 6 — Polish, ops, and later

- [ ] Tests: backend integration tests for vote upsert + aggregation (sqlx test harness against a test DB); a couple of frontend component tests if a test setup is added.
- [ ] Deployment: containerize backend (`adapter-node` already on frontend), env config for `DATABASE_URL`, run migrations on startup (already wired).
- [ ] Observability: expand `tracing`, request logging.
- [ ] Season rollover: seeder should handle a new season; UI defaults to current year with a year switcher.
- [ ] Stretch: light accounts / "claim your votes", per-driver or per-race commentary, share-a-session links (spoiler-safe).

---

## Open questions (decide before/while building)

1. **Data source for seeding** — f1db (rich, historical, verify current-season session times) vs Jolpica/OpenF1 (current-season schedule-focused). Leaning: vendor a snapshot from whichever cleanly provides per-session start times for the current season.
2. **Practice sessions** — collapse FP1/FP2/FP3 into one `FreePractice`, or add a `session_number` so each is votable separately? (F1 fans likely want FP sessions distinguished — sprint weekends only have one FP.)
3. **CORS vs Vite proxy** for connecting frontend↔backend in dev.
4. **How strict on anti-double-vote for v1?** Cookie-only to start, add IP rate limiting later — confirm that's acceptable.
5. **Voting window** — can users vote before a session happens, only after, or anytime? (Affects whether "worth watching" is a prediction or a retrospective. Likely: vote only after the session has occurred.)
6. **Hosting/domain** — where will this run?

---

## Suggested first slice (do this next)

1. Phase 1: add the `votes` unique constraint + `race_weekend.round` migrations.
2. Phase 0: real `list_weekends` with DTOs + error status mapping.
3. Phase 3: minimal seeder for the current season (even a hand-built JSON for one or two weekends to unblock the frontend).
4. Phase 4: `POST vote` (cookie identity + upsert) and `GET results`.
5. Phase 5: weekend view with voting + live vote distribution for a single weekend.

That yields one race weekend fully votable end-to-end, with a designed-in slot
for spoiler-gated race results once that data lands.
