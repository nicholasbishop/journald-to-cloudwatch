mod cloudwatch;
mod configuration;
mod ec2;

use cloudwatch::CloudWatch;
use configuration::Configuration;
use std::process::exit;
use systemd::journal::{Journal, JournalFiles, JournalRecord, JournalSeek};

fn upload_record(
    conf: &Configuration,
    cloudwatch: &mut CloudWatch,
    record: &JournalRecord,
) {
    if let Some(message) = record.get("MESSAGE") {
        cloudwatch.upload(conf, message.to_string());
    }
}

fn run_main_loop(
    conf: &Configuration,
    cloudwatch: &mut CloudWatch,
    journal: &mut Journal,
) {
    let wait_time = None;
    loop {
        match journal.await_next_record(wait_time) {
            Ok(Some(record)) => upload_record(conf, cloudwatch, &record),
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
    let mut cloudwatch = CloudWatch::new(&conf);
    let runtime_only = false;
    let local_only = false;
    match Journal::open(JournalFiles::All, runtime_only, local_only) {
        Ok(mut journal) => {
            // Move to the end of the message log
            if let Err(err) = journal.seek(JournalSeek::Tail) {
                eprintln!("failed to seek to tail: {}", err);
            }

            run_main_loop(&conf, &mut cloudwatch, &mut journal);
        }
        Err(err) => {
            eprintln!("failed to open journal: {}", err);
            exit(1);
        }
    }
}
