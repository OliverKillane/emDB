use crate::utils::{choose, choose_internal, total};
use data_logs::Database;
use emdb::macros::emql;
use rand::{rngs::ThreadRng, Rng};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
}

emql! {
    impl data_logs as Interface{
        pub = on,
    };
    impl emdb_impl as Serialized{
        interface = data_logs,
        pub = on,
        ds_name = EmDB,
        aggressive_inlining = on,
        op_impl = Iter,
    };

    table logs {
        timestamp: usize,
        comment: Option<String>,
        level: crate::data_logs::LogLevel,
    }

    query add_event(
        timestamp: usize,
        comment: Option<String>,
        log_level: crate::data_logs::LogLevel,
    ) {
        row(
            timestamp: usize = timestamp,
            comment: Option<String> = comment,
            level: crate::data_logs::LogLevel = log_level,
        ) ~> insert(logs as ref log_id);
    }

    // Description:
    //   Get the number of errors per minute counter
    // Reasoning:
    //   Requires a large mapping (accellerated by parallelism), and a groupby
    //   aggregation. For demonstrating OLAP performance.
    query get_errors_per_minute() {
        use logs as (timestamp, level)
            |> filter(*level == crate::data_logs::LogLevel::Error)
            |> map(min: usize = timestamp % 60)
            |> groupby(min for let errors in {
                use errors
                    |> count(num_logs)
                    ~> map(min: usize = min, errors: usize = num_logs)
                    ~> return;
            })
            |> collect(errors)
            ~> return;
    }

    // Description:
    //   Get the first 30 characters of each comment, and the length of the
    //   comment.
    // Reasoning:
    //   Requires a fast map over a large stream of values, a common OLAP workload.
    query get_comment_summaries(time_start: usize, time_end: usize) {
        use logs as (comment, timestamp)
            |> filter(**timestamp >= time_start && **timestamp <= time_end && comment.is_some())
            |> map(slice: &'db str = comment.as_ref().unwrap())
            |> map(
                comment: &'db str = &slice[..(std::cmp::min(30, slice.len()))],
                length: usize = slice.len()
            )
            |> collect(comments)
            ~> return;
    }

    // Description:
    //   Demote all errors to warnings.
    // Reasoning:
    //   A data cleaning workload.
    query demote_error_logs() {
        ref logs as log_ref
            |> deref(log_ref as log_data use level)
            |> update(log_ref use level = (
                if crate::data_logs::LogLevel::Error == log_data.level {
                    crate::data_logs::LogLevel::Warning
                } else {
                    log_data.level
                }
            ));
    }
}

pub fn populate_table<DS: data_logs::Datastore>(rng: &mut ThreadRng, size: usize) -> DS {
    let mut ds = DS::new();
    {
        let mut db = ds.db();
        for t in 0..size {
            db.add_event(
                t,
                choose! { rng
                  1 => None,
                  1 => Some({
                    random_string(rng)
                  }),
                },
                choose! { rng
                  1 => LogLevel::Error,
                  2 => LogLevel::Warning,
                  2 => LogLevel::Info,
                },
            );
        }
    }
    ds
}

pub fn random_string(rng: &mut ThreadRng) -> String {
    let size = rng.gen_range(0..1024);
    let mut s = String::with_capacity(size);
    for _ in 0..size {
        s.push(rng.gen_range(b'a'..b'z') as char);
    }
    s
}

pub mod duckdb_impl;
pub mod sqlite_impl;
mod copy_selector_emdb_impl; pub use copy_selector_emdb_impl::*;
