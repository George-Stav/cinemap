# Problem Statement

You are a coding agent that is helping a single programmer build an application called CineMap. The app is cross-platform and turns finding a movie into a map-first experience. Open it and you instantly see the cinemas around you, each marked with the poster of what's playing next - like Life360, but for films. Tap a cinema to see all its showtimes, screen numbers, and ticket prices without ever leaving the map. Tap a film and the map transforms to show every cinema nearby playing it, sorted by next available screening. Built for the spontaneous moviegoer who wants to know what's on close by, right now - no accounts, no scrolling through listings, no hunting across cinema websites. Launching in Athens, designed to scale globally.

# CineMap MVP Devopment Plan

A 9-phase plan for building the CineMap MVP using a client-side SQLite approach, optimized for a developer with strong Rust, database, and API skills but no frontend experience.

---

## Overview

This plan inverts the traditional "backend first, then frontend" approach. Because all data lives in a client-side SQLite database, you build the data pipeline first, then go straight to the app. There is no API server in this MVP.

**Scope:** Athens market only — approximately 47 cinemas and 60 movies.

**Target platforms:** iOS, Android, and web (single React Native + Expo codebase).

**Estimated timeline:** 4–5 weeks of focused full-time work.

---

## Phase 0 — Set up your tools

**Duration:** 1–2 days

- Install Node.js (LTS), Rust toolchain via `rustup`
- Install SQLite locally and a GUI like DB Browser for SQLite
- Free Mapbox account, save the access token
- TMDB API account, save the API key
- Project structure:
  - `pipeline/` (Rust)
  - `data/` (output SQLite files)
  - `app/` (React Native)

---

## Phase 1 — SQLite schema design

**Duration:** 1 day

**Goal:** design the schema once, since it'll be both written by Rust and read by the app.

- Four tables, simpler than the Postgres version because no PostGIS:
  - `cinemas` — id, name, address, lat (REAL), lng (REAL), price_range, price_notes
  - `movies` — id, tmdb_id, title, original_title, genre, duration_minutes, rating, poster_url
  - `showtimes` — id, cinema_id, movie_id, screen_number, start_time (ISO 8601 string), format
  - `metadata` — single-row table with version, generated_at, source_week
- Add indexes: `(cinema_id)` and `(movie_id)` on showtimes, plain index on cinema lat/lng (no R-Tree needed at this scale)
- Write the schema as a `schema.sql` file you can run from either side
- Manually create one test database with 3 cinemas, 5 movies, 20 showtimes — confirm queries work in the GUI

**Output:** a documented schema and a tiny test SQLite file.

---

## Phase 2 — Rust pipeline

**Duration:** 3–5 days

**Goal:** take the Athinorama CSV and output a deployable SQLite artifact.

