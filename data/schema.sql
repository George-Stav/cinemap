-- CineMap SQLite Schema
-- Contract between the Rust pipeline (writer) and the React Native app (reader).
-- Version this file carefully — schema changes require migration logic in the app.

CREATE TABLE IF NOT EXISTS cinemas (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT    NOT NULL,
    address         TEXT,
    lat             REAL,
    lng             REAL,
    price_range     TEXT,
    price_notes     TEXT
);

CREATE TABLE IF NOT EXISTS movies (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tmdb_id         INTEGER,
    title           TEXT    NOT NULL,
    original_title  TEXT,
    genre           TEXT,
    duration_minutes INTEGER,
    rating          REAL,
    poster_url      TEXT
);

CREATE TABLE IF NOT EXISTS showtimes (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    cinema_id       INTEGER NOT NULL REFERENCES cinemas(id),
    movie_id        INTEGER NOT NULL REFERENCES movies(id),
    screen_number   TEXT,
    start_time      TEXT    NOT NULL,  -- ISO 8601 (e.g. "2025-05-08T20:30:00")
    format          TEXT               -- e.g. "2D", "3D", "IMAX"
);

CREATE TABLE IF NOT EXISTS metadata (
    version         INTEGER NOT NULL,
    generated_at    TEXT    NOT NULL,  -- ISO 8601 timestamp
    source_week     TEXT               -- e.g. "2025-W19"
);

-- Indexes on showtimes for the two main query patterns:
--   "show all showtimes for a cinema" and "show all cinemas playing a movie"
CREATE INDEX IF NOT EXISTS idx_showtimes_cinema_id ON showtimes(cinema_id);
CREATE INDEX IF NOT EXISTS idx_showtimes_movie_id  ON showtimes(movie_id);

-- Plain index on cinema coordinates for proximity sorting
CREATE INDEX IF NOT EXISTS idx_cinemas_lat_lng ON cinemas(lat, lng);
