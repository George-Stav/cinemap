# Session Memory — 2025-05-05

## Phase 1 — SQLite Schema Design

### What was done
- Added `sqlite = "0.36"` dependency to `Cargo.toml` (dropped explicit `sqlite3-sys` due to version conflict with sqlite crate's own dep)
- Created `data/schema.sql` — standalone schema file with 4 tables + 3 indexes:
  - `cinemas` (id, name, address, lat, lng, price_range, price_notes)
  - `movies` (id, tmdb_id, title, original_title, genre, duration_minutes, rating, poster_url)
  - `showtimes` (id, cinema_id, movie_id, screen_number, start_time ISO8601, format)
  - `metadata` (version, generated_at, source_week)
  - Indexes: idx_showtimes_cinema_id, idx_showtimes_movie_id, idx_cinemas_lat_lng
- Rewrote `pipeline/schema.rs` to dynamically build DB from `data/athinorama.csv`:
  - Manual CSV parser (handles quoted fields with commas, no external crate)
  - Deduplicates cinemas and movies by name, assigns sequential IDs
  - Maps `room = "default"` to NULL screen_number
  - Stores raw Greek schedule strings in start_time (parsing into ISO 8601 is Phase 2)
  - Inserts metadata row with version=1 and current timestamp
  - Output: 41 cinemas, 85 movies, 427 showtimes
- Only dependency: `sqlite = "0.36"` (no csv crate — manual parsing)

### Files created/modified
- `Cargo.toml` — `sqlite = "0.36"`, `schema` binary pointing to `pipeline/schema.rs`
- `data/schema.sql` — standalone schema contract between pipeline and app
- `pipeline/schema.rs` — binary that reads CSV and builds `data/test.sqlite`
- `data/test.sqlite` — generated database (41 cinemas, 85 movies, 427 showtimes)

### Current state
- Phase 1 COMPLETE. Schema designed, CSV-driven DB generation working.
- `data/athinorama.csv` — 428 lines (header + 427 data rows), schema: cinema,room,movie,schedule
- `pipeline/main.py` — Python scraper (Phase 0 artifact, scrapes athinorama.gr)
- `pipeline/main.rs` — "Hello, world!" stub (unused)
- Schedule strings are raw Greek text (e.g. "Πέμ.-Τετ. : 18.15") — not yet parsed

### Next steps
- Phase 2: Rust pipeline — parse schedule strings into ISO 8601 showtimes, enrich with TMDB, geocode cinemas