- Set up a Rust binary `pipeline` with these dependencies: `csv`, `rusqlite`, `reqwest`, `tokio`, `serde`
- Implement step-by-step:
  - **Read** the input CSV into typed structs
  - **Enrich** by querying TMDB for each unique movie title — cache results in memory and on disk so you don't re-fetch the same movie across runs
  - **Build** a fresh SQLite file from scratch each run (don't try to update; just regenerate — simpler and bulletproof at this scale)
  - **Write** all four tables, set the metadata row with version and timestamp
  - **Verify** post-write — run a few sanity queries (count rows, find one cinema's showtimes) to make sure data looks right
  - **Output** the file to `data/athens-v{version}.sqlite`
- Add a CLI argument for the input CSV path
- Run it manually with a sample CSV; open the resulting SQLite file in DB Browser to inspect

**Output:** a Rust binary you run with `cargo run -- weekly_data.csv` that produces a complete SQLite file ready to ship.

---

## Phase 3 — Versioning and distribution mechanism

**Duration:** 1–2 days

**Goal:** define how the app finds the right SQLite file each week.

- The pipeline also writes a `manifest.json` alongside the SQLite file, containing:
  - Latest version number
  - File URL
  - File checksum (SHA-256)
  - `generated_at` timestamp
- For the MVP, "distribution" can be as simple as putting both files in a public folder served by any static host (you said skip DevOps; locally, just `python -m http.server` works for development)
- Document the contract clearly so future-you doesn't need to remember:
  - `manifest.json` lives at `<base_url>/manifest.json`
  - The SQLite file lives at `<base_url>/athens-v{N}.sqlite`
- Test by manually changing a row in the source CSV, re-running the pipeline, confirming a new file with new version appears

**Output:** a working pipeline that produces versioned, checksummed SQLite artifacts ready to be downloaded by an app.

---

## Phase 4 — Frontend foundations

**Duration:** 3–4 days

**Goal:** get React Native + Expo running with your stack, before any data work.

- Initialize the project: `npx create-expo-app app --template blank-typescript`
- Install dependencies: `expo-sqlite`, `expo-file-system`, `expo-location`, `expo-font`, `@tanstack/react-query`, `zustand`, `nativewind`
- Configure NativeWind, bundle DM Sans
- Define design tokens in one file (gold #EF9F27, light/dark variants, type scale, spacing)
- Build a minimal "Hello CineMap" screen confirming everything renders on web, iOS sim, and Android emulator
- Set up TanStack Query and a Zustand store, even mostly empty for now

**Output:** a running React Native app on all three platforms showing your logo. Boring but essential.

---

## Phase 5 — Database loader

**Duration:** 2–3 days

**Goal:** implement the "check for new version, download if needed, open locally" flow. This is the biggest unique piece of the client-side approach — get it right and everything else is straightforward.

- On app launch:
  - Check `expo-file-system` for an existing local SQLite file
  - Fetch `manifest.json` from your hosting URL
  - If no local file, or local version < manifest version, download the new SQLite file (with progress indicator)
  - Verify the checksum matches the manifest
  - Atomically swap (write to temp file, then rename) so you don't end up with a corrupt file
  - Open the local SQLite database with `expo-sqlite` and keep the connection in a Zustand store
- If the manifest fetch fails (offline), use the existing local file — app still works
- Build a "Database loading" splash screen using your design tokens
- Handle the first-launch case (downloading the initial database) separately with a clearer "Setting things up" message

**Output:** app launches → checks for updates → downloads if needed → opens database → shows the empty map screen.

---

## Phase 6 — Map screen

**Duration:** 4–7 days

**Goal:** working map with real cinema markers from the local database.

- Install `@rnmapbox/maps` (native) + `mapbox-gl` (web) — Expo handles the platform split
- Build `MapScreen` filling the viewport
- Request location permission via `expo-location`, center on user with the gold pulsing dot
- Build a `useNearbyCinemas` hook: takes user lat/lng, queries SQLite for all cinemas, computes distance in JS, returns the closest 50. Wrap it in TanStack Query for caching even though it's local — the abstraction is still useful.
- Render simple gold-circle markers first, confirm taps update Zustand's `activeCinema`
- Once tap handling works, replace circles with poster-image markers from `poster_url` (use `expo-image` for caching)
- Add the count badge for cinemas with multiple movies

> **Tip:** use `npm run web` for primary development — instant reload, browser dev tools, much faster iteration than simulators.

**Output:** working map of Athens with real cinemas, real posters, taps tracked in state.

---

## Phase 7 — Bottom panel and ticker

**Duration:** 3–5 days

**Goal:** the core CineMap interaction.

- Build `BottomPanel` — fixed overlay, animates up when `activeCinema` is set
- Build `Ticker` — always-visible bottom bar with horizontal scroll of nearby movies
- Inside the panel, query SQLite directly: get all showtimes for `activeCinema.id` joined with movies, grouped by movie, ordered by start_time
- Implement showtime chips with screen numbers, time-based ordering, faded passed times (0.4 opacity, no strikethrough)
- Day selector tabs (Today, Tomorrow, Fri, Sat, Sun) — filter showtimes by start_time date range
- Showtimes/Movies tab toggle
- Price badge in gold light-color background

**Output:** tapping a cinema shows the panel with all showtimes. Bottom ticker shows nearby movies.

---

## Phase 8 — Movie focus mode

**Duration:** 2–3 days

**Goal:** tenet 5 — switch the entire app into "find this movie" mode.

- When a movie is tapped, set `focusedMovie` in Zustand
- Map markers observe `focusedMovie` — dim/grayscale cinemas not showing it
- Replace ticker with a focused-movie showtime list (all screenings across cinemas, sorted by time)
- "Back" button to clear focus
- "Also at..." links inside cinema panel that trigger movie focus

**Output:** full CineMap experience, all from local SQLite, all working on web.

---

## Phase 9 — Mobile polish

**Duration:** 3–5 days

**Goal:** make it feel right on phones.

- Test iOS Simulator, Android Emulator — fix touch targets, animation timing
- Status bar handling, safe areas, gesture conflicts
- Test on a real device via Expo Go (scan QR code)
- Configure PWA manifest + service worker for installable web version
- Cache poster images aggressively (they're already in TMDB's CDN, but local cache helps offline)

**Output:** smooth experience across iOS, Android, and web from one codebase.

---

## Realistic Timeline

| Phase Group | Duration | Difficulty |
|-------------|----------|------------|
| Pipeline (Phases 0–3) | ~1 week | Comfortable territory |
| App (Phases 4–9) | ~3–4 weeks | The learning curve |
| **Total** | **4–5 weeks** | (Add 50% buffer) |

This is shorter than an API-based plan because you've eliminated three phases of work: API design, API endpoints, and HTTP client integration. Those would typically add 1–2 weeks.

---

## Architectural Notes to Internalize

- **TanStack Query still wraps your SQLite calls** — same hooks, same caching, same loading-state machinery, just no network. When you eventually add a real API for user accounts, you swap the data source inside the hook and nothing else changes.
- **Keep all SQLite query code in one folder** (`app/db/`) with a clear interface. If you ever need to migrate to API calls, you replace the implementations in this folder and the rest of the app doesn't know.
- **The schema you ship is a contract** between the pipeline and the app. Version it carefully. If you change the schema, you'll need migration logic in the app to handle "user has v1 SQLite, app expects v2."
- **Don't try to write to the shipped SQLite file.** Treat it as read-only. If you later need user-specific data (saved cinemas, favorites), keep it in a *separate* local SQLite file owned by the app — never mix shipped data and user data in the same file.

---

## Caveat: expo-sqlite on Web

`expo-sqlite` works on web via WebAssembly, but the file system access pattern is different (browser uses IndexedDB or OPFS underneath). The library abstracts this, but **test the web flow early in Phase 5** so you don't get surprised. If it's painful, an alternative is `sql.js` directly for web, with `expo-sqlite` for native.

---

## Critical Advice

- **Don't skip Phase 4.** Resist the urge to start building features before the boring foundation is solid. Your gap is frontend; treat it like learning a new language.
- **Use AI assistance heavily for frontend.** Rust developers tend to under-use LLMs because they trust their own code. For React Native, lean on Claude Code or similar — it'll save you weeks.
- **Build the cheapest version of every feature first.** Plain colored circles before poster markers. Static panel before animated. Get the data flowing before you worry about polish.
- **Don't optimize until the whole flow works.** Frontend feels broken constantly during development; that's normal.
- **Web first, mobile second.** Web iteration is 10x faster (instant reload, browser dev tools). Solve every problem on web, then verify mobile.
