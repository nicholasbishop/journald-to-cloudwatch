use crate::configuration::Configuration;
use chrono::{DateTime, Utc};
use rusoto_core::Region;
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, CreateLogStreamRequest,
    DescribeLogStreamsRequest, InputLogEvent, LogStream, PutLogEventsRequest,
};
use std::sync::mpsc;
use std::time::Duration;

struct CloudWatch {
    client: CloudWatchLogsClient,
    sequence_token: Option<String>,
    conf: Configuration,
}

impl CloudWatch {
    fn new(conf: Configuration) -> CloudWatch {
        let client = CloudWatchLogsClient::new(Region::default());
        let mut cw = CloudWatch {
            sequence_token: None,
            client,
            conf,
        };
        cw.update_sequence_token();
        cw
    }

    fn get_log_stream(&self) -> Option<LogStream> {
        let result = self
            .client
            .describe_log_streams(DescribeLogStreamsRequest {
                log_group_name: self.conf.log_group_name.clone(),
                log_stream_name_prefix: Some(self.conf.log_stream_name.clone()),
                limit: Some(1),
                ..Default::default()
            })
            .sync();
        match result {
            Ok(result) => {
                if let Some(log_streams) = result.log_streams {
                    if let Some(log_stream) = log_streams.first() {
                        if log_stream.log_stream_name
                            == Some(self.conf.log_stream_name.clone())
                        {
                            return Some(log_stream.clone());
                        }
                    }
                }
                None
            }
            Err(_) => None,
        }
    }

    fn create_log_stream(&self) {
        if let Err(err) = self
            .client
            .create_log_stream(CreateLogStreamRequest {
                log_group_name: self.conf.log_group_name.clone(),
                log_stream_name: self.conf.log_stream_name.clone(),
            })
            .sync()
        {
            eprintln!("failed to create log stream: {}", err);
        }
    }

    fn update_sequence_token(&mut self) {
        let mut log_stream = self.get_log_stream();
        if log_stream.is_none() {
            self.create_log_stream();
            log_stream = self.get_log_stream();
        }

        if let Some(log_stream) = log_stream {
            self.sequence_token = log_stream.upload_sequence_token;
        } else {
            eprintln!("log stream {} does not exist", self.conf.path());
        }
    }

    fn upload(&mut self, events: Vec<InputLogEvent>) {
        let result = self
            .client
            .put_log_events(PutLogEventsRequest {
                log_events: events,
                log_group_name: self.conf.log_group_name.clone(),
                log_stream_name: self.conf.log_stream_name.clone(),
                sequence_token: self.sequence_token.clone(),
            })
            .sync();
        match result {
            Ok(result) => {
                self.sequence_token = result.next_sequence_token;
            }
            Err(err) => {
                eprintln!("send_to_cloudwatch failed: {}", err);
                self.update_sequence_token();
            }
        }
    }
}

/// Calculate the number of bytes this message requires as counted
/// by the PutLogEvents API.
///
/// Reference:
/// docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_PutLogEvents.html
fn get_event_num_bytes(event: &InputLogEvent) -> usize {
    event.message.len() + 26
}

struct UploadThreadState {
    cloudwatch: CloudWatch,
    events: Vec<InputLogEvent>,
    first_timestamp: Option<i64>,
    last_timestamp: Option<i64>,
    num_pending_bytes: usize,
    last_flush_time: Option<DateTime<Utc>>,
}

impl UploadThreadState {
    fn new(conf: Configuration) -> UploadThreadState {
        UploadThreadState {
            cloudwatch: CloudWatch::new(conf),
            events: Vec::new(),
            first_timestamp: None,
            last_timestamp: None,
            num_pending_bytes: 0,
            last_flush_time: None,
        }
    }

    fn push(&mut self, event: InputLogEvent) {
        self.cloudwatch
            .conf
            .debug("upload thread event received".to_string());

        // Flush if the latest event's timestamp is older than the
        // previous event
        if let Some(last_timestamp) = self.last_timestamp {
            if event.timestamp < last_timestamp {
                self.flush();
            }
        }

        // Flush if the maximum size (in bytes) of events has been reached
        let max_bytes = 1048576;
        let event_num_bytes = get_event_num_bytes(&event);
        if self.num_pending_bytes + event_num_bytes > max_bytes {
            self.flush();
        }

        // Flush if the maximum number of events has been reached
        let max_events = 10000;
        if self.events.len() + 1 >= max_events {
            self.flush();
        }

        // Add the event to the pending events
        if self.first_timestamp.is_none() {
            self.first_timestamp = Some(event.timestamp);
        }
        self.last_timestamp = Some(event.timestamp);
        self.num_pending_bytes += event_num_bytes;
        self.events.push(event);
    }

    /// Upload all pending events to CloudWatch Logs
    fn flush(&mut self) {
        self.cloudwatch
            .conf
            .debug(format!("flush: {}", self.summary()));

        if self.events.is_empty() {
            return;
        }

        let mut events = Vec::new();
        std::mem::swap(&mut events, &mut self.events);
        self.cloudwatch.upload(events);
        self.first_timestamp = None;
        self.last_timestamp = None;
        self.num_pending_bytes = 0;

        self.last_flush_time = Some(Utc::now());
    }

    fn summary(&self) -> String {
        format!("events.len()={}, last_timestamp={:?}, num_pending_bytes={}, last_flush_time={:?}",
                self.events.len(),
                self.last_timestamp, self.num_pending_bytes, self.last_flush_time)
    }
}

pub fn upload_thread(conf: Configuration, rx: mpsc::Receiver<InputLogEvent>) {
    conf.debug("upload thread started".to_string());
    let mut state = UploadThreadState::new(conf.clone());
    loop {
        conf.debug(format!("upload thread state: {}", state.summary()));

        if let Ok(record) = rx.recv_timeout(Duration::from_secs(1)) {
            state.push(record);
        }

        // If we have "old" records, flush now
        if let Some(first_timestamp) = state.first_timestamp {
            if Utc::now().timestamp_millis() - first_timestamp > 1000 {
                state.flush();
            }
        }
    }
}
