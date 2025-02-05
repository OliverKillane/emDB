use emdb::macros::emql;
use data_logs::Datastore;

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
    impl data_logs_iter as Serialized{
        interface = data_logs,
        pub = on,
        aggressive_inlining = on,
        op_impl = Iter,
    };

    table logs {
        timestamp: usize,
        comment: Option<String>,
        level: crate::valid::complex::data_logs::LogLevel,
    }

    query add_event(
        timestamp: usize,
        comment: Option<String>,
        log_level: crate::valid::complex::data_logs::LogLevel,
    ) {
        row(
            timestamp: usize = timestamp,
            comment: Option<String> = comment,
            level: crate::valid::complex::data_logs::LogLevel = log_level,
        ) ~> insert(logs as ref log_id);
    }

    // Description:
    //   Get the number of errors per minute counter
    // Reasoning:
    //   Requires a large mapping (accelerated by parallelism), and a groupby
    //   aggregation. For demonstrating OLAP performance.
    query get_errors_per_minute() {
        use logs as (timestamp, level)
            |> filter(*level == crate::valid::complex::data_logs::LogLevel::Error)
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
                if crate::valid::complex::data_logs::LogLevel::Error == log_data.level {
                    crate::valid::complex::data_logs::LogLevel::Warning
                } else {
                    log_data.level
                }
            ));
    }
}

pub fn test() {
    let mut ds = data_logs_iter::Datastore::new();
    let db = ds.db();
    // TODO: Add some more tests for correctness
}