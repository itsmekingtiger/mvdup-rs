use std::path::{Path, PathBuf};

use rusqlite::{Connection, Error, Error::QueryReturnedNoRows, OpenFlags, Result};

/// For sqlite options, see:
///     https://www.sqlite.org/c3ref/open.html
///     https://www.sqlite.org/pragma.html
///
/// And for sqlcipher options, see:
///     https://www.zetetic.net/sqlcipher/sqlcipher-api/
pub fn new_connection<P: AsRef<Path>>(dst: P, passphrase: &str)  -> Result<Connection> {
    let mut flags = OpenFlags::SQLITE_OPEN_NO_MUTEX;
    flags.insert(OpenFlags::SQLITE_OPEN_READ_WRITE);
    flags.insert(OpenFlags::SQLITE_OPEN_CREATE);

    let conn = Connection::open_with_flags(append_db_filename(dst), flags)?;

    /// Use in-memory temp store
    conn.pragma_update(None, "temp_store", "memory")?;

    /// Enable foreign key supports
    conn.pragma_update(None, "foreign_keys", "on")?;

    /// Apply SQLCipher
    conn.pragma_update(None, "key", passphrase)?;

    /// SQLite does not shrink file size when rows deleted, but reuse disk pages for next insert.
    /// So file size grown and grown. `auto_vacuum` change this behavior.
    /// But we normally does not delete rows, do does not use this option.
    ///
    /// See also `secure_delete`.
    // conn.pragma_update(None, "auto_vacuum", "INCREMENTAL".to_string())?;

    Ok(conn)
}

fn append_db_filename<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut path = PathBuf::from(path.as_ref());
    path.push(".mvdup.db");
    return path;
}

pub fn open_at<P: AsRef<Path>>(dst: P, pass: &str) {
    let conn = new_connection(dst, pass).expect("Failed to open database");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            file_name TEXT,
            hash_value TEXT
        )",
        [],
    )
    .expect("Failed to create table. Could be incorrect password");
}

pub fn is_duplicated<P: AsRef<Path>>(dst: P, pass: &str, hash_val: &str) -> (bool, String) {
    let conn = new_connection(dst, pass).expect("Failed to open database");
    let result: Result<String> = conn.query_row(
        "SELECT file_name FROM files WHERE hash_value = ?1",
        rusqlite::params![hash_val],
        |row| row.get(0),
    );

    match result {
        Ok(filename) => (true, filename),
        Err(QueryReturnedNoRows) => (false, "".to_string()),
        Err(e) => panic!("something wrong: {}", e),
    }
}

pub fn add<P: AsRef<Path>>(dst: P, pass: &str, hash_val: String, new_name: String) {
    let conn = new_connection(dst, pass).expect("Failed to open database");
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

pub fn rename<P: AsRef<Path>>(dst: P, pass: &str, hash_val: String, new_name: String) {
    let conn = new_connection(dst, pass).expect("Failed to open database");

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

pub fn read_all<P: AsRef<Path>>(dst: P, pass: &str) -> Result<Vec<(String, String)>, Error> {
    let conn = new_connection(dst, pass).expect("Failed to open database");

    let mut stmt = conn.prepare("SELECT file_name, hash_value FROM files")?;

    let rows = stmt.query_map([], |r| Ok((r.get(0).unwrap(), r.get(1).unwrap())))?;

    let mut entries: Vec<(String, String)> = Vec::new();

    for row in rows {
        entries.push(row.unwrap())
    }

    Ok(entries)
}

pub fn find<P>(dst: P, pass: &str, target: String) -> Result<Vec<(String, String)>, Error>
where
    P: AsRef<Path>,
{
    let conn = new_connection(dst, pass).expect("Failed to open database");

    let mut stmt = conn.prepare(
        "SELECT
            file_name, hash_value
        FROM
            files
        WHERE
                file_name LIKE '%' || ? || '%'
            OR  hash_value LIKE ? || '%'",
    )?;

    let rows = stmt.query_map((target.clone(), target), |r| {
        Ok((r.get(0).unwrap(), r.get(1).unwrap()))
    })?;

    let mut entries: Vec<(String, String)> = Vec::new();

    for row in rows {
        entries.push(row.unwrap())
    }

    Ok(entries)
}
