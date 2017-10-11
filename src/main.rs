extern crate chrono;
extern crate itertools;
extern crate sha2;

mod transaction;
mod transaction_log;

//use std::io::BufRead;
//use std::io;

use transaction::Transaction;
use transaction_log::*;

fn main() {
    let n = 1_000_000;
    let mut tx_log = FullTransactionLog::with_capactity(n);
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

#[cfg(test)]
mod test {

    use super::*;
    use chrono::prelude::*;

    /*
    #[test]
    fn board_example() {
        let mut transactions = Vec::new();

        let e1 = "00000001;041017-10:00:00;00;01;Testü;267C4D5033ED7F96B43216FD8C871E4B96F1221204312AD6F43362F2D12C9B29\n";
        let t1 = Transaction::build()
            .with_id(1)
            .with_timestamp(
                DateTime::parse_from_rfc3339("2017-10-04T10:00:00+01:00").unwrap(),
            )
            .with_group_id(0)
            .with_process_id(1)
            .with_text("Testü".to_owned())
            .try_finish_with_prev(&transactions)
            .unwrap();
        let s1 = t1.to_string();
        assert_eq!(e1, s1);
        transactions.push(t1);

        let e2 = "00000002;041017-10:01:00;00;01;Test2;6CB09B876CA855D7F3D8168E2001594E354D2EFEC110F5D5FCED478641E96C9C\n";
        let t2 = Transaction::build()
            .with_id(2)
            .with_timestamp(
                DateTime::parse_from_rfc3339("2017-10-04T10:01:00+01:00").unwrap(),
            )
            .with_group_id(0)
            .with_process_id(1)
            .with_text("Test2".to_owned())
            .try_finish_with_prev(&transactions)
            .unwrap();
        let s2 = t2.to_string();
        assert_eq!(e2, s2);
        transactions.push(t2);
    }
    */

    #[test]
    fn parse1() {
        let input = "00000001;041017-10:00:00;00;01;Testü;267C4D5033ED7F96B43216FD8C871E4B96F1221204312AD6F43362F2D12C9B29\n";
        let parsed = Transaction::parse(input);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().to_string(), input);
    }

    #[test]
    fn parse2() {
        let t = Transaction::build()
            .with_id(42)
            .with_timestamp(
                DateTime::parse_from_rfc3339("2017-10-04T11:05:00+01:00")
                .unwrap()
                .with_timezone(&FixedOffset::east(Transaction::TZ_OFFSET)),
            )
            .with_group_id(43)
            .with_process_id(44)
            .with_text("Großes S".to_owned())
            .try_finish_with_log(&SingleTransactionLog::new())
            .unwrap();
        let parsed = Transaction::parse(&t.to_string());
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), t);
    }


}
