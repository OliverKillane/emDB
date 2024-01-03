//! A naive implementation for comparison with one using optimisations only possible when knowing the full set of queries.
//!
//! ## Assumptions
//! - We allow the id to be generated, and assume fast id lookup is required
//! - We assume deletions may be needed at some point in the future
use super::{AddCreditsErr, IDNotFoundErr, PremCreditsErr, UserDetails, UserInfo};
use typed_generational_arena::{Arena, Index};

fn premcred_predicate(premium: bool, credits: i32) -> bool {
    premium || credits >= 0
}

pub struct Record {
    name: String,
    premium: bool,
    credits: i32,
}

pub struct NaiveDatabase {
    users: Arena<Record>,
}

impl<'a> UserDetails<'a> for NaiveDatabase {
    type UsersID = Index<Record, usize, usize>;
    type ReturnString = String;

    fn new() -> Self {
        NaiveDatabase {
            users: Arena::new(),
        }
    }

    fn new_user(&mut self, username: String, prem: bool) -> Self::UsersID {
        self.users.insert(Record {
            name: username,
            premium: prem,
            credits: 0,
        })
    }
    fn get_info(
        &self,
        user_id: Self::UsersID,
    ) -> Result<UserInfo<Self::ReturnString, Self::UsersID>, IDNotFoundErr> {
        if let Some(rec) = self.users.get(user_id) {
            Ok(UserInfo {
                id: user_id,
                name: rec.name.clone(),
                premium: rec.premium,
                credits: rec.credits,
            })
        } else {
            Err(IDNotFoundErr)
        }
    }
    fn get_snapshot(&self) -> Vec<UserInfo<Self::ReturnString, Self::UsersID>> {
        self.users
            .iter()
            .map(|(id, rec)| UserInfo {
                id,
                name: rec.name.clone(),
                premium: rec.premium,
                credits: rec.credits,
            })
            .collect::<Vec<_>>()
    }
    fn add_credits(&mut self, user: Self::UsersID, creds: i32) -> Result<(), AddCreditsErr> {
        if let Some(rec) = self.users.get_mut(user) {
            if premcred_predicate(rec.premium, rec.credits + creds) {
                rec.credits += creds;
                Ok(())
            } else {
                Err(AddCreditsErr::PremCredits)
            }
        } else {
            Err(AddCreditsErr::IDNotFound)
        }
    }
    fn reward_premium(&mut self, cred_bonus: f32) -> Result<i32, PremCreditsErr> {
        for (_, v) in self.users.iter_mut() {
            if !premcred_predicate(v.premium, ((v.credits as f32) * cred_bonus) as i32) {
                return Err(PremCreditsErr);
            }
        }
        let mut total_creds = 0;
        for (_, v) in self.users.iter_mut() {
            let new_creds = ((v.credits as f32) * cred_bonus) as i32;
            total_creds += new_creds;
            v.credits = new_creds;
        }

        Ok(total_creds)
    }
    fn total_premium_credits(&self) -> i64 {
        self.users
            .iter()
            .filter(|(_, v)| v.premium)
            .map(|(_, v)| v.credits as i64)
            .sum()
    }
}
