use std::path::{Path, PathBuf};

use rusqlite::{Connection, Error, Error::QueryReturnedNoRows, Result};

fn __path<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut path = PathBuf::from(path.as_ref());
    path.push(".mvdup.db");
    return path;
}

pub fn open_at<P: AsRef<Path>>(dst: P) {
    let conn = Connection::open(__path(dst)).expect("Failed to open database");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            file_name TEXT,
            hash_value TEXT
        )",
        [],
    )
    .expect("failed to create table");
}

pub fn is_duplicated<P: AsRef<Path>>(dst: P, hash_val: &str) -> (bool, String) {
    let conn = Connection::open(__path(dst)).expect("Failed to open database");
    let result: Result<String> = conn.query_row(
        "SELECT file_name FROM files WHERE hash_value = ?1",
        rusqlite::params![hash_val],
        |row| row.get(0),
    );

    match result {
        Ok(filename) => (true, filename),
        Err(QueryReturnedNoRows) => (false, "".to_string()),
        Err(e) => panic!("something worong: {}", e),
    }
}

pub fn add<P: AsRef<Path>>(dst: P, hash_val: String, new_name: String) {
    let conn = Connection::open(__path(dst)).expect("Failed to open database");
    conn.execute(
        "INSERT INTO files (
            file_name,
            hash_value
        ) VALUES (
            ?1,
            ?2
        )",
        [new_name, hash_val],
    )
    .expect("failed to insert table");
}

pub fn rename<P: AsRef<Path>>(dst: P, hash_val: String, new_name: String) {
    let conn = Connection::open(__path(dst)).expect("Failed to open database");

    conn.execute(
        "UPDATE files SET
            file_name = ?1
        WHERE
            hash_value = ?2
        ",
        [new_name, hash_val],
    )
    .expect("failed to rename file");
}

pub fn read_all<P: AsRef<Path>>(dst: P) -> Result<Vec<(String, String)>, Error> {
    let conn = Connection::open(__path(dst.as_ref())).expect("Failed to open database");

    let mut stmt = conn.prepare("SELECT file_name, hash_value FROM files")?;

    let rows = stmt.query_map([], |r| Ok((r.get(0).unwrap(), r.get(1).unwrap())))?;

    let mut entries: Vec<(String, String)> = Vec::new();

    for row in rows {
        entries.push(row.unwrap())
    }

    Ok(entries)
}
