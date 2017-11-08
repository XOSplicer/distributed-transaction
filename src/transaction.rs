use chrono;
use chrono::prelude::*;
use itertools::Itertools;
use sha2::{Digest, Sha256};

use transaction_log::TransactionLog;

#[derive(Debug, Clone)]
pub struct Transaction {
    id: u32,
    timestamp: DateTime<chrono::FixedOffset>,
    group_id: u8,
    process_id: u8,
    text: String,
    hash: Vec<u8>,
    hash_str: String,
}

#[derive(Debug, Clone)]
pub struct TransactionBuilder {
    id: Option<u32>,
    timestamp: Option<DateTime<chrono::FixedOffset>>,
    group_id: Option<u8>,
    process_id: Option<u8>,
    text: Option<String>,
}

impl Transaction {
    pub const MIN_ID: u32 = 1;
    pub const MAX_ID: u32 = 99_999_999;
    pub const MIN_GID: u8 = 0;
    pub const MAX_GID: u8 = 99;
    pub const MIN_PID: u8 = 0;
    pub const MAX_PID: u8 = 99;
    const INVALID_CHAR: &'static [&'static str] = &["\n", "\r", "\t", ";", "\0"];

    pub const TZ_OFFSET: i32 = 2 * 3600;

    pub fn build() -> TransactionBuilder {
        TransactionBuilder::new()
    }

    fn check_input(id: u32, gid: u8, pid: u8, text: &str) -> Result<(), String> {
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
        Ok(())
    }

    fn with_log<L: TransactionLog>(
        log: &L,
        id: u32,
        time: DateTime<chrono::FixedOffset>,
        gid: u8,
        pid: u8,
        text: String,
    ) -> Result<Transaction, String> {
        Transaction::check_input(id, gid, pid, &text)?;

        // Hash all previous transaction strings
        // and the partial new transaction
        let mut hasher = Sha256::default();
        hasher.input(
            Self::partial_string(id, &time, gid, pid, &text)
                .into_bytes()
                .as_ref(),
        );
        let prev = log.last().map_err(|_| "Could not get last log entry".to_owned())?;
        if prev.is_some() {
            hasher.input(prev.unwrap().hash_str().as_bytes());
        }

        let hash = Vec::from(hasher.result().as_slice());
        let hash_str = Transaction::hash_string(&hash);

        Ok(Transaction {
            id: id,
            timestamp: time,
            group_id: gid,
            process_id: pid,
            text: text,
            hash: hash,
            hash_str: hash_str,
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
        Transaction::check_input(id, gid, pid, &text)?;
        let hash_str = Transaction::hash_string(&hash);
        Ok(Transaction {
            id: id,
            timestamp: time,
            group_id: gid,
            process_id: pid,
            text: text,
            hash: hash,
            hash_str: hash_str,
        })
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

    fn hash_string(hash: &Vec<u8>) -> String {
        format!("{:02X}", hash.iter().format(""))
    }

    pub fn to_string(&self) -> String {
        format!(
            "{data}{chksm}",
            data = &self.to_data_string(),
            chksm = &self.hash_str,
        )
    }

    pub fn to_data_string(&self) -> String {
        format!("{partial}",
            partial = Self::partial_string(
                self.id,
                &self.timestamp,
                self.group_id,
                self.process_id,
                &self.text
            ),
        )
    }

    pub fn parse(src: &str) -> Result<Self, String> {
        let mut parts = src.split(";");
        let mut builder = Transaction::build();

        builder = builder.with_id(parts
            .next()
            .ok_or("Incomplete source, no id".to_owned())?
            .parse()
            .map_err(|_| "Could not parse id".to_owned())?);

        builder = builder.with_timestamp(
            //timezone
            chrono::FixedOffset::east(Transaction::TZ_OFFSET)
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

        let mut hash = Vec::with_capacity(32);
        let mut hash_str = parts
            .next()
            .ok_or("Incomplete source, no hash".to_owned())?
            .trim()
            .chars();
        //FIXME: ugly way to parse hex to Vec<u8>
        loop {
            match (hash_str.next(), hash_str.next()) {
                (Some(c1), Some(c2)) => {
                    hash.push(u8::from_str_radix(&format!("{}{}", c1, c2), 16)
                        .map_err(|_| "could not parse hash".to_owned())?);
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

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn ts<'a>(&'a self) -> &'a DateTime<chrono::FixedOffset> {
        &self.timestamp
    }

    pub fn gid(&self) -> u8 {
        self.group_id
    }

    pub fn pid(&self) -> u8 {
        self.process_id
    }

    pub fn text<'a>(&'a self) -> &'a str {
        self.text.as_str()
    }

    pub fn hash<'a>(&'a self) -> &'a [u8] {
        self.hash.as_slice()
    }

    pub fn hash_str<'a>(&'a self) -> &'a str {
        self.hash_str.as_str()
    }

    pub fn next_id(&self) -> u32 {
        if self.id >= Transaction::MAX_ID {
            Transaction::MIN_ID
        } else {
            self.id + 1
        }
    }

    pub fn verify(curr: &Transaction, prev: &Transaction) -> Result<(), String> {

        if curr.id() != prev.next_id() {
            return Err("Non-consecutive IDs".to_owned());
        }

        let mut hasher = Sha256::default();
        hasher.input(curr.to_data_string().into_bytes().as_ref());
        hasher.input(prev.hash_str().as_bytes());

        let expected_hash = hasher.result();
        let curr_hash = curr.hash();

        if expected_hash.as_slice() != curr_hash {
            return Err("Missmatched hashes".to_owned());
        }
        Ok(())
    }

    pub fn verify_single(tx: &Transaction) -> Result<(), String> {
        let mut hasher = Sha256::default();
        hasher.input(tx.to_data_string().into_bytes().as_ref());
        let expected_hash = hasher.result();
        let curr_hash = tx.hash();
        if expected_hash.as_slice() != curr_hash {
            return Err("Missmatched hashes".to_owned());
        }
        Ok(())
    }

}


