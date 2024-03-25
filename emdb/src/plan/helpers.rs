//! Functions for building [super::repr::LogicalPlan]
use crate::plan::repr::RecordData;

use super::repr::{LogicalTable, Record};

impl LogicalTable {
    pub(crate) fn get_all_cols_type(&self) -> Record {
        Record {
            fields: self
                .columns
                .iter()
                .map(|(n, lc)| (n.clone(), RecordData::Rust(lc.data_type.clone())))
                .collect(),
            stream: true,
        }
    }
}
