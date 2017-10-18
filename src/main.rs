extern crate chrono;
extern crate itertools;
extern crate sha2;

mod transaction;
mod transaction_log;

use transaction::Transaction;
use transaction_log::*;

fn main() {
    let n = 100_000;
    //let mut tx_log = FullTransactionLog::with_capactity(n);
    let mut tx_log = DirectFileLog::new("/tmp/tx_log_tmp").unwrap();
    let gid = 0;
    let pid = 1;
    for _ in 0..n {
        let tx = Transaction::build()
            .with_id(tx_log.next_id())
            .with_current_timestamp()
            .with_group_id(gid)
            .with_process_id(pid)
            .with_text("".to_owned())
            .try_finish_with_log(&tx_log)
            .unwrap();
        tx_log.append(tx).unwrap();
    }
    println!("{}", tx_log.last().unwrap().to_string());
}

