mod cloudwatch;
mod configuration;
mod ec2;

use configuration::Configuration;
use rusoto_logs::InputLogEvent;
use std::process::exit;
use std::sync::mpsc;
use systemd::journal::{Journal, JournalFiles, JournalRecord, JournalSeek};

fn parse_record(record: &JournalRecord) -> Option<InputLogEvent> {
    let message = record.get("MESSAGE");
    let timestamp = record.get("_SOURCE_REALTIME_TIMESTAMP");

    if let (Some(message), Some(timestamp)) = (message, timestamp) {
        if let Ok(timestamp) = timestamp.parse::<i64>() {
            Some(InputLogEvent {
                message: message.to_string(),
                timestamp,
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn run_main_loop(journal: &mut Journal, tx: mpsc::Sender<InputLogEvent>) {
    let wait_time = None;
    loop {
        match journal.await_next_record(wait_time) {
            Ok(Some(record)) => {
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
    let runtime_only = false;
    let local_only = false;
    let (tx, rx) = mpsc::channel();
    let uploader =
        std::thread::spawn(move || cloudwatch::upload_thread(conf, rx));
    match Journal::open(JournalFiles::All, runtime_only, local_only) {
        Ok(mut journal) => {
            // Move to the end of the message log
            if let Err(err) = journal.seek(JournalSeek::Tail) {
                eprintln!("failed to seek to tail: {}", err);
            }

            run_main_loop(&mut journal, tx);
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
