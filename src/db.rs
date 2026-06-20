use crate::state::{Entry, EntryType};
use rusqlite::{Connection, Result, params};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct DbEntry {
    pub time_str: String,
    pub entry_type: EntryType,
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
        let type_str: String = row.get(1)?;
        let entry_type = if type_str.to_uppercase() == "IN" { EntryType::In } else { EntryType::Out };
        Ok(DbEntry {
            time_str: row.get(0)?,
            entry_type,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

pub fn load_entries_for_date(db_path: &Path, target_date: chrono::NaiveDate) -> Result<Vec<Entry>> {
    let history = load_history(db_path)?;
    let mut date_entries = Vec::new();

    for r in history {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&r.time_str) {
            let local_dt = dt.with_timezone(&chrono::Local);
            if local_dt.date_naive() == target_date {
                date_entries.push(Entry {
                    entry_type: r.entry_type.clone(),
                    time: local_dt,
                });
            }
        }
    }
    Ok(date_entries)
}

pub fn save_entries(db_path: &Path, date: chrono::NaiveDate, entries: &[Entry]) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS time_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            time TEXT NOT NULL,
            entry_type TEXT NOT NULL
        )",
        [],
    )?;

    // Delete existing entries for this date so we can reinsert the edited ones (or leave empty)
    let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(chrono::Local).single().unwrap();
    let end_of_day = date.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(chrono::Local).single().unwrap();
    
    conn.execute(
        "DELETE FROM time_log WHERE time >= ?1 AND time <= ?2",
        params![start_of_day.to_rfc3339(), end_of_day.to_rfc3339()],
    )?;

    for entry in entries {
        let type_str = entry.entry_type.to_string();
        // Store the time as ISO 8601 string
        let time_str = entry.time.to_rfc3339();

        conn.execute(
            "INSERT INTO time_log (time, entry_type) VALUES (?1, ?2)",
            params![time_str, type_str],
        )?;
    }

    Ok(())
}
