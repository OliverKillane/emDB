use super::Database as _;
use rusqlite::{params, Connection, OptionalExtension};

pub struct SQLite {
    conn: Connection,
}

pub struct Database<'imm> {
    conn: &'imm mut Connection,
}

fn mod_sqlite_int(inp: i64) -> i32 {
    inp as i32
}

impl super::userdetails::Datastore for SQLite {
    type DB<'imm> = Database<'imm>;
    type users_key = usize;
    fn new() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name VARCHAR NOT NULL,
                premium BOOLEAN NOT NULL,
                credits MEDIUMINT NOT NULL,

                CONSTRAINT premcredits CHECK (premium OR credits >= 0)
            );
        ",
        )
        .unwrap();
        Self { conn }
    }

    fn db(&mut self) -> Self::DB<'_> {
        Database {
            conn: &mut self.conn,
        }
    }
}

impl<'imm> super::userdetails::Database<'imm> for Database<'imm> {
    type Datastore = SQLite;
    fn new_user<'qy>(
        &'qy mut self,
        username: String,
        prem: bool,
        start_creds: Option<i32>,
    ) -> usize {
        self.conn
            .prepare_cached(
                "INSERT INTO users (name, premium, credits) VALUES (?, ?, ?) RETURNING id",
            )
            .unwrap()
            .query_row::<<Self::Datastore as super::userdetails::Datastore>::users_key, _, _>(
                params![username, prem, start_creds.unwrap_or(0)],
                |row| row.get(0),
            )
            .unwrap()
    }

    fn get_info<'qy>(
        &'qy self,
        user_id: <Self::Datastore as super::userdetails::Datastore>::users_key,
    ) -> Result<(usize, String, bool, i32), ()> {
        self.conn
            .prepare_cached("SELECT name, premium, credits FROM users WHERE id = ?")
            .unwrap()
            .query_row(params![user_id], |row| {
                Ok((
                    user_id,
                    row.get(0)?,
                    row.get(1)?,
                    mod_sqlite_int(row.get(2)?),
                ))
            })
            .optional()
            .unwrap()
            .map_or(Err(()), Ok)
    }

    fn get_snapshot<'qy>(&'qy self) -> Vec<(usize, String, bool, i32)> {
        self.conn
            .prepare_cached("SELECT id, name, premium, credits FROM users")
            .unwrap()
            .query_map(params![], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    mod_sqlite_int(row.get(3)?),
                ))
            })
            .unwrap()
            .map(|row| row.unwrap())
            .collect()
    }

    fn add_credits<'qy>(
        &'qy mut self,
        user: <Self::Datastore as super::userdetails::Datastore>::users_key,
        creds: i32,
    ) -> Result<(), ()> {
        let rows = self
            .conn
            .prepare_cached("UPDATE users SET credits = credits + ? WHERE id = ?")
            .unwrap()
            .execute(params![creds, user])
            .unwrap();
        if rows == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    fn reward_premium<'qy>(&'qy mut self, cred_bonus: f32) -> Result<i64, ()> {
        let trans = self.conn.transaction().unwrap();

        let diff = {
            let mut prem_creds_stat = trans
                .prepare_cached("SELECT SUM(credits) FROM users WHERE premium = TRUE")
                .unwrap();

            let before: i64 = prem_creds_stat
                .query_row([], |a| Ok(a.get(0)))
                .unwrap()
                .unwrap_or(0);

            trans
                .prepare_cached(
                    "UPDATE users SET credits = ROUND(credits * ?, 0) WHERE premium = TRUE",
                )
                .unwrap()
                .execute(params![cred_bonus])
                .map_err(|_| ())?;

            let after: i64 = prem_creds_stat
                .query_row([], |a| Ok(a.get(0)))
                .unwrap()
                .unwrap_or(0);

            after - before
        };

        trans.commit().unwrap();

        Ok(diff)
    }

    fn total_premium_credits<'qy>(&'qy self) -> i64 {
        self.conn
            .prepare_cached("SELECT SUM(credits) FROM users WHERE premium = TRUE")
            .unwrap()
            .query_row([], |a| Ok(a.get(0)))
            .unwrap()
            .unwrap_or(0)
    }
}

impl super::GetNewUserKey for SQLite {
    fn new_user_wrap<'imm>(
        db: &mut Self::DB<'imm>,
        username: String,
        prem: bool,
        start_creds: Option<i32>,
    ) -> <Self as super::userdetails::Datastore>::users_key {
        db.new_user(username, prem, start_creds)
    }
}
