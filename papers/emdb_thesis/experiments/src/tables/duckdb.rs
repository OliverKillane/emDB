use duckdb::{params, Connection, OptionalExt};

use super::{PremCreditsErr, UserDetails};

pub struct DuckDBDatabase {
    conn: Connection,
}

impl<'a> UserDetails<'a> for DuckDBDatabase {
    type UsersID = usize;
    type ReturnString = String;

    fn new() -> Self {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute_batch(
            "
            CREATE SEQUENCE user_ids START 1;
            CREATE TABLE users (
                id BIGINT PRIMARY KEY DEFAULT NEXTVAL('user_ids'),
                name VARCHAR NOT NULL,
                premium BOOLEAN NOT NULL,
                credits INTEGER NOT NULL,

                CONSTRAINT premcredits CHECK (premium OR credits >= 0)
            );
        ",
        )
        .unwrap();
        Self { conn }
    }

    fn new_user(&mut self, username: String, prem: bool) -> Self::UsersID {
        self.conn
            .prepare_cached(
                "INSERT INTO users (name, premium, credits) VALUES (?, ?, 0) RETURNING id",
            )
            .unwrap()
            .query_row::<Self::UsersID, _, _>(params![username, prem], |row| row.get(0))
            .unwrap()
    }

    fn get_info(
        &self,
        user_id: Self::UsersID,
    ) -> Result<super::UserInfo<Self::ReturnString, Self::UsersID>, super::IDNotFoundErr> {
        self.conn
            .prepare_cached("SELECT name, premium, credits FROM users WHERE id = ?")
            .unwrap()
            .query_row(params![user_id], |row| {
                Ok(super::UserInfo {
                    id: user_id,
                    name: row.get(0)?,
                    premium: row.get(1)?,
                    credits: row.get(2)?,
                })
            })
            .optional()
            .unwrap()
            .map_or(Err(super::IDNotFoundErr), Ok)
    }

    fn get_snapshot(&self) -> Vec<super::UserInfo<Self::ReturnString, Self::UsersID>> {
        self.conn
            .prepare_cached("SELECT id, name, premium, credits FROM users")
            .unwrap()
            .query_map(params![], |row| {
                Ok(super::UserInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    premium: row.get(2)?,
                    credits: row.get(3)?,
                })
            })
            .unwrap()
            .map(|row| row.unwrap())
            .collect()
    }

    fn add_credits(&mut self, user: Self::UsersID, creds: i32) -> Result<(), super::AddCreditsErr> {
        let rows = self
            .conn
            .prepare_cached("UPDATE users SET credits = credits + ? WHERE id = ?")
            .unwrap()
            .execute(params![creds, user])
            .unwrap();
        if rows == 0 {
            Err(super::AddCreditsErr::IDNotFound)
        } else {
            Ok(())
        }
    }

    fn reward_premium(&mut self, cred_bonus: f32) -> Result<i64, super::PremCreditsErr> {
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
                .prepare_cached("UPDATE users SET credits = credits * ? WHERE premium = TRUE")
                .unwrap()
                .execute(params![cred_bonus])
                .map_err(|_| PremCreditsErr)?;

            let after: i64 = prem_creds_stat
                .query_row([], |a| Ok(a.get(0)))
                .unwrap()
                .unwrap_or(0);

            after - before
        };

        trans.commit().unwrap();

        Ok(diff)
    }

    fn total_premium_credits(&self) -> i64 {
        self.conn
            .prepare_cached("SELECT SUM(credits) FROM users WHERE premium = TRUE")
            .unwrap()
            .query_row([], |a| Ok(a.get(0)))
            .unwrap()
            .unwrap_or(0)
    }
}
