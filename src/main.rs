mod cloudwatch;
mod configuration;
mod ec2;

use configuration::Configuration;
use cloudwatch::CloudWatch;
use std::process::exit;
use systemd::journal::{Journal, JournalFiles, JournalRecord};

fn upload_record(conf: &Configuration, cloudwatch: &mut CloudWatch, record: &JournalRecord) {
    if let Some(message) = record.get("MESSAGE") {
        cloudwatch.upload(conf, message.to_string());
    }
}

fn run_main_loop(conf: &Configuration, cloudwatch: &mut CloudWatch, journal: &mut Journal) {
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
        Ok(id) => {
            match ec2::get_instance_name(id) {
                Ok(name) => { return name; }
                Err(err) => {
                    println!("get_instance_name failed: {:?}", err);
                }
            }
        }
        Err(err) => {
            println!("get_instance_id failed: {}", err);
        }
    }
    "unknown".to_string()
}

fn main() {
    let conf = Configuration::new(get_log_stream_name());
    if let Some(mut cloudwatch) = CloudWatch::new(&conf) {
        let runtime_only = true;
        let local_only = true;
        match Journal::open(JournalFiles::All, runtime_only, local_only) {
            Ok(mut journal) => {
                run_main_loop(&conf, &mut cloudwatch, &mut journal);
            }
            Err(err) => {
                eprintln!("failed to open journal: {}", err);
                exit(1);
            }
        }
    } else {
        eprintln!("failed to connect to cloudwatch");
    }
}
