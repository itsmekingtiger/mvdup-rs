use std::path::{Path, PathBuf};

use rusqlite::{Connection, Error::QueryReturnedNoRows, Result};

fn __path(path: &Path) -> PathBuf {
    let mut path = PathBuf::from(path);
    path.push(".mvdup.db");
    return path;
}

pub fn open_at(dst: &Path) {
    let conn = Connection::open(__path(dst)).expect("Failed to open database");

    conn.execute(
        "create table if not exists files (
            file_name TEXT,
            hash_value TEXT
        )",
        [],
    )
    .expect("failed to create table");
}

pub fn is_duplicated(dst: &Path, hash_val: &str) -> (bool, String) {
    let conn = Connection::open(__path(dst)).expect("Failed to open database");
    let result: Result<String> = conn.query_row(
        "select file_name from files where hash_value = ?1",
        rusqlite::params![hash_val],
        |row| row.get(0),
    );

    match result {
        Ok(filename) => (true, filename),
        Err(QueryReturnedNoRows) => (false, "".to_string()),
        Err(e) => panic!("something worong: {}", e),
    }
}

pub fn add(dst: &Path, hash_val: String, new_name: String) {
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

pub fn rename(dst: &Path, hash_val: String, new_name: String) {
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
