use std::env::var;

#[derive(Clone, Debug)]
pub struct Configuration {
    pub log_group_name: String,
    pub log_stream_name: String,
    pub is_debug_mode_enabled: bool,
}

impl Configuration {
    pub fn new(log_stream_name: String) -> Configuration {
        Configuration {
            log_group_name: var("LOG_GROUP_NAME")
                .unwrap_or("journald-to-cloudwatch".to_string()),
            log_stream_name,
            is_debug_mode_enabled: var("DEBUG").is_ok(),
        }
    }

    pub fn path(&self) -> String {
        format!("{}/{}", self.log_group_name, self.log_stream_name)
    }

    pub fn debug(&self, message: String) {
        if self.is_debug_mode_enabled {
            eprintln!("{}", message);
        }
    }
}
