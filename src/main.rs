mod cloudwatch;
mod configuration;
mod ec2;

use chrono::Utc;
use configuration::Configuration;
use rusoto_logs::InputLogEvent;
use std::process::exit;
use std::sync::mpsc;
use systemd::journal::{Journal, JournalFiles, JournalRecord, JournalSeek};

fn get_record_timestamp_millis(record: &JournalRecord) -> i64 {
    if let Some(timestamp) = record.get("_SOURCE_REALTIME_TIMESTAMP") {
        if let Ok(timestamp) = timestamp.parse::<i64>() {
            // Convert microseconds to milliseconds
            return timestamp / 1000;
        }
    }
    // Fall back to current time
    Utc::now().timestamp_millis()
}

fn get_record_comm(record: &JournalRecord) -> &str {
    if let Some(comm) = record.get("_COMM") {
        comm
    } else {
        "unknown"
    }
}

fn parse_record(record: &JournalRecord) -> Option<InputLogEvent> {
    if let Some(message) = record.get("MESSAGE") {
        Some(InputLogEvent {
            message: format!(
                "{}: {}",
                get_record_comm(record),
                message.to_string()
            ),
            timestamp: get_record_timestamp_millis(record),
        })
    } else {
        None
    }
}

fn run_main_loop(
    conf: &Configuration,
    journal: &mut Journal,
    tx: mpsc::Sender<InputLogEvent>,
) {
    let wait_time = None;
    loop {
        match journal.await_next_record(wait_time) {
            Ok(Some(record)) => {
                conf.debug(format!("new record: {:?}", &record));
                if let Some(event) = parse_record(&record) {
                    if let Err(err) = tx.send(event) {
                        eprintln!("queue send failed: {}", err);
                    }
                }
            }
            Ok(None) => {}
            Err(err) => eprintln!("await_next_record failed: {}", err),
        }
    }
}

fn get_log_stream_name() -> String {
    match ec2::get_instance_id() {
        Ok(id) => match ec2::get_instance_name(id) {
            Ok(name) => {
                return name;
            }
            Err(err) => {
                println!("get_instance_name failed: {:?}", err);
            }
        },
        Err(err) => {
            println!("get_instance_id failed: {}", err);
        }
    }
    "unknown".to_string()
}

fn main() {
    let conf = Configuration::new(get_log_stream_name());
    let conf2 = conf.clone();
    let runtime_only = false;
    let local_only = false;
    let (tx, rx) = mpsc::channel();
    let uploader =
        std::thread::spawn(move || cloudwatch::upload_thread(conf2, rx));
    match Journal::open(JournalFiles::All, runtime_only, local_only) {
        Ok(mut journal) => {
            // Move to the end of the message log
            if let Err(err) = journal.seek(JournalSeek::Tail) {
                eprintln!("failed to seek to tail: {}", err);
            }

            run_main_loop(&conf, &mut journal, tx);
        }
        Err(err) => {
            eprintln!("failed to open journal: {}", err);
            exit(1);
        }
    }
    if let Err(err) = uploader.join() {
        eprintln!("join failed: {:?}", err);
    }
}
