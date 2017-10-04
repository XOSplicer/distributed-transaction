
extern crate chrono;
extern crate itertools;
extern crate sha2;

use itertools::Itertools;
use chrono::prelude::*;
use sha2::{Digest, Sha256};

#[derive(Debug)]
struct Transaction<Tz: TimeZone> {
    id: u32,
    timestamp: DateTime<Tz>,
    group_id: u8,
    process_id: u8,
    text: String,
    checksum: Vec<u8>,
}

struct TransactionBuilder<Tz: TimeZone> {
    id: Option<u32>,
    timestamp: Option<DateTime<Tz>>,
    group_id: Option<u8>,
    process_id: Option<u8>,
    text: Option<String>,
}

impl Transaction<chrono::FixedOffset> {

    const MIN_ID: u32 = 1;
    const MAX_ID: u32 = 99_999_999;
    const MIN_GID: u8 = 0;
    const MAX_GID: u8 = 99;
    const MIN_PID: u8 = 0;
    const MAX_PID: u8 = 99;
    const INVALID_CHAR: &'static[&'static str] = &["\n", "\r", "\t", ";", "\0"];

    pub fn build() -> TransactionBuilder<chrono::FixedOffset> {
        TransactionBuilder::new()
    }

    pub fn new(
        prev: &Vec<Transaction<chrono::FixedOffset>>,
        id: u32,
        time: DateTime<chrono::FixedOffset>,
        gid: u8,
        pid: u8,
        text: String,
    ) -> Result<Transaction<chrono::FixedOffset>, String> {

        // Check input parameters
        if id < Transaction::MIN_ID || id > Transaction::MAX_ID {
            return Err(format!("Invalid id {}", id));
        }
        if gid < Transaction::MIN_GID || gid > Transaction::MAX_GID {
            return Err(format!("Invalid group id {}", gid));
        }
        if pid < Transaction::MIN_PID || pid > Transaction::MAX_PID {
            return Err(format!("Invalid process id {}", pid));
        }
        if Transaction::INVALID_CHAR.iter().any(|c| text.contains(c)) {
            return Err(format!("Invalid text message `{}`", text));
        }

        // Hash all previous transaction strings
        // and the partial new transaction
        let mut hasher = Sha256::default();
        for t in prev.iter() {
            hasher.input(t.to_string().into_bytes().as_ref());
        }
        hasher.input(
            Self::partial_string(id, &time, gid, pid, &text)
                .into_bytes()
                .as_ref(),
        );
        let checksum = Vec::from(hasher.result().as_slice());

        Ok(Transaction {
            id: id,
            timestamp: time,
            group_id: gid,
            process_id: pid,
            text: text,
            checksum: checksum,
        })
    }

    fn partial_string(
        id: u32,
        ts: &DateTime<chrono::FixedOffset>,
        gid: u8,
        pid: u8,
        text: &str,
    ) -> String {
        format!(
            "{id:08};{ts};{gid:02};{pid:02};{msg};",
            id = id,
            ts = ts.format("%d%m%y-%H:%M:%S"),
            gid = gid,
            pid = pid,
            msg = text,
        )
    }

    pub fn to_string(&self) -> String {
        format!(
            "{partial}{chksm:02X}\n",
            partial = Self::partial_string(
                self.id,
                &self.timestamp,
                self.group_id,
                self.process_id,
                &self.text
            ),
            chksm = self.checksum.iter().format(""),
        )
    }
}

impl TransactionBuilder<chrono::FixedOffset> {
    pub fn new() -> Self {
        TransactionBuilder {
            id: None,
            timestamp: None,
            group_id: None,
            process_id: None,
            text: None,
        }
    }

    pub fn with_id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_timestamp(mut self, ts: DateTime<chrono::FixedOffset>) -> Self {
        self.timestamp = Some(ts);
        self
    }

    pub fn with_group_id(mut self, gid: u8) -> Self {
        self.group_id = Some(gid);
        self
    }

    pub fn with_process_id(mut self, pid: u8) -> Self {
        self.process_id = Some(pid);
        self
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }


    pub fn try_finish(
        self,
        prev: &Vec<Transaction<chrono::FixedOffset>>,
    ) -> Result<Transaction<chrono::FixedOffset>, String> {
        Transaction::new(
            prev,
            self.id.ok_or("No id given".to_string())?,
            self.timestamp.ok_or("No timestamp given".to_string())?,
            self.group_id.ok_or("No group id given".to_string())?,
            self.process_id.ok_or("No process id given".to_string())?,
            self.text.unwrap_or("".to_string())
        )
    }
}


fn main() {
    let mut transactions = Vec::new();

    let t3 = Transaction::build()
        .with_id(1)
        .with_timestamp(DateTime::parse_from_rfc3339("2017-10-04T10:00:00+01:00").unwrap())
        .with_group_id(0)
        .with_process_id(1)
        .with_text("Testü".to_owned())
        .try_finish(&transactions)
        .unwrap();
    println!("{:#?}", t3.to_string());
    transactions.push(t3);

    let t4 = Transaction::build()
        .with_id(2)
        .with_timestamp(DateTime::parse_from_rfc3339("2017-10-04T10:01:00+01:00").unwrap())
        .with_group_id(0)
        .with_process_id(1)
        .with_text("Test2".to_owned())
        .try_finish(&transactions)
        .unwrap();
    println!("{:#?}", t4.to_string());
    transactions.push(t4);

}
