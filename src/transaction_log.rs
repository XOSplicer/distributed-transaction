use std::iter::IntoIterator;

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