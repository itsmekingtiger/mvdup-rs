use std::error::Error;
use std::path::{Path, PathBuf};
use anyhow::Context;

use rusqlite::{Connection, Error::QueryReturnedNoRows, OpenFlags};
use crate::mvdup::fs::is_exist;


#[derive(thiserror::Error, Debug)]
pub enum DataStoreError {
    #[error("data store disconnected")]
    RusqliteError(#[from] rusqlite::Error),

    #[error("Can not open Database: {0}")]
    CanNotOpenDatabase(String),

    #[error("SomeOtherError")]
    SomeOtherError(String),

    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },

    #[error("unknown data store error")]
    Unknown,
}

pub struct DataBase {
    conn: Connection,
}

impl DataBase {
    pub fn init(db_path: &Path, password: Option<String>) -> Result<DataBase, DataStoreError> {
        let conn = DataBase::new_connection(db_path)?;

        if password.is_some() {
            conn.pragma_update(None, "key", password)?;
        }

        let db = DataBase { conn };
        db.create_database()?;
        return Ok(db);
    }

    /// Try to open database.
    /// If failed to open once, let user insert password and try again.
    pub fn try_open(db_path: &Path) -> Result<DataBase, DataStoreError> {
        if !is_exist(db_path) {
            return Err(DataStoreError::CanNotOpenDatabase("database does not exists".to_string()));
        }

        let conn = DataBase::new_connection(db_path)?;

        let result = conn.pragma_query(
            None,
            "schema_version",
            |row| Ok(()),
        );

        if result.is_ok() {
            return Ok(DataBase { conn });
        }

        // TODO: Database could be corrupted or other reason rather than encryption.
        //       Would be better to match errors at here.

        println!("Failed to open database, maybe database is encrypted. Trying with password: {:?}", conn);

        let passwd = rpassword::prompt_password("Your password: ").unwrap();

        conn.pragma_update(None, "key", passwd)?;
        conn.pragma_query(
            None,
            "schema_version",
            |row| Ok(()),
        )?;

        Ok(DataBase { conn })
    }

    /// Try to open database with/without password.
    /// Result is one successfully open database or panic.
    // pub fn open(props: InitProps) -> Result<Connection, DataStoreError> {
    //     if !is_exist(props.get_path()) {
    //         return Err(DataStoreError::CanNotOpenDatabase("database does not exists".to_string()));
    //     }
    //
    //
    //     let db_path = append_db_filename(path);
    //     let conn = DataBase::new_connection(db_path)?;
    //     match props {
    //         InitProps::WithoutEncryption { path } => {
    //             match DataBase::new_connection(path) {
    //                 Ok(conn) => return Ok(conn),
    //                 Err(err) => return Err(DataStoreError::RusqliteError(err)),
    //             }
    //         }
    //         InitProps::WithEncryption { path, password } => {
    //             let conn = DataBase::new_connection(path)?;
    //             println!("암호화중 {}", password);
    //             let asdf = conn.pragma_update(None, "key", password);
    //             if let Ok(asdf) = asdf {
    //                 println!("성공!")
    //             }
    //             return Ok(conn);
    //         }
    //     };
    //
    //     // TODO: validate conn
    //
    //     // return Ok(conn);
    // }

    // fn open_without_encryption()

    /// For sqlite options, see:
    ///     https://www.sqlite.org/c3ref/open.html
    ///     https://www.sqlite.org/pragma.html
    ///
    /// And for sqlcipher options, see:
    ///     https://www.zetetic.net/sqlcipher/sqlcipher-api/
    fn new_connection<P: AsRef<Path>>(db_path: P) -> Result<Connection, rusqlite::Error> {
        let mut flags = OpenFlags::SQLITE_OPEN_NO_MUTEX;
        flags.insert(OpenFlags::SQLITE_OPEN_READ_WRITE);
        flags.insert(OpenFlags::SQLITE_OPEN_CREATE);

        let conn = Connection::open_with_flags(db_path, flags)?;

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
    pub fn create_database(&self) -> Result<(), DataStoreError> {
        match self.conn.execute(CREATE_TABLE, []) {
            Ok(_) => Ok(()),
            Err(why) => Err(DataStoreError::RusqliteError(why))
        }
    }

    pub fn is_duplicated(&self, hash_val: &str) -> (bool, String) {
        let result: Result<String, rusqlite::Error> = self.conn.query_row(
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

    pub fn add(&self, hash_val: String, new_name: String) {
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

    pub fn rename(&self, hash_val: String, new_name: String) {
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

    pub fn read_all(&self) -> Result<Vec<(String, String)>, DataStoreError> {
        let mut stmt = self.conn.prepare("SELECT file_name, hash_value FROM files")?;

        let rows = stmt.query_map([], |r| Ok((r.get(0).unwrap(), r.get(1).unwrap())))?;

        let mut entries: Vec<(String, String)> = Vec::new();

        for row in rows {
            entries.push(row.unwrap())
        }

        Ok(entries)
    }

    pub fn find(&self, target: String) -> Result<Vec<(String, String)>, DataStoreError> {
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

    pub fn add_tag(&self, hash: String, term: String) -> Result<Vec<(String, String)>, DataStoreError> {
        panic!("TODO:")
    }
}

pub fn append_db_filename<P: AsRef<Path>>(path: P) -> PathBuf {
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