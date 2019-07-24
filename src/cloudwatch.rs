use crate::configuration::Configuration;
use rusoto_core::Region;
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, CreateLogStreamRequest,
    DescribeLogStreamsRequest, InputLogEvent, LogStream, PutLogEventsRequest,
};

fn get_log_stream(
    client: &CloudWatchLogsClient,
    conf: &Configuration,
) -> Option<LogStream> {
    let result = client
        .describe_log_streams(DescribeLogStreamsRequest {
            log_group_name: conf.log_group_name.clone(),
            log_stream_name_prefix: Some(conf.log_stream_name.clone()),
            limit: Some(1),
            ..Default::default()
        })
        .sync();
    match result {
        Ok(result) => {
            if let Some(log_streams) = result.log_streams {
                if let Some(log_stream) = log_streams.first() {
                    if log_stream.log_stream_name == Some(conf.log_stream_name.clone())
                    {
                        return Some(log_stream.clone());
                    }
                }
            }
            None
        }
        Err(_) => {
            None
        }
    }
}

fn create_log_stream(
    client: &CloudWatchLogsClient,
    conf: &Configuration,
) {
    if let Err(err) = client
        .create_log_stream(CreateLogStreamRequest {
            log_group_name: conf.log_group_name.clone(),
            log_stream_name: conf.log_stream_name.clone(),
        })
        .sync()
    {
        eprintln!("failed to create log stream: {}", err);
    }
}

fn get_initial_sequence_token(
    client: &CloudWatchLogsClient,
    conf: &Configuration,
) -> Option<String> {
    let mut log_stream = get_log_stream(&client, conf);
    if log_stream.is_none() {
        create_log_stream(&client, conf);
        log_stream = get_log_stream(&client, conf);
    }

    if let Some(log_stream) = log_stream {
        if let Some(token) = log_stream.upload_sequence_token {
            Some(token)
        } else {
            eprintln!(
                "log stream {}/{} exists but is missing upload_sequence_token",
                conf.log_group_name, conf.log_stream_name
            );
            None
        }
    } else {
        eprintln!(
            "log stream {}/{} does not exist",
            &conf.log_group_name, &conf.log_stream_name
        );
        None
    }
}

pub struct CloudWatch {
    client: CloudWatchLogsClient,
    sequence_token: String,
}

impl CloudWatch {
    pub fn new(conf: &Configuration) -> Option<CloudWatch> {
        let client = CloudWatchLogsClient::new(Region::default());
        if let Some(token) = get_initial_sequence_token(&client, conf) {
            Some(CloudWatch {
                client,
                sequence_token: token,
            })
        } else {
            None
        }
    }

    pub fn upload(&mut self, conf: &Configuration, message: String) {
        let result = self.client
            .put_log_events(PutLogEventsRequest {
                log_events: vec![InputLogEvent {
                    message,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                }],
                log_group_name: conf.log_group_name.clone(),
                log_stream_name: conf.log_stream_name.clone(),
                sequence_token: Some(self.sequence_token.clone()),
            })
            .sync();
        match result {
            Ok(result) => {
                if let Some(token) = result.next_sequence_token {
                    self.sequence_token = token;
                } else {
                    eprintln!("missing sequence token");
                }
            }
            Err(err) => {
                eprintln!("send_to_cloudwatch failed: {}", err);
            }
        }
    }
}
