//! Implementation using the knowledge of all possible queries.

use crate::UserDetails;
use slab::Slab;
use std::{cell::UnsafeCell, marker::PhantomData};

fn premcred_predicate(premium: bool, credits: i32) -> bool {
    premium || credits >= 0
}

fn id_to_table(id: usize) -> (bool, usize) {
    (id & 1 == 1, id >> 1)
}

fn table_to_id(premium: bool, id: usize) -> usize {
    (id << 1) | (premium as usize)
}

struct Record {
    name: UnsafeCell<String>,
    credits: i32,
}

pub struct ForesightDatabase<'a> {
    non_prem_user: Slab<Record>,
    prem_user: Slab<Record>,
    premium_credits: i64,
    _marker: PhantomData<&'a str>,
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
    pub struct Commit<'a> {
        pub id: usize,
        pub name: &'a str,
        pub premium: bool,
        pub credits: i32,
    }
    #[derive(Debug)]
    pub enum Errors {
        IDNotFound,
    }
    pub type Return<'a> = Result<Commit<'a>, Errors>;
}

mod get_snapshot {
    pub struct UsersRecord<'a> {
        pub id: usize,
        pub name: &'a str,
        pub premium: bool,
        pub credits: i32,
    }
    pub struct Commit<'a> {
        pub it: Vec<UsersRecord<'a>>,
    }
    #[derive(Debug)]
    pub enum Errors {}
    pub type Return<'a> = Result<Commit<'a>, Errors>;
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

impl<'a> UserDetails<'a> for ForesightDatabase<'a> {
    type NewUserReturn = new_user::Return;
    type GetInfoReturn = get_info::Return<'a>;
    type GetSnapshotReturn = get_snapshot::Return<'a>;
    type AddCreditsReturn = add_credits::Return;
    type RewardPremiumReturn = reward_premium::Return;
    type TotalPremiumReturn = total_premium_credits::Return;

    fn new() -> Self {
        Self {
            non_prem_user: Slab::new(),
            prem_user: Slab::new(),
            premium_credits: 0,
            _marker: PhantomData,
        }
    }

    fn new_user(&mut self, username: String, prem: bool) -> usize {
        let res: new_user::Return = if prem {
            let id = self.prem_user.insert(Record {
                name: username.into(),
                credits: 0,
            });
            Ok(new_user::Commit {
                id: table_to_id(true, id),
            })
        } else {
            let id = self.non_prem_user.insert(Record {
                name: username.into(),
                credits: 0,
            });
            Ok(new_user::Commit {
                id: table_to_id(false, id),
            })
        };
        res.unwrap().id
    }
    fn get_info(&self, user_id: usize) -> get_info::Return<'a> {
        let (premium, id) = id_to_table(user_id);
        let rec = if premium {
            self.prem_user.get(id)
        } else {
            self.non_prem_user.get(id)
        };
        match rec {
            Some(rec) => Ok(get_info::Commit {
                id: user_id,
                name: unsafe { rec.name.get().as_ref().unwrap() }, // given we do not deallocate, and it is not mutated
                premium,
                credits: rec.credits,
            }),
            None => Err(get_info::Errors::IDNotFound),
        }
    }
    fn get_snapshot(&self) -> get_snapshot::Return<'a> {
        let mut it = Vec::with_capacity(self.non_prem_user.len() + self.prem_user.len());
        for (id, rec) in self.non_prem_user.iter() {
            it.push(get_snapshot::UsersRecord {
                id: table_to_id(false, id),
                name: unsafe { rec.name.get().as_ref().unwrap() },
                premium: false,
                credits: rec.credits,
            });
        }
        for (id, rec) in self.prem_user.iter() {
            it.push(get_snapshot::UsersRecord {
                id: table_to_id(true, id),
                name: unsafe { rec.name.get().as_ref().unwrap() },
                premium: true,
                credits: rec.credits,
            });
        }
        Ok(get_snapshot::Commit { it })
    }
    fn add_credits(&mut self, user: usize, creds: i32) -> add_credits::Return {
        let (premium, id) = id_to_table(user);
        let rec = if premium {
            self.prem_user.get_mut(id)
        } else {
            self.non_prem_user.get_mut(id)
        };
        match rec {
            Some(rec) => {
                if premcred_predicate(premium, rec.credits + creds) {
                    rec.credits += creds;
                    if premium {
                        self.premium_credits += creds as i64;
                    }
                    Ok(add_credits::Commit)
                } else {
                    Err(add_credits::Errors::PremCredits)
                }
            }
            None => Err(add_credits::Errors::IDNotFound),
        }
    }
    fn reward_premium(&mut self, cred_bonus: f32) -> reward_premium::Return {
        for (_, v) in self.prem_user.iter_mut() {
            if !premcred_predicate(true, ((v.credits as f32) * cred_bonus) as i32) {
                return Err(reward_premium::Errors::PremCredits);
            }
        }
        let mut total_creds = 0;
        let mut cred_diff = 0;
        for (_, v) in self.prem_user.iter_mut() {
            let new_creds = ((v.credits as f32) * cred_bonus) as i32;
            total_creds += new_creds;
            cred_diff += (new_creds - v.credits) as i64;
            v.credits = new_creds;
        }
        self.premium_credits += cred_diff;

        Ok(reward_premium::Commit { it: total_creds })
    }
    fn total_premium_credits(&self) -> total_premium_credits::Return {
        Ok(total_premium_credits::Commit {
            it: self.premium_credits,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_insert() {
        let mut db = ForesightDatabase::new();
        let bob = db.new_user(String::from("bob"), true);

        assert_eq!(db.get_info(bob).unwrap().credits, 0);
        db.add_credits(bob, 10).unwrap();

        assert_eq!(db.get_info(bob).unwrap().credits, 10);
        assert_eq!(db.reward_premium(2f32).unwrap().it, 20);
        assert_eq!(db.total_premium_credits().unwrap().it, 20);
    }
}
