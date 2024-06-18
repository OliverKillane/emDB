//! Implementation using the knowledge of all possible queries.

use super::{AddCreditsErr, IDNotFoundErr, PremCreditsErr, UserDetails, UserInfo};
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

impl<'a> UserDetails<'a> for ForesightDatabase<'a> {
    type UsersID = usize;
    type ReturnString = &'a str;

    fn new() -> Self {
        Self {
            non_prem_user: Slab::new(),
            prem_user: Slab::new(),
            premium_credits: 0,
            _marker: PhantomData,
        }
    }

    fn new_user(&mut self, username: String, prem: bool) -> Self::UsersID {
        if prem {
            let id = self.prem_user.insert(Record {
                name: username.into(),
                credits: 0,
            });

            table_to_id(true, id)
        } else {
            let id = self.non_prem_user.insert(Record {
                name: username.into(),
                credits: 0,
            });
            table_to_id(false, id)
        }
    }
    fn get_info(
        &self,
        user_id: usize,
    ) -> Result<UserInfo<Self::ReturnString, Self::UsersID>, IDNotFoundErr> {
        let (premium, id) = id_to_table(user_id);
        let rec = if premium {
            self.prem_user.get(id)
        } else {
            self.non_prem_user.get(id)
        };
        match rec {
            Some(rec) => Ok(UserInfo {
                id: user_id,
                name: unsafe { rec.name.get().as_ref().unwrap() }, // given we do not deallocate, and it is not mutated
                premium,
                credits: rec.credits,
            }),
            None => Err(IDNotFoundErr),
        }
    }
    fn get_snapshot(&self) -> Vec<UserInfo<Self::ReturnString, Self::UsersID>> {
        let mut it: Vec<UserInfo<Self::ReturnString, Self::UsersID>> =
            Vec::with_capacity(self.non_prem_user.len() + self.prem_user.len());
        for (id, rec) in self.non_prem_user.iter() {
            it.push(UserInfo {
                id: table_to_id(false, id),
                name: unsafe { rec.name.get().as_ref::<'a>().unwrap() },
                premium: false,
                credits: rec.credits,
            });
        }
        for (id, rec) in self.prem_user.iter() {
            it.push(UserInfo {
                id: table_to_id(true, id),
                name: unsafe { rec.name.get().as_ref::<'a>().unwrap() },
                premium: true,
                credits: rec.credits,
            });
        }
        it
    }
    fn add_credits(&mut self, user: usize, creds: i32) -> Result<(), AddCreditsErr> {
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
                    Ok(())
                } else {
                    Err(AddCreditsErr::PremCredits)
                }
            }
            None => Err(AddCreditsErr::IDNotFound),
        }
    }
    fn reward_premium(&mut self, cred_bonus: f32) -> Result<i64, PremCreditsErr> {
        for (_, v) in self.prem_user.iter_mut() {
            if !premcred_predicate(true, ((v.credits as f32) * cred_bonus) as i32) {
                return Err(PremCreditsErr);
            }
        }
        let mut cred_diff = 0;
        for (_, v) in self.prem_user.iter_mut() {
            let new_creds = ((v.credits as f32) * cred_bonus) as i32;
            cred_diff += (new_creds - v.credits) as i64;
            v.credits = new_creds;
        }
        self.premium_credits += cred_diff;

        Ok(cred_diff)
    }
    fn total_premium_credits(&self) -> i64 {
        self.premium_credits
    }
}
