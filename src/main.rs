extern crate chrono;
extern crate itertools;
extern crate sha2;
#[macro_use] extern crate quick_error;
extern crate clap;

mod transaction;
mod transaction_log;

use clap::{App, Arg};

use transaction::Transaction;
use transaction_log::*;

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
        .unwrap_or("/tmp/transaction_log.tmp");
    let mut tx_log = SimpleFileLog::new(file_path);
    for _ in 0..n {
        let tx = Transaction::build()
            .with_id(tx_log.next_id().unwrap())
            .with_current_timestamp()
            .with_group_id(gid)
            .with_process_id(pid)
            .with_text("testü".to_owned())
            .try_finish_with_log(&tx_log)
            .unwrap();
        println!("appending {}", tx.to_string());
        //println!("pre append log: {:#?}", &tx_log);
        tx_log.append(tx).unwrap();
        //println!("post append log: {:#?}", &tx_log);
        //println!("###########################################################");
    }
    println!("Last tx: {}", tx_log.last().unwrap().unwrap().to_string());
}

