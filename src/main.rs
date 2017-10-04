
extern crate sha2;
extern crate chrono;
extern crate itertools;

use itertools::Itertools;
use chrono::prelude::*;
use sha2::{Sha256, Digest};

#[derive(Debug)]
struct Transaction<Tz: TimeZone> {
    id: u32,
    timestamp: DateTime<Tz>,
    group_id: u8,
    process_id: u8,
    text: String,
    checksum: Vec<u8>,
}

impl Transaction<chrono::FixedOffset> {
    pub fn new(
        prev: &Vec<Transaction<chrono::FixedOffset>>,
        id: u32,
        time: DateTime<chrono::FixedOffset>,
        gid: u8,
        pid: u8,
        text: &str,
    ) -> Result<Transaction<chrono::FixedOffset>, String> {

        let max_gid = 99;
        let max_pid = 99;
        let max_id = 99_999_999;

        if gid > max_gid {
            return Err(format!("Invalid group id {}", gid));
        }
        if pid > max_pid {
            return Err(format!("Invalid process id {}", pid));
        }

        let msg = text.replace("\n", "").replace("\r", "").replace(";", "");

        let mut hasher = Sha256::default();
        for t in prev.iter() {
            hasher.input(t.to_string().into_bytes().as_ref());
        }


        hasher.input(Self::partial_string(id, &time, gid, pid, &msg)
            .into_bytes().as_ref());

        let checksum = Vec::from(hasher.result().as_slice());

        Ok(Transaction {
            id: id,
            timestamp: time,
            group_id: gid,
            process_id: pid,
            text: msg,
            checksum: checksum,
        })
    }

    fn partial_string(id: u32, ts: &DateTime<chrono::FixedOffset>, gid: u8, pid: u8, text: &str) -> String {
        format!("{id:08};{ts};{gid:02};{pid:02};{msg};",
            id = id,
            ts = ts.format("%d%m%y-%H:%M:%S"),
            gid = gid,
            pid = pid,
            msg = text,
        )
    }

    pub fn to_string(&self) -> String {
        format!("{partial}{chksm:02X}\n",
            partial = Self::partial_string(self.id, &self.timestamp, self.group_id, self.process_id, &self.text),
            chksm = self.checksum.iter().format(""),
        )
    }

}


fn main() {
    let mut transactions = Vec::new();

    let t1 = Transaction::new(
        &vec![],
        1,
        DateTime::parse_from_rfc3339("2017-10-04T10:00:00+01:00").unwrap(),
        0,
        1,
        "Testü"
        ).unwrap();
    println!("{:#?}", t1.to_string());
    transactions.push(t1);

    let t2 = Transaction::new(
        &transactions,
        2,
        DateTime::parse_from_rfc3339("2017-10-04T10:01:00+01:00").unwrap(),
        0,
        1,
        "Test2"
        ).unwrap();

    println!("{:#?}", t2.to_string());
}
