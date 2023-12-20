//! A naive implementation for comparison with one using optimisations only possible when knowing the full set of queries.
//!
//! This implementation uses a generalised table structure, and an a secondary index (we cannot assume no ids are deleted,
//! and thus must ensure deleted ids are not reused).
use crate::UserDetails;
use std::collections::HashMap;
use typed_generational_arena::{Arena, Index};

fn premcred_predicate(premium: bool, credits: i32) -> bool {
    premium || credits >= 0
}

struct Record {
    name: String,
    premium: bool,
    credits: i32,
}

pub struct NaiveDatabase {
    users: Arena<Record>,
    userd_id_index: HashMap<usize, Index<Record, usize, usize>>,
    user_id_cnt: usize,
}

mod new_user {
    pub struct Commit {
        pub id: usize,
    }
    #[derive(Debug)]
    pub enum Errors {
        PremCredits,
    }
    pub type Return = Result<Commit, Errors>;
}

mod get_info {
    pub struct Commit {
        pub id: usize,
        pub name: String,
        pub premium: bool,
        pub credits: i32,
    }
    #[derive(Debug)]
    pub enum Errors {
        IDNotFound,
    }
    pub type Return = Result<Commit, Errors>;
}

mod get_snapshot {
    pub struct UsersRecord {
        pub id: usize,
        pub name: String,
        pub premium: bool,
        pub credits: i32,
    }
    pub struct Commit {
        pub it: Vec<UsersRecord>,
    }
    #[derive(Debug)]
    pub enum Errors {}
    pub type Return = Result<Commit, Errors>;
}

mod add_credits {
    pub struct Commit;
    #[derive(Debug)]
    pub enum Errors {
        IDNotFound,
        PremCredits,
    }
    pub type Return = Result<Commit, Errors>;
}
mod reward_premium {
    pub struct Commit {
        pub it: i32,
    }
    #[derive(Debug)]
    pub enum Errors {
        PremCredits,
    }
    pub type Return = Result<Commit, Errors>;
}
mod total_premium_credits {
    pub struct Commit {
        pub it: i64,
    }
    #[derive(Debug)]
    pub enum Errors {}
    pub type Return = Result<Commit, Errors>;
}

impl<'a> UserDetails<'a> for NaiveDatabase {
    type NewUserReturn = new_user::Return;
    type GetInfoReturn = get_info::Return;
    type GetSnapshotReturn = get_snapshot::Return;
    type AddCreditsReturn = add_credits::Return;
    type RewardPremiumReturn = reward_premium::Return;
    type TotalPremiumReturn = total_premium_credits::Return;

    fn new() -> Self {
        NaiveDatabase {
            users: Arena::new(),
            userd_id_index: HashMap::new(),
            user_id_cnt: 0,
        }
    }

    fn new_user(&mut self, username: String, prem: bool) -> usize {
        let rec = Record {
            name: username,
            premium: prem,
            credits: 0,
        };

        let res: new_user::Return = if premcred_predicate(rec.premium, rec.credits) {
            let arena_id = self.users.insert(rec);
            let id = self.user_id_cnt;
            self.user_id_cnt += 1;

            self.userd_id_index.insert(id, arena_id);

            Ok(new_user::Commit { id })
        } else {
            Err(new_user::Errors::PremCredits)
        };
        res.unwrap().id
    }
    fn get_info(&self, user_id: usize) -> get_info::Return {
        if let Some(i) = self.userd_id_index.get(&user_id) {
            let rec = self.users.get(*i).unwrap();
            Ok(get_info::Commit {
                id: user_id,
                name: rec.name.clone(),
                premium: rec.premium,
                credits: rec.credits,
            })
        } else {
            Err(get_info::Errors::IDNotFound)
        }
    }
    fn get_snapshot(&self) -> get_snapshot::Return {
        Ok(get_snapshot::Commit {
            it: self
                .userd_id_index
                .iter()
                .map(|(id, idx)| {
                    let rec = self.users.get(*idx).unwrap();
                    get_snapshot::UsersRecord {
                        id: *id,
                        name: rec.name.clone(),
                        premium: rec.premium,
                        credits: rec.credits,
                    }
                })
                .collect::<Vec<_>>(),
        })
    }
    fn add_credits(&mut self, user: usize, creds: i32) -> add_credits::Return {
        let idx = self.userd_id_index.get(&user).copied();
        if let Some(i) = idx {
            let rec = self.users.get_mut(i).unwrap();
            if premcred_predicate(rec.premium, rec.credits + creds) {
                rec.credits += creds;
                Ok(add_credits::Commit {})
            } else {
                Err(add_credits::Errors::PremCredits)
            }
        } else {
            Err(add_credits::Errors::IDNotFound)
        }
    }
    fn reward_premium(&mut self, cred_bonus: f32) -> reward_premium::Return {
        for (_, v) in self.users.iter_mut() {
            if !premcred_predicate(v.premium, ((v.credits as f32) * cred_bonus) as i32) {
                return Err(reward_premium::Errors::PremCredits);
            }
        }
        let mut total_creds = 0;
        for (_, v) in self.users.iter_mut() {
            let new_creds = ((v.credits as f32) * cred_bonus) as i32;
            total_creds += new_creds;
            v.credits = new_creds;
        }

        Ok(reward_premium::Commit { it: total_creds })
    }
    fn total_premium_credits(&self) -> total_premium_credits::Return {
        Ok(total_premium_credits::Commit {
            it: self
                .users
                .iter()
                .filter(|(_, v)| v.premium)
                .map(|(_, v)| v.credits as i64)
                .sum(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_insert() {
        let mut db = NaiveDatabase::new();
        let bob = db.new_user(String::from("bob"), true);

        assert_eq!(db.get_info(bob).unwrap().credits, 0);
        db.add_credits(bob, 10).unwrap();
        assert_eq!(db.get_info(bob).unwrap().credits, 10);
        assert_eq!(db.reward_premium(2f32).unwrap().it, 20);
        assert_eq!(db.total_premium_credits().unwrap().it, 20);
    }
}
