extern crate chrono;
extern crate itertools;
extern crate sha2;

use itertools::Itertools;
use chrono::prelude::*;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Transaction<Tz: TimeZone> {
    id: u32,
    timestamp: DateTime<Tz>,
    group_id: u8,
    process_id: u8,
    text: String,
    checksum: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    const INVALID_CHAR: &'static [&'static str] = &["\n", "\r", "\t", ";", "\0"];

    pub fn build() -> TransactionBuilder<chrono::FixedOffset> {
        TransactionBuilder::new()
    }

    fn with_prev(
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

    fn with_hash(
        hash: Vec<u8>,
        id: u32,
        time: DateTime<chrono::FixedOffset>,
        gid: u8,
        pid: u8,
        text: String,
    ) -> Result<Self, String> {
        //FIXME: Calculating the hash in with_prev from is unnecessary
        let mut t = Self::with_prev(&vec![], id, time, gid, pid, text)?;
        t.checksum = hash;
        Ok(t)
    }

    // generate the unfinished string representation for a transaction
    // which does not include the hash, so it can be hashed
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


    pub fn try_from_str(src: &str) -> Result<Self, String> {
        let mut parts = src.split(";");
        let mut builder = Transaction::build();

        builder = builder.with_id(parts
            .next()
            .ok_or("Incomplete source, no id".to_owned())?
            .parse()
            .map_err(|_| "Could not parse id".to_owned())?);

        builder = builder.with_timestamp(
            //timezone
            chrono::FixedOffset::east(1 * 3600)
                .datetime_from_str(
                    parts.next().ok_or("Incomplete source, no id".to_owned())?,
                    "%d%m%y-%H:%M:%S",
                )
                .map_err(|_| "Could not parse timstamp".to_owned())?,
        );

        builder = builder.with_group_id(parts
            .next()
            .ok_or("Incomplete source, no gid".to_owned())?
            .parse()
            .map_err(|_| "Could not parse gid".to_owned())?);

        builder = builder.with_process_id(parts
            .next()
            .ok_or("Incomplete source, no pid".to_owned())?
            .parse()
            .map_err(|_| "Could not parse pid".to_owned())?);

        builder = builder.with_text(
            parts
                .next()
                .ok_or("Incomplete source, no text".to_owned())?
                .to_owned(),
        );

        let mut hash = Vec::new();
        let mut hash_str = parts
            .next()
            .ok_or("Incomplete source, no hash".to_owned())?
            .trim()
            .chars();
        //FIXME: ugly way to parse hex to Vec<u8>
        loop {
            match (hash_str.next(), hash_str.next()) {
                (Some(c1), Some(c2)) => {
                    hash.push(u8::from_str_radix(&format!("{}{}", c1, c2), 16).map_err(
                        |_| {
                            "could not parse hash".to_owned()
                        },
                    )?);
                }
                (Some(_), None) => return Err("Invalid hash length".to_owned()),
                (None, _) => break,

            }
        }
        let t = builder.try_finish_with_hash(hash)?;

        match parts.next() {
            Some(_) => Err("Invalid source, too many parts".to_owned()),
            None => Ok(t),
        }
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

    pub fn try_finish_with_prev(
        self,
        prev: &Vec<Transaction<chrono::FixedOffset>>,
    ) -> Result<Transaction<chrono::FixedOffset>, String> {
        Transaction::with_prev(
            prev,
            self.id.ok_or("No id given".to_string())?,
            self.timestamp.ok_or("No timestamp given".to_string())?,
            self.group_id.ok_or("No group id given".to_string())?,
            self.process_id.ok_or("No process id given".to_string())?,
            self.text.ok_or("No text given".to_string())?,
        )
    }

    pub fn try_finish_with_hash(
        self,
        hash: Vec<u8>,
    ) -> Result<Transaction<chrono::FixedOffset>, String> {
        Transaction::with_hash(
            hash,
            self.id.ok_or("No id given".to_string())?,
            self.timestamp.ok_or("No timestamp given".to_string())?,
            self.group_id.ok_or("No group id given".to_string())?,
            self.process_id.ok_or("No process id given".to_string())?,
            self.text.ok_or("No text given".to_string())?,
        )
    }
}


fn main() {
    let mut transactions = Vec::new();

    let t3 = Transaction::build()
        .with_id(1)
        .with_timestamp(
            DateTime::parse_from_rfc3339("2017-10-04T10:00:00+01:00").unwrap(),
        )
        .with_group_id(0)
        .with_process_id(1)
        .with_text("Testü".to_owned())
        .try_finish_with_prev(&transactions)
        .unwrap();
    println!("{:#?}", t3.to_string());
    transactions.push(t3);

    let t4 = Transaction::build()
        .with_id(2)
        .with_timestamp(
            DateTime::parse_from_rfc3339("2017-10-04T10:01:00+01:00").unwrap(),
        )
        .with_group_id(0)
        .with_process_id(1)
        .with_text("Test2".to_owned())
        .try_finish_with_prev(&transactions)
        .unwrap();
    println!("{:#?}", t4.to_string());
    transactions.push(t4);

}

mod test {

    use super::*;
    use chrono::prelude::*;

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

    #[test]
    fn parse1() {
        let input = "00000001;041017-10:00:00;00;01;Testü;267C4D5033ED7F96B43216FD8C871E4B96F1221204312AD6F43362F2D12C9B29\n";
        let parsed = Transaction::try_from_str(input);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().to_string(), input);
    }

    #[test]
    fn parse2() {
        let t = Transaction::build()
            .with_id(42)
            .with_timestamp(
                DateTime::parse_from_rfc3339("2017-10-04T11:05:00+01:00").unwrap(),
            )
            .with_group_id(43)
            .with_process_id(44)
            .with_text("Großes S".to_owned())
            .try_finish_with_prev(&vec![])
            .unwrap();
        let parsed = Transaction::try_from_str(&t.to_string());
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), t);
    }


}
