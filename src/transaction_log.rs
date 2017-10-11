use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io;
use std::io::prelude::*;

use transaction::Transaction;

pub trait TransactionLog {
    type AppendError;

    fn last<'a>(&'a self) -> Option<&'a Transaction>;
    fn append(&mut self, tx: Transaction) -> Result<(), Self::AppendError>;
    fn next_id(&self) -> u32 {
        self.last().map(|t| t.next_id()).unwrap_or(
            Transaction::MIN_ID,
        )
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
    type AppendError = ();

    fn last<'a>(&'a self) -> Option<&'a Transaction> {
        self.last.as_ref()
    }

    fn append(&mut self, tx: Transaction) -> Result<(), Self::AppendError> {
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
    type AppendError = ();

    fn last<'a>(&'a self) -> Option<&'a Transaction> {
        self.log.last()
    }

    fn append(&mut self, tx: Transaction) -> Result<(), Self::AppendError> {
        self.log.push(tx);
        Ok(())
    }
}

#[derive(Debug)]
pub struct DirectFileLog {
    file: File,
    last: Option<Transaction>,
}

impl DirectFileLog {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Ok(DirectFileLog {
            file: OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(path)?,
            last: None
        })
    }
}

impl TransactionLog for DirectFileLog {
    type AppendError = io::Error;

    fn last<'a>(&'a self) -> Option<&'a Transaction> {
        self.last.as_ref()
    }

    fn append(&mut self, tx: Transaction) -> Result<(), Self::AppendError> {
        self.last = Some(tx);
        let string = self.last().unwrap().to_string();
        self.file.write_all(string.as_bytes())
    }
}