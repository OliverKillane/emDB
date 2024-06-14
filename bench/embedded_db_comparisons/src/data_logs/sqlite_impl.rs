use rusqlite::{params, Connection};

use super::data_logs::{Database, Datastore};

pub struct SQLite {
    conn: Connection,
}

pub struct SQLiteDatabase<'imm> {
    conn: &'imm Connection,
}

impl Datastore for SQLite {
    type DB<'imm> = SQLiteDatabase<'imm>;

    fn new() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE logs (
                timestamp INTEGER,
                comment VARCHAR, 
                level INT8 -- 0 error, 1 warn, 2 info
            );
        ",
        )
        .unwrap();
        SQLite { conn }
    }

    fn db(&mut self) -> Self::DB<'_> {
        SQLiteDatabase { conn: &self.conn }
    }
}

impl<'imm> Database<'imm> for SQLiteDatabase<'imm> {
    type Datastore = SQLite;

    fn add_event(
        &mut self,
        timestamp: usize,
        comment: Option<String>,
        log_level: crate::data_logs::LogLevel,
    ) {
        self.conn
            .prepare_cached("INSERT INTO logs (timestamp, comment, level) VALUES (?, ?, ?);")
            .unwrap()
            .execute(params![
                timestamp,
                comment,
                match log_level {
                    crate::data_logs::LogLevel::Error => 0,
                    crate::data_logs::LogLevel::Warning => 1,
                    crate::data_logs::LogLevel::Info => 2,
                }
            ])
            .unwrap();
    }

    fn get_errors_per_minute(&self) -> Vec<(usize, usize)> {
        self.conn
            .prepare_cached(
                "
        WITH error_logs AS (
            SELECT
                timestamp,
                comment,
                level
            FROM
                logs
            WHERE
                level = 0 -- Assuming 0 corresponds to 'Error' log level
        ),
        minute_logs AS (
            SELECT
                timestamp % 60 AS min
            FROM
                error_logs
        ),
        errors_per_minute AS (
            SELECT
                min,
                COUNT(*) AS errors
            FROM
                minute_logs
            GROUP BY
                min
        )
        SELECT
            min,
            errors
        FROM
            errors_per_minute;
        ",
            )
            .unwrap()
            .query_map(params![], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn get_comment_summaries(&self, time_start: usize, time_end: usize) -> Vec<(String, usize)> {
        self.conn
            .prepare_cached(
                "
            SELECT
                SUBSTRING(comment, 1, 30) AS comment_summary,
                LENGTH(comment) AS comment_length
            FROM
                logs
            WHERE
                timestamp BETWEEN ? AND ?
                AND comment IS NOT NULL;
        ",
            )
            .unwrap()
            .query_map(params![time_start, time_end], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn demote_error_logs(&mut self) {
        self.conn
            .prepare_cached(
                "
            UPDATE logs
            SET level = 1 -- Assuming 1 corresponds to 'Warning' log level
            WHERE level = 0; -- Assuming 0 corresponds to 'Error' log level
        ",
            )
            .unwrap()
            .execute(params![])
            .unwrap();
    }
}
