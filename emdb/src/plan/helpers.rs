//! Functions for building [super::repr::LogicalPlan]
use std::fmt::Display;

use quote::ToTokens;

use crate::plan::repr::{RecordData, ScalarType};

use super::repr::{LogicalPlan, LogicalTable, Record};

impl LogicalTable {
    pub(crate) fn get_all_cols_type(&self) -> Record {
        Record {
            fields: self
                .columns
                .iter()
                .map(|(n, lc)| {
                    (
                        n.clone(),
                        RecordData::Scalar(ScalarType::Rust(lc.data_type.clone())),
                    )
                })
                .collect(),
            stream: true,
        }
    }
}

pub(crate) struct WithPlan<'a, A> {
    pub plan: &'a LogicalPlan,
    pub extended: A,
}

impl<'a, 'b> Display for WithPlan<'a, &'b Record> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.extended.stream {
            write!(f, "|> ")?
        }
        write!(f, "{{")?;
        for (a, b) in self.extended.fields.iter() {
            write!(
                f,
                "{a}: {}, ",
                WithPlan {
                    plan: self.plan,
                    extended: b
                }
            )?;
        }
        write!(f, "}}")
    }
}

impl<'a, 'b> Display for WithPlan<'a, &'b ScalarType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.extended {
            ScalarType::Ref(t) => {
                // INV: the type is valid => the index is in the plan
                write!(f, "ref {}", self.plan.tables.get(*t).unwrap().name)
            }
            ScalarType::Rust(t) => t.to_token_stream().fmt(f),
            ScalarType::Bag(b) => {
                write!(f, "collected {{{}}}", WithPlan {
                    plan: self.plan,
                    extended: b
                })
            }
        }
    }
}

impl<'a, 'b> Display for WithPlan<'a, &'b RecordData> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.extended {
            RecordData::Record(r) => WithPlan {
                plan: self.plan,
                extended: r,
            }
            .fmt(f),
            RecordData::Scalar(s) => WithPlan {
                plan: self.plan,
                extended: s,
            }
            .fmt(f),
        }
    }
}
