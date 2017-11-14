use std::str::FromStr;
use std::fmt;

use chrono;
use chrono::prelude::*;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub enum Error {
    IllegalArgument(String),
    ParseError(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyError {
    NonConsecutiveID(u32, u32),
    MissmatchingHash(u32),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransactionId(u32);

#[derive(Debug, Clone)]
pub struct TransactionTime(DateTime<chrono::FixedOffset>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionData {
    gid: u8,
    pid: u8,
    text: String,
}

#[derive(Debug, Clone)]
pub struct TransactionHash {
    vec: Vec<u8>,
    string: String,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    id: TransactionId,
    ts: TransactionTime,
    data: TransactionData,
    hash: TransactionHash,
}


impl TransactionId {
    pub const MIN_ID: u32 = 1;
    pub const MAX_ID: u32 = 99_999_999;

    pub fn new(id: u32) -> Result<Self, Error> {
        if id < Self::MIN_ID || id > Self::MAX_ID {
            return Err(Error::IllegalArgument(format!("Invalid id: {}", id)));
        }
        Ok(TransactionId(id))
    }

    pub fn inner(&self) -> u32 {
        self.0
    }

    pub fn next(&self) -> Self {
        if self.0 >= Self::MAX_ID {
            TransactionId(Self::MIN_ID)
        } else {
            TransactionId(self.0 + 1)
        }
    }
}

impl Default for TransactionId {
    fn default() -> Self {
        TransactionId(Self::MIN_ID)
    }
}

impl FromStr for TransactionId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: u32 = s.parse().map_err(|_| {
            Error::ParseError(format!("Could not parse id `{}` ", s.to_owned()))
        })?;
        Ok(Self::new(id)?)
    }
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:08}", self.0)
    }
}

impl TransactionTime {
    pub const TZ_OFFSET: i32 = 1 * 3600;
    pub const FORMAT: &'static str = "%d%m%y-%H:%M:%S";

    pub fn current() -> Self {
        TransactionTime(Utc::now().with_timezone(
            &FixedOffset::east(Self::TZ_OFFSET),
        ))
    }
}

impl FromStr for TransactionTime {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = chrono::FixedOffset::east(Self::TZ_OFFSET)
            .datetime_from_str(s, Self::FORMAT)
            .map_err(|_| {
                Error::ParseError(
                    format!("Could not parse time `{}` ", s.to_owned()),
                )
            })?;
        Ok(TransactionTime(t))
    }
}

impl fmt::Display for TransactionTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.format(Self::FORMAT))
    }
}


impl TransactionData {
    pub const MIN_GID: u8 = 0;
    pub const MAX_GID: u8 = 99;
    pub const MIN_PID: u8 = 0;
    pub const MAX_PID: u8 = 99;
    const INVALID_CHAR: &'static [&'static str] =
        &["\n", "\r", "\t", ";", "\0"];

    pub fn new<S: AsRef<str>>(
        gid: u8,
        pid: u8,
        text: S,
    ) -> Result<Self, Error> {
        let text = text.as_ref();

        if gid < Self::MIN_GID || gid > Self::MAX_GID {
            return Err(Error::IllegalArgument(format!("Invalid gid: {}", gid)));
        }
        if pid < Self::MIN_PID || pid > Self::MAX_PID {
            return Err(Error::IllegalArgument(format!("Invalid pid: {}", pid)));
        }
        if Self::INVALID_CHAR.iter().any(|c| text.contains(c)) {
            return Err(
                Error::IllegalArgument(format!("Invalid text: `{}`", text)),
            );
        }

        Ok(TransactionData {
            gid,
            pid,
            text: text.to_owned(),
        })

    }

    pub fn gid(&self) -> u8 {
        self.gid
    }

    pub fn pid(&self) -> u8 {
        self.pid
    }

    pub fn text<'a>(&'a self) -> &'a str {
        self.text.as_str()
    }
}

impl FromStr for TransactionData {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //println!("Parsing for data: {}", s);
        let mut parts = s.split(";");
        let gid: u8 =
            parts
                .next()
                .ok_or(Error::ParseError("Incomplete data".to_owned()))?
                .parse()
                .map_err(
                    |_| Error::ParseError("Could not parse gid".to_owned()),
                )?;
        let pid: u8 =
            parts
                .next()
                .ok_or(Error::ParseError("Incomplete data".to_owned()))?
                .parse()
                .map_err(
                    |_| Error::ParseError("Could not parse pid".to_owned()),
                )?;
        let text = parts.next().ok_or(
            Error::ParseError("Incomplete data".to_owned()),
        )?;
        if parts.next().is_some() {
            return Err(Error::ParseError("Too much data".to_owned()));
        }
        Ok(TransactionData::new(gid, pid, text)?)
    }
}

impl fmt::Display for TransactionData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02};{:02};{}", self.gid, self.pid, &self.text)
    }
}


impl TransactionHash {
    fn new(
        id: &TransactionId,
        ts: &TransactionTime,
        data: &TransactionData,
        prev: Option<&Transaction>,
    ) -> Self {
        let mut hasher = Sha256::default();
        let s = format!("{};{};{};", id, ts, data);
        //println!("Hashing {}", &s);
        hasher.input(s.as_bytes());
        if prev.is_some() {
            let p = format!("{}", prev.unwrap().hash());
            //println!("Hashing {}", &p);
            hasher.input(p.as_bytes());
        }
        let hash = Vec::from(hasher.result().as_slice());
        let hash_str = format!("{:X}", hasher.result());
        TransactionHash {
            vec: hash,
            string: hash_str,
        }

    }

    pub fn as_slice<'a>(&'a self) -> &'a [u8] {
        self.vec.as_slice()
    }
}