impl TransactionBuilder {
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

    pub fn with_current_timestamp(self) -> Self {
        self.with_timestamp(Utc::now().with_timezone(&FixedOffset::east(Transaction::TZ_OFFSET)))
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

    pub fn try_finish_with_log<L: TransactionLog>(self, log: &L) -> Result<Transaction, String> {
        Transaction::with_log(
            log,
            self.id.ok_or("No id given".to_string())?,
            self.timestamp.ok_or("No timestamp given".to_string())?,
            self.group_id.ok_or("No group id given".to_string())?,
            self.process_id.ok_or("No process id given".to_string())?,
            self.text.ok_or("No text given".to_string())?,
        )
    }

    fn try_finish_with_hash(self, hash: Vec<u8>) -> Result<Transaction, String> {
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


#[cfg(test)]
mod test {

    use super::*;
    use super::super::*;
    use chrono::prelude::*;

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

    #[test]
    fn verify1() {
        let mut log = FullTransactionLog::new();
        let tx1 = Transaction::build()
            .with_id(1)
            .with_timestamp(
                DateTime::parse_from_rfc3339("2017-10-04T11:05:00+01:00")
                .unwrap()
                .with_timezone(&FixedOffset::east(Transaction::TZ_OFFSET)),
            )
            .with_group_id(1)
            .with_process_id(1)
            .with_text("test1".to_owned())
            .try_finish_with_log(&log)
            .unwrap();
        log.append(tx1.clone());
        let tx2 = Transaction::build()
            .with_id(2)
            .with_timestamp(
                DateTime::parse_from_rfc3339("2017-10-04T11:05:00+01:00")
                .unwrap()
                .with_timezone(&FixedOffset::east(Transaction::TZ_OFFSET)),
            )
            .with_group_id(2)
            .with_process_id(2)
            .with_text("test2".to_owned())
            .try_finish_with_log(&log)
            .unwrap();
        assert_eq!(Transaction::verify(&tx2, &tx1), Ok(()));
    }


    #[test]
    fn verify2() {
        let mut log = FullTransactionLog::new();
        let tx1 = Transaction::build()
            .with_id(1)
            .with_timestamp(
                DateTime::parse_from_rfc3339("2017-10-04T11:05:00+01:00")
                .unwrap()
                .with_timezone(&FixedOffset::east(Transaction::TZ_OFFSET)),
            )
            .with_group_id(1)
            .with_process_id(1)
            .with_text("test1".to_owned())
            .try_finish_with_log(&log)
            .unwrap();
        log.append(tx1.clone());
        let tx2_input = "00000002;041017-10:00:00;00;01;Testü;267C4D5033ED7F96B43216FD8C871E4B96F1221204312AD6F43362F2D12C9B29\n";
        let tx2 = Transaction::parse(tx2_input).unwrap();
        assert!(Transaction::verify(&tx2, &tx1).is_err());
    }

}
