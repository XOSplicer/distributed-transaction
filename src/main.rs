extern crate chrono;
extern crate itertools;
extern crate sha2;
#[macro_use] extern crate quick_error;
extern crate clap;

mod transaction;
mod transaction_log;
mod transaction_queue;

use clap::{App, Arg};

use transaction_log::*;
use transaction_queue::*;

fn main() {
    let arg_matches = App::new("transaction")
        .arg(Arg::with_name("LOG_FILE")
            .required(true)
            .index(1))
        .arg(Arg::with_name("pid")
            .long("pid")
            .takes_value(true))
        .arg(Arg::with_name("gid")
            .long("gid")
            .takes_value(true))
        .get_matches();

    let n = 20;
    let gid = arg_matches.value_of("gid")
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(0);
    let pid = arg_matches.value_of("pid")
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(0);
    let file_path = arg_matches.value_of("LOG_FILE")
        .unwrap_or("/tmp/transaction_log.tmp").to_owned();
    let queue = TransactionQueue::new(SimpleFileLog::new(file_path));
    let q = queue.sender();
    for _ in 0..n {
        q.send(QueueMessage::Queue(QueuedTransaction {
            text: "ÄÜÖ".to_owned(),
            gid: gid,
            pid: pid,
        })).unwrap();
    }
    q.send(QueueMessage::Flush).unwrap();
    q.send(QueueMessage::Finish).unwrap();
    queue.join().unwrap();
}