impl FromStr for TransactionHash {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut hash = Vec::with_capacity(32);
        let mut hash_chars = s.trim().chars();
        //FIXME: ugly way to parse hex to Vec<u8>
        //check for uppercase in hexstring
        if hash_chars.clone().any(
            |c| !(c.is_uppercase() || c.is_numeric()),
        )
        {
            return Err(
                Error::ParseError("Invalid hash character case".to_owned()),
            );
        }
        loop {
            match (hash_chars.next(), hash_chars.next()) {
                (Some(c1), Some(c2)) => {
                    hash.push(u8::from_str_radix(&format!("{}{}", c1, c2), 16)
                        .map_err(|_|
                            Error::ParseError("Invalid hash HEX".to_owned())
                        )?
                    )
                }
                (Some(_), None) => {
                    return Err(
                        Error::ParseError("Invalid hash length".to_owned()),
                    )
                }
                (None, _) => break,
            }
        }
        Ok(TransactionHash {
            vec: hash,
            string: s.to_owned(),
        })
    }
}

impl fmt::Display for TransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.string)
    }
}

impl Transaction {
    pub fn new(
        id: TransactionId,
        ts: TransactionTime,
        data: TransactionData,
        prev: Option<&Transaction>,
    ) -> Self {
        let hash = TransactionHash::new(&id, &ts, &data, prev);
        Transaction { id, ts, data, hash }
    }

    pub fn id(&self) -> &TransactionId {
        &self.id
    }

    pub fn ts(&self) -> &TransactionTime {
        &self.ts
    }

    pub fn data(&self) -> &TransactionData {
        &self.data
    }

    pub fn hash(&self) -> &TransactionHash {
        &self.hash
    }
}

impl FromStr for Transaction {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(";");
        let err = Error::ParseError("Incomplete data".to_owned());
        let id: TransactionId =
            parts.next().ok_or_else(|| err.clone())?.parse()?;
        let ts: TransactionTime =
            parts.next().ok_or_else(|| err.clone())?.parse()?;
        let data_gid = parts.next().ok_or_else(|| err.clone())?;
        let data_pid = parts.next().ok_or_else(|| err.clone())?;
        let data_text = parts.next().ok_or_else(|| err.clone())?;
        let data: TransactionData =
            format!("{};{};{}", data_gid, data_pid, data_text).parse()?;
        let hash: TransactionHash =
            parts.next().ok_or_else(|| err.clone())?.parse()?;
        if parts.next().is_some() {
            return Err(Error::ParseError("Too much data".to_owned()));
        }
        Ok(Transaction { id, ts, data, hash })
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{id};{ts};{data};{hash}",
            id = &self.id,
            ts = &self.ts,
            data = &self.data,
            hash = &self.hash
        )
    }
}


pub fn verify_transaction(
    tx: &Transaction,
    prev: Option<&Transaction>,
) -> Result<(), VerifyError> {
    if let Some(ref p) = prev {
        if &p.id().next() != tx.id() {
            return Err(VerifyError::NonConsecutiveID(
                p.id().inner(),
                tx.id().inner(),
            ));
        }
    }
    let hash = TransactionHash::new(tx.id(), tx.ts(), tx.data(), prev);
    if tx.hash().as_slice() != hash.as_slice() {
        return Err(VerifyError::MissmatchingHash(tx.id().inner()));
    }
    Ok(())
}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn example() {
        let tx = Transaction::new(
            TransactionId::new(1).unwrap(),
            "041017-10:00:00".parse::<TransactionTime>().unwrap(),
            TransactionData::new(0, 1, "Testü").unwrap(),
            None,
        );
        let expected = "00000001;041017-10:00:00;00;01;Testü;267C4D5033ED7F96B43216FD8C871E4B96F1221204312AD6F43362F2D12C9B29";
        assert_eq!(expected, tx.to_string());
    }

    #[test]
    fn parse1() {
        let input = "00000001;041017-10:00:00;00;01;Testü;267C4D5033ED7F96B43216FD8C871E4B96F1221204312AD6F43362F2D12C9B29\n";
        let parsed: Result<Transaction, _> = input.parse();
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().to_string(), input);
    }

    #[test]
    fn parse2() {
        let tx = Transaction::new(
            TransactionId::new(1).unwrap(),
            "041017-10:00:00".parse::<TransactionTime>().unwrap(),
            TransactionData::new(0, 1, "Testü").unwrap(),
            None,
        );
        let parsed: Result<Transaction, _> = tx.to_string().parse();
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().to_string(), tx.to_string());
    }

    #[test]
    fn verify_ok() {
        let tx1 = Transaction::new(
            TransactionId::new(1).unwrap(),
            "041017-10:00:00".parse::<TransactionTime>().unwrap(),
            TransactionData::new(0, 1, "Testü").unwrap(),
            None,
        );
        let tx2 = Transaction::new(
            TransactionId::new(2).unwrap(),
            "041017-10:00:00".parse::<TransactionTime>().unwrap(),
            TransactionData::new(0, 1, "Großes ß").unwrap(),
            Some(&tx1),
        );
        assert_eq!(verify_transaction(&tx2, Some(&tx1)), Ok(()));
    }

    #[test]
    fn verify_err() {
        let tx1 = Transaction::new(
            TransactionId::new(1).unwrap(),
            "041017-10:00:00".parse::<TransactionTime>().unwrap(),
            TransactionData::new(0, 1, "Testü").unwrap(),
            None,
        );
        let tx2 = Transaction::new(
            TransactionId::new(1).unwrap(),
            "041017-10:00:00".parse::<TransactionTime>().unwrap(),
            TransactionData::new(0, 1, "Großes S").unwrap(),
            None,
        );
        assert!(verify_transaction(&tx2, Some(&tx1)).is_err());
    }



}
