use sqlite;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const SCHEMA_SQL: &str = include_str!("../data/schema.sql");
const CSV_PATH: &str = "../data/athinorama.csv";
const DB_PATH: &str = "../data/test.sqlite";

/// Parse a single CSV line handling quoted fields.
/// Fields containing commas are wrapped in double quotes in the CSV.
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    fields.push(current.trim().to_string());
    fields
}

fn main() {
    // Remove existing test DB so we start fresh
    if Path::new(DB_PATH).exists() {
        fs::remove_file(DB_PATH).expect("failed to remove existing test DB");
    }

    let conn = sqlite::open(DB_PATH).expect("failed to create SQLite database");

    // Create schema
    conn.execute(SCHEMA_SQL)
        .expect("failed to execute schema SQL");
    println!("Schema created successfully.");

    // Read CSV
    let csv_content = fs::read_to_string(CSV_PATH).expect("failed to read CSV file");
    let mut lines = csv_content.lines();

    // Skip header
    lines.next();

    // Track unique cinemas and movies, mapping name -> inserted row id
    let mut cinema_ids: HashMap<String, i64> = HashMap::new();
    let mut movie_ids: HashMap<String, i64> = HashMap::new();
    let mut next_cinema_id: i64 = 1;
    let mut next_movie_id: i64 = 1;
    let mut showtime_count: i64 = 0;

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }

        let fields = parse_csv_line(line);
        if fields.len() < 4 {
            eprintln!("skipping malformed line: {}", line);
            continue;
        }

        let cinema_name = &fields[0];
        let room = &fields[1];
        let movie_title = &fields[2];
        let schedule = &fields[3];

        // Insert cinema if not seen before
        if !cinema_ids.contains_key(cinema_name) {
            let mut stmt = conn
                .prepare("INSERT INTO cinemas (name) VALUES (?);")
                .unwrap();
            stmt.bind((1, cinema_name.as_str())).unwrap();
            stmt.next().unwrap();
            cinema_ids.insert(cinema_name.clone(), next_cinema_id);
            next_cinema_id += 1;
        }

        // Insert movie if not seen before
        if !movie_ids.contains_key(movie_title) {
            let mut stmt = conn
                .prepare("INSERT INTO movies (title) VALUES (?);")
                .unwrap();
            stmt.bind((1, movie_title.as_str())).unwrap();
            stmt.next().unwrap();
            movie_ids.insert(movie_title.clone(), next_movie_id);
            next_movie_id += 1;
        }

        let cinema_id = cinema_ids[cinema_name];
        let movie_id = movie_ids[movie_title];

        // Insert showtime — store the raw schedule string in start_time for now.
        // Proper schedule parsing (expanding day ranges + times into individual
        // ISO 8601 showtimes) is a Phase 2 concern.
        let mut stmt = conn
            .prepare(
                "INSERT INTO showtimes (cinema_id, movie_id, screen_number, start_time)
                 VALUES (?, ?, ?, ?);",
            )
            .unwrap();
        stmt.bind((1, cinema_id)).unwrap();
        stmt.bind((2, movie_id)).unwrap();
	stmt.bind((3, room.as_str())).unwrap();
        stmt.bind((4, schedule.as_str())).unwrap();
        stmt.next().unwrap();
        showtime_count += 1;
    }

    // Insert metadata
    let mut stmt = conn
        .prepare("INSERT INTO metadata (version, generated_at, source_week) VALUES (?, datetime('now'), ?);")
        .unwrap();
    stmt.bind((1, 1i64)).unwrap();
    stmt.bind((2, sqlite::Value::Null)).unwrap();
    stmt.next().unwrap();

    println!(
        "Done: {} cinemas, {} movies, {} showtimes.",
        cinema_ids.len(),
        movie_ids.len(),
        showtime_count
    );
    println!("Database written to: {}", DB_PATH);
}
