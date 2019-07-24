use std::env::var;

pub struct Configuration {
    pub log_group_name: String,
    pub log_stream_name: String,
}

impl Configuration {
    pub fn new(log_stream_name: String) -> Configuration {
        Configuration {
            log_group_name: var("LOG_GROUP_NAME")
                .unwrap_or("journald-to-cloudwatch".to_string()),
            log_stream_name,
        }
    }
}
