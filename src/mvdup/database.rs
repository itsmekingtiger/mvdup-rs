use std::fmt::Error;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, Error, Error::QueryReturnedNoRows, OpenFlags, Result};
use crate::mvdup::fs::is_exist;


pub struct DataBase {
    conn: Connection,
}

pub enum InitProps<'a> {
    WithoutEncryption { path: &'a str },
    WithEncryption { path: &'a str, password: &'a str },
}

impl<'a> InitProps<'a> {
    // Does later lifetime is needed?
    fn get_path(&self) -> &'a str {
        match self {
            InitProps::WithoutEncryption { path } => path,
            InitProps::WithEncryption { path, .. } => path,
        }
    }
}


impl DataBase {
    pub fn init(props: InitProps) {

        match props {
            InitProps::WithoutEncryption { path } => DataBase::new_connection(path),
            InitProps::WithEncryption { path, password } => {
                let conn = DataBase::new_connection(path);
                conn.pragma_update(None, "key", password)
            }
        }.expect("Failed to initialize database.");

        println!("Successfully initialize database.")
    }

    /// Try to open database with/without password.
    /// Result is one successfully open database or panic.
    pub fn open(props: InitProps) -> Result<Connection> {
        if !is_exist(props.get_path()) {
            return Error::new("Database does not exists!")
        }

        let conn = match props {
            InitProps::WithoutEncryption { path } => DataBase::new_connection(path),
            InitProps::WithEncryption { path, password } => {
                let conn = DataBase::new_connection(path);
                conn.pragma_update(None, "key", password)
            }
        }?;

        // TODO: validate conn

        return Ok(conn);
    }

    /// For sqlite options, see:
    ///     https://www.sqlite.org/c3ref/open.html
    ///     https://www.sqlite.org/pragma.html
    ///
    /// And for sqlcipher options, see:
    ///     https://www.zetetic.net/sqlcipher/sqlcipher-api/
    fn new_connection<P: AsRef<Path>>(dst: P) -> Result<Connection> {
        let mut flags = OpenFlags::SQLITE_OPEN_NO_MUTEX;
        flags.insert(OpenFlags::SQLITE_OPEN_READ_WRITE);
        flags.insert(OpenFlags::SQLITE_OPEN_CREATE);

        let conn = Connection::open_with_flags(append_db_filename(dst), flags)?;

        /// Use in-memory temp store
        conn.pragma_update(None, "temp_store", "memory")?;

        /// Enable foreign key supports
        conn.pragma_update(None, "foreign_keys", "on")?;

        /// SQLite does not shrink file size when rows deleted, but reuse disk pages for next insert.
        /// So file size grown and grown. `auto_vacuum` change this behavior.
        /// But we normally does not delete rows, do does not use this option.
        ///
        /// See also `secure_delete`.
        // conn.pragma_update(None, "auto_vacuum", "INCREMENTAL".to_string())?;

        Ok(conn)
    }
}


impl DataBase {

    pub fn is_duplicated<P: AsRef<Path>>(&self, hash_val: &str) -> (bool, String) {
        let result: Result<String> = self.conn.query_row(
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

    pub fn add<P: AsRef<Path>>(&self, hash_val: String, new_name: String) {
        self.conn.execute(
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

    pub fn rename<P: AsRef<Path>>(&self, hash_val: String, new_name: String) {
        self.conn.execute(
            "UPDATE files SET
            file_name = ?1
        WHERE
            hash_value = ?2
        ",
            [new_name, hash_val],
        )
            .expect("failed to rename file");
    }

    pub fn read_all<P: AsRef<Path>>(&self) -> Result<Vec<(String, String)>, Error> {
        let mut stmt = self.conn.prepare("SELECT file_name, hash_value FROM files")?;

        let rows = stmt.query_map([], |r| Ok((r.get(0).unwrap(), r.get(1).unwrap())))?;

        let mut entries: Vec<(String, String)> = Vec::new();

        for row in rows {
            entries.push(row.unwrap())
        }

        Ok(entries)
    }

    pub fn find<P>(&self, target: String) -> Result<Vec<(String, String)>, Error>
        where
            P: AsRef<Path>,
    {
        let mut stmt = self.conn.prepare(
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

    pub fn add_tag<P>(&self, hash: String, term: String) -> Result<Vec<(String, String)>, Error>
        where
            P: AsRef<Path>,
    {
        panic!("TODO:")
    }
}

fn append_db_filename<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut path = PathBuf::from(path.as_ref());
    path.push(".mvdup.db");
    return path;
}

const CREATE_TABLE: &str = "
CREATE TABLE IF NOT EXISTS files (
    file_id     INTEGER PRIMARY KEY,
    file_name   VARCHAR,
    hash_value  VARCHAR
);

CREATE TABLE IF NOT EXISTS file_tags (
    file_id     INTEGER REFERENCES files (file_id),
    tag_value   INTEGER NOT NULL,

    UNIQUE (file_id, tag_value)
);
";
