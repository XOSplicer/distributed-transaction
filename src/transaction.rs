use chrono;
use chrono::prelude::*;
use itertools::Itertools;
use sha2::{Digest, Sha256};

use transaction_log::TransactionLog;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transaction {
    id: u32,
    timestamp: DateTime<chrono::FixedOffset>,
    group_id: u8,
    process_id: u8,
    text: String,
    hash: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
        let prev = log.last();
        if prev.is_some() {
            hasher.input(prev.unwrap().hash());
        }

        let hash = Vec::from(hasher.result().as_slice());

        Ok(Transaction {
            id: id,
            timestamp: time,
            group_id: gid,
            process_id: pid,
            text: text,
            hash: hash,
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
        Ok(Transaction {
            id: id,
            timestamp: time,
            group_id: gid,
            process_id: pid,
            text: text,
            hash: hash,
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

    fn hash_string(&self) -> String {
        format!("{:02X}", self.hash.iter().format(""))
    }

    pub fn to_string(&self) -> String {
        format!(
            "{partial}{chksm}\n",
            partial = Self::partial_string(
                self.id,
                &self.timestamp,
                self.group_id,
                self.process_id,
                &self.text
            ),
            chksm = self.hash_string(),
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

    pub fn next_id(&self) -> u32 {
        if self.id >= Transaction::MAX_ID {
            Transaction::MIN_ID
        } else {
            self.id + 1
        }
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

    pub fn try_finish_with_hash(self, hash: Vec<u8>) -> Result<Transaction, String> {
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
