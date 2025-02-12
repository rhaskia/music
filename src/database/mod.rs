use std::fs;
use log::info;
use rusqlite::{
    params,
    Connection,
    Result,
    ToSql,
    OptionalExtension,
    types::{ToSqlOutput, FromSql, Value, ValueRef, FromSqlResult, FromSqlError}
};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use crate::app::track::{Track, Mood};
use crate::app::settings::Settings;
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose, Engine};

pub fn hash_filename(name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name);
    let hash = hasher.finalize();

    general_purpose::STANDARD.encode(hash)
}

pub fn init_db() -> Result<Connection> {
    let file = Settings::dir().join("tracks.db");
    let db_exists = file.exists();
    let conn = Connection::open(file)?;

    if !db_exists {
        conn.execute(
            "CREATE TABLE cache (
                file_hash TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                title TEXT NOT NULL,
                album TEXT NOT NULL,
                artists TEXT NOT NULL,
                genres TEXT NOT NULL,
                mood TEXT,
                trackno INTEGER NOT NULL,
                year TEXT NOT NULL,
                len REAL NOT NULL
            )",
            [],
        )?;
    }

    Ok(conn)
}

pub fn get_from_cache(conn: &Connection, filename: &str) -> Result<Option<Track>> {
    let file_hash = hash_filename(filename);
    let mut stmt = conn.prepare("SELECT * FROM cache WHERE file_hash = ?1")?;
    
    let result = stmt.query_row(params![file_hash], |row| {
        let artists_raw: String = row.get(4)?;
        let artists = artists_raw.split(";").map(|s| s.to_string()).collect();
        let genres_raw: String = row.get(5)?;
        let genres = genres_raw.split(";").map(|s| s.to_string()).collect();
        let mood_raw: Option<String> = row.get(6)?;
        let mood = match mood_raw {
            Some(text) => Some(string_to_mood(&text)),
            None => None,
        };

        Ok(Track {
            file: row.get(1)?,
            title: row.get(2)?,
            album: row.get(3)?,
            artists,
            genres,
            mood,
            trackno: row.get(7)?,
            year: row.get(8)?,
            len: row.get(9)?,
        })
    }).optional()?;

    Ok(result)
}

pub fn save_to_cache(conn: &Connection, item: &Track) -> Result<()> {
    let file_hash = hash_filename(&item.file);
    conn.execute(
        "INSERT INTO cache (file_hash, file_path, title, album, artists, genres, mood, trackno, year, len) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10
        )",
        // ON CONFLICT(file_hash) DO UPDATE SET 
        //     filename = excluded.filename,
        //     size = excluded.size,
        //     metadata = excluded.metadata",
        params![
            file_hash,
            item.file,
            item.title,
            item.album,
            item.artists.join(";"),
            item.genres.join(";"),
            item.mood,
            item.trackno,
            item.year,
            item.len
        ],
    )?;
    Ok(())
}

impl ToSql for Mood {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        let text: Vec<&str> = self.to_vec().into_iter().map(|b| if b { "Y" } else { "N" }).collect();
        Ok(ToSqlOutput::Owned(Value::Text(text.join("").to_string())))
    }
}

impl FromSql for Mood {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let text = if let ValueRef::Text(text) = value { text } else {
            return FromSqlResult::Err(FromSqlError::InvalidType);
        };

        let bools = text.iter().map(|c| if *c == b'Y' { true } else { false }).collect();

        FromSqlResult::Ok(Self::from_vec(bools))
    }
}

fn string_to_mood(s: &str) -> Mood {
    let bools = s.as_bytes().iter().map(|c| if *c == b'Y' { true } else { false }).collect();
    Mood::from_vec(bools)
}
