use std::collections::BTreeMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::path::Path;

use transaction::*;


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


quick_error! {
    #[derive(Debug)]
    pub enum FileError {
        Io(err: io::Error) {
            from()
        }
        Transaction(err: Error) {
            from()
        }
        Verify(err: VerifyError) {
            from()
        }
        Other(err: String) {
            from()
        }
        No(err: ()) {
            from()
        }
    }
}

#[derive(Debug)]
pub struct SimpleFileLog<P: AsRef<Path>> {
    path: P,
}

impl<P: AsRef<Path>> SimpleFileLog<P> {
    pub fn new(path: P) -> Self {
        SimpleFileLog {
            path
        }
    }
}

impl<P: AsRef<Path>> TransactionLog for SimpleFileLog<P>{
    type Error = FileError;

    fn create(
        &mut self,
        data: TransactionData,
        time: Option<TransactionTime>,
    ) -> Result<Transaction, Self::Error> {
        let tx = Transaction::new(
            self.next_id()?,
            time.unwrap_or_else(|| TransactionTime::current()),
            data,
            self.last()?.as_ref(),
        );
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.path.as_ref())?;
        f.seek(io::SeekFrom::End(0))?;
        f.write_all(format!("{}\n", tx).as_bytes())?;
        Ok(tx)
    }

    fn last(&self) -> Result<Option<Transaction>, Self::Error> {
        let mut f = File::open(self.path.as_ref())?;
        let mut buffer = String::new();
        let file_size = f.metadata()?.len();
        let chunk_size = 10_240;
        let start_pos = if file_size < chunk_size { 0 } else { file_size - chunk_size };
        f.seek(io::SeekFrom::Start(start_pos))?;
        f.take(chunk_size).read_to_string(&mut buffer)?;

        //FIXME: use proper error handling to make mapping:
        // None -> None
        // Some(Ok(t)) -> Some(t)
        // Some(Err(e)) -> early return Err(e)
        let mut lines = buffer.lines().rev();
        let last_tx = lines.next()
            .and_then(|line| line.parse::<Transaction>().ok());
        let last2_tx = lines.next()
            .and_then(|line| line.parse::<Transaction>().ok());
        match (&last2_tx, &last_tx) {
            (&Some(ref tx1), &Some(ref tx2)) => {
                verify_transaction(tx2, Some(tx1))?;
            },
            (&None, &Some(ref tx2)) => {
                verify_transaction(tx2, None)?;
            },
            (&None, &None) => {
                // no verify necessary
            },
            _ => panic!("Unexpected file behavior, last tx does not exist, but second last")
        }
        Ok(last_tx)
    }


}

impl<P: AsRef<Path>> GetAll for SimpleFileLog<P> {
    type Error = FileError;
    fn get_all(&self) -> Result<Vec<Transaction>, Self::Error> {
        let f = File::open(self.path.as_ref())?;
        let mut lines = io::BufReader::new(f).lines();
        let mut vec = Vec::with_capacity(lines.size_hint().0);
        {
            let mut last_tx = None;
            for l in lines {
                let line = l?;
                let tx: Transaction = line.parse()?;
                verify_transaction(&tx, last_tx.as_ref())?;
                vec.push(tx.clone());
                last_tx = Some(tx);
            }
        }
        Ok(vec)
    }
}

#[derive(Debug)]
pub struct DualLog<P: AsRef<Path>> {
    full_log: FullTransactionLog,
    file_log: SimpleFileLog<P>,
}

impl<P: AsRef<Path>> DualLog<P> {
    pub fn load(path: P) -> Result<Self, FileError> {
        let file_log = SimpleFileLog::new(path);
        let all = file_log.get_all()?;
        let map: BTreeMap<u32, Transaction> = all
            .into_iter()
            .map(|t| (t.id().inner(), t))
            .collect();
        let full_log = FullTransactionLog {
            log: map
        };
        Ok(DualLog {
            full_log,
            file_log
        })
    }
}

impl<P: AsRef<Path>> TransactionLog for DualLog<P> {
    type Error = FileError;

    fn create(
        &mut self,
        data: TransactionData,
        time: Option<TransactionTime>,
    ) -> Result<Transaction, Self::Error> {
        let tx = self.file_log.create(data, time)?;
        self.full_log.log.insert(tx.id().inner(), tx.clone());
        Ok(tx)
    }

    fn last(&self) -> Result<Option<Transaction>, Self::Error> {
        Ok(self.full_log.last()?)
    }
}

impl<P: AsRef<Path>> GetById for DualLog<P> {
    type Error = FileError;
    fn get_by_id(&self, id: u32) -> Result<Option<Transaction>, Self::Error> {
        Ok(self.full_log.get_by_id(id)?)
    }
}

impl<P: AsRef<Path>> GetAll for DualLog<P> {
    type Error = FileError;
    fn get_all(&self) -> Result<Vec<Transaction>, Self::Error> {
        Ok(self.full_log.get_all()?)
    }
}
