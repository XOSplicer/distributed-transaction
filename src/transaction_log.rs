//use std::fs::{File, OpenOptions};
//use std::path::Path;
//use std::io;
//use std::io::prelude::*;

use std::fmt;
use std::collections::BTreeMap;

use transaction::{Transaction, TransactionId, TransactionTime, TransactionData};


pub trait TransactionLog {
    type Error: fmt::Debug;
    fn create(
        &mut self,
        data: TransactionData,
        time: Option<TransactionTime>,
    ) -> Result<Transaction, Self::Error>;

    fn last(&self) -> Result<Option<Transaction>, Self::Error>;

    fn next_id(&self) -> Result<TransactionId, Self::Error> {
        Ok(self.last()?.map(|t| t.id().next()).unwrap_or_default())
    }
}

pub trait GetById {
    type Error: fmt::Debug;
    fn get_by_id(&self, id: u32) -> Result<Option<Transaction>, Self::Error>;
}

pub trait GetAll {
    type Error: fmt::Debug;
    fn get_all(&self) -> Result<Vec<Transaction>, Self::Error>;
}


#[derive(Debug)]
pub struct FullTransactionLog {
    log: BTreeMap<u32, Transaction>,
}

impl FullTransactionLog {
    pub fn new() -> Self {
        FullTransactionLog { log: BTreeMap::new() }
    }
}

impl TransactionLog for FullTransactionLog {
    type Error = ();

    fn create(
        &mut self,
        data: TransactionData,
        time: Option<TransactionTime>,
    ) -> Result<Transaction, Self::Error> {
        let next_id = self.next_id()?;
        let last = self.last()?;
        let tx = Transaction::new(
            next_id,
            time.unwrap_or_else(|| TransactionTime::current()),
            data,
            last.as_ref(),
        );
        let c = tx.clone();
        self.log.insert(tx.id().inner(), tx);
        Ok(c)
    }

    fn last(&self) -> Result<Option<Transaction>, Self::Error> {
        Ok(self.log.iter().next_back().map(|(_, t)| t).cloned())
    }
}

impl GetById for FullTransactionLog {
    type Error = ();
    fn get_by_id(&self, id: u32) -> Result<Option<Transaction>, Self::Error> {
        Ok(self.log.get(&id).cloned())
    }
}

impl GetAll for FullTransactionLog {
    type Error = ();
    fn get_all(&self) -> Result<Vec<Transaction>, Self::Error> {
        Ok(self.log.values().cloned().collect())
    }
}



/*

pub trait TransactionLog {
    type Error: fmt::Debug;

    fn last(&self) -> Result<Option<Transaction>, Self::Error>;
    fn append(&mut self, tx: Transaction) -> Result<(), Self::Error>;
    fn next_id(&self) -> Result<u32, Self::Error> {
        Ok(self.last()?.map(|t| t.next_id()).unwrap_or(
            Transaction::MIN_ID,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct SingleTransactionLog {
    last: Option<Transaction>,
}

impl SingleTransactionLog {
    pub fn new() -> Self {
        SingleTransactionLog { last: None }
    }
}

impl TransactionLog for SingleTransactionLog {
    type Error = ();

    fn last(&self) -> Result<Option<Transaction>, Self::Error> {
        Ok(self.last.clone())
    }

    fn append(&mut self, tx: Transaction) -> Result<(), Self::Error> {
        self.last = Some(tx);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FullTransactionLog {
    log: Vec<Transaction>,
}

impl FullTransactionLog {
    pub fn new() -> Self {
        FullTransactionLog { log: Vec::new() }
    }

    pub fn with_capactity(c: usize) -> Self {
        FullTransactionLog { log: Vec::with_capacity(c)}
    }
}

impl TransactionLog for FullTransactionLog {
    type Error = ();

    fn last(&self) -> Result<Option<Transaction>, Self::Error> {
        Ok(self.log.last().cloned())
    }

    fn append(&mut self, tx: Transaction) -> Result<(), Self::Error> {
        self.log.push(tx);
        Ok(())
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum FileError {
        Io(err: io::Error) {
            from()
        }
        Other(err: String) {
            from()
        }
    }
}

#[derive(Debug)]
pub struct SimpleFileLog<P: AsRef<Path>> {
    path: P,
}


impl<P: AsRef<Path>> TransactionLog for SimpleFileLog<P>{
    type Error = FileError;

    fn last(&self) -> Result<Option<Transaction>, Self::Error> {
        let mut f = File::open(self.path.as_ref())?;
        let mut buffer = String::new();
        let file_size = f.metadata()?.len();
        let chunk_size = 10_240;
        let start_pos = if file_size < chunk_size { 0 } else { file_size - chunk_size };
        f.seek(io::SeekFrom::Start(start_pos))?;
        f.take(chunk_size).read_to_string(&mut buffer)?;

        let mut lines = buffer.lines().rev();
        let last_tx = lines.next()
            .and_then(|line| Transaction::parse(line).ok());
        let last2_tx = lines.next()
            .and_then(|line| Transaction::parse(line).ok());
        match (&last2_tx, &last_tx) {
            (&Some(ref tx1), &Some(ref tx2)) => {
                Transaction::verify(tx2, tx1)?;
            },
            (&None, &Some(ref tx2)) => {
                Transaction::verify_single(tx2)?;
            },
            (&None, &None) => {
                // no verify necessary
            },
            _ => Err("Unexpected file behavior".to_owned())?
        }
        Ok(last_tx)
    }

    fn append(&mut self, tx: Transaction) -> Result<(), Self::Error> {
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.path.as_ref())?;
        f.seek(io::SeekFrom::End(0))?;
        f.write_all(tx.to_string().as_bytes())?;
        Ok(())
    }
}

impl<P: AsRef<Path>> SimpleFileLog<P> {
    pub fn new(path: P) -> Self {
        SimpleFileLog {
            path: path,
        }
    }
}

pub trait AllTransactions {
    type Error: fmt::Debug;
    fn all_transactions(&self) -> Result<Vec<Transaction>, Self::Error>;
}

impl AllTransactions for FullTransactionLog {
    type Error = ();
    fn all_transactions(&self) -> Result<Vec<Transaction>, Self::Error> {
        Ok(self.log.clone())
    }
}

*/