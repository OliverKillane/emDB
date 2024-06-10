use super::*;

emql! {
    impl emdb_table_thunderdome_impl as Serialized{
        interface = data_logs,
        pub = on,
        ds_name = EmDBThunderdome,
        op_impl = Iter,
        table_select = Thunderdome,
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

    query get_errors_per_minute() {
        use logs
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

    query get_comment_summaries(time_start: usize, time_end: usize) {
        use logs
            |> filter(*timestamp >= time_start && *timestamp <= time_end && comment.is_some())
            |> map(
                length: usize = comment.as_ref().unwrap().len(),
                slice: String = comment.unwrap().chars().take(30).collect::<String>()
            )
            |> collect(comments)
            ~> return;
    }

    query demote_error_logs() {
        ref logs as log_ref
            |> deref(log_ref as log_data)
            |> update(log_ref use level = (
                if crate::data_logs::LogLevel::Error == log_data.level {
                    crate::data_logs::LogLevel::Warning
                } else {
                    log_data.level
                }
            ));
    }
}