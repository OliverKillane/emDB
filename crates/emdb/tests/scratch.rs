#![allow(dead_code, unused_variables)]
//! For manually debugging generated code.
//!
//! - Ensure that the proc macro is built. In vscode on the bottom bar you can
//!   hover over `rust-analyzer` and click `Rebuild Proc Macros`
//! - Saving this file should re-run the emql macro, to generate outputs.
use emdb::macros::emql;

#[derive(Debug, Clone, Copy)]
enum RGB {
    Red,
    Blue,
    Green,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
}

emql! {
    impl my_interface as Interface{
        traits_with_db = { },
    };
    impl my_db as Serialized{
        // debug_file = "emdb/tests/code.rs",
        // interface = my_interface,
        // pub = on,
        ds_name = EmDBDebug,
        aggressive_inlining = on,
    };
    impl code_display as PlanViz{
        path = "emdb/tests/debug/code.dot",
        types = off,
        ctx = on,
        control = on,
    };

    table logs {
        timestamp: u64,
        comment: Option<String>,
        level: crate::LogLevel,
    }

    query add_event(
        timestamp: u64,
        comment: Option<String>,
        log_level: crate::LogLevel,
    ) {
        row(
            timestamp: u64 = timestamp,
            comment: Option<String> = comment,
            level: crate::LogLevel = log_level,
        ) ~> insert(logs as ref log_id);
    }

    query get_errors_per_minute() {
        use logs
            |> filter(*level == crate::LogLevel::Error)
            |> map(min: u64 = timestamp % 60)
            |> groupby(min for let errors in {
                use errors
                    |> count(num_logs)
                    ~> map(min: u64 = min, errors: usize = num_logs)
                    ~> return;
            })
            |> collect(errors)
            ~> return;
    }

    query get_comment_summaries(time_start: u64, time_end: u64) {
        use logs
            |> filter(**timestamp >= time_start && **timestamp <= time_end && comment.is_some())
            |> map(
                comment: &'db str = &comment.as_ref().unwrap()[..100],
                length: usize = comment.as_ref().unwrap().len()
            )
            |> collect(comments)
            ~> return;
    }

    query demote_error_logs() {
        ref logs as log_ref
            |> deref(log_ref as log_data)
            |> update(log_ref use level = (
                if crate::LogLevel::Error == log_data.level { crate::LogLevel::Warning } else { log_data.level.clone() }
            ));
    }
}

fn main() {
    // use my_interface::Datastore;
    // let mut ds = my_db::EmDBDebug::new();
    // let db = ds.db();
}
