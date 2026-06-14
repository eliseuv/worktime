use crate::state::{Entry, EntryType};
use rusqlite::{Connection, Result, params};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct DbEntry {
    pub time_str: String,
    pub entry_type: String,
}

pub fn load_history(db_path: &Path) -> Result<Vec<DbEntry>> {
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let conn = Connection::open(db_path)?;
    let mut stmt =
        conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='time_log'")?;
    if !stmt.exists([])? {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare("SELECT time, entry_type FROM time_log ORDER BY time ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(DbEntry {
            time_str: row.get(0)?,
            entry_type: row.get(1)?,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

pub fn load_today_entries(db_path: &Path) -> Result<Vec<Entry>> {
    let history = load_history(db_path)?;
    let mut today_entries = Vec::new();
    let today = chrono::Local::now().date_naive();

    for r in history {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&r.time_str) {
            let local_dt = dt.with_timezone(&chrono::Local);
            if local_dt.date_naive() == today {
                today_entries.push(Entry {
                    entry_type: if r.entry_type == "IN" { EntryType::In } else { EntryType::Out },
                    time: local_dt,
                });
            }
        }
    }
    Ok(today_entries)
}

pub fn save_entries(db_path: &Path, entries: &[Entry]) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS time_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            time TEXT NOT NULL,
            entry_type TEXT NOT NULL
        )",
        [],
    )?;

    // We collect the dates of the entries we are saving
    let mut dates_to_clear = std::collections::HashSet::new();
    for entry in entries {
        dates_to_clear.insert(entry.time.date_naive());
    }

    // Delete existing entries for these dates so we can reinsert the edited ones
    for date in dates_to_clear {
        let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(chrono::Local).single().unwrap();
        let end_of_day = date.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(chrono::Local).single().unwrap();
        
        conn.execute(
            "DELETE FROM time_log WHERE time >= ?1 AND time <= ?2",
            params![start_of_day.to_rfc3339(), end_of_day.to_rfc3339()],
        )?;
    }

    for entry in entries {
        let type_str = match entry.entry_type {
            EntryType::In => "IN",
            EntryType::Out => "OUT",
        };
        // Store the time as ISO 8601 string
        let time_str = entry.time.to_rfc3339();

        conn.execute(
            "INSERT INTO time_log (time, entry_type) VALUES (?1, ?2)",
            params![time_str, type_str],
        )?;
    }

    Ok(())
}
