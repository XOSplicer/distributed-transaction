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

quick_error! {
    #[derive(Debug)]
    pub enum SyncError {
        Io(err: io::Error) {
            from()
        }
        Other(err: String) {
            from()
        }
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


#[derive(Debug)]
pub struct SyncroizedFileLog<P: AsRef<Path>> {
    path: P,
    last: Option<Transaction>,
    written: bool,
}

impl<P: AsRef<Path>>  SyncroizedFileLog<P> {

    pub fn new(path: P) -> Self {
        SyncroizedFileLog {
            path: path,
            last: None,
            written: false,
        }
    }

    pub fn syncronize(&mut self) -> Result<(), SyncError> {
        //let mut f = File::open(self.path.as_ref())?;
        let mut f = File::open(self.path.as_ref())?;
        let mut buffer = String::new();
        let file_size = f.metadata()?.len();
        let chunk_size = 10240;
        let start_pos = if file_size < chunk_size { 0 } else { file_size - chunk_size };
        f.seek(io::SeekFrom::Start(start_pos))?;
        f.take(chunk_size).read_to_string(&mut buffer)?;

        let mut lines = buffer.lines().rev();
        let last_line = lines.next();
        let second_last_line = lines.next();
        let last_tx = last_line.map(|line| Transaction::parse(line).unwrap()); //FIXME: use ?
        let second_last_tx = second_last_line.map(|line| Transaction::parse(line).unwrap()); //FIXME: use ?
        println!("last two entries: {:?}\n{:?}", &second_last_tx, &last_tx);
        println!("last tx in cache: {:?}", &self.last);
        match (&second_last_tx, &last_tx) {
            (&Some(ref tx1), &Some(ref tx2)) => {
                Transaction::verify(tx2, tx1)?;
            },
            (&None, &Some(ref tx2)) => {
                Transaction::verify_single(tx2)?;
            },
            (&None, &None) => {
                // no verify necessary
            },
            _ => panic!("Unexpected file behavior")
        }

        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.path.as_ref())?;
        f.seek(io::SeekFrom::End(0))?;
        if !self.written && self.last.is_some() {
            println!("writing last to file");
            f.write_all(self.last.as_ref().unwrap().to_string().as_bytes())?;
            self.written = true;
        } else {
            println!("no need to write, just load last");
            println!("loaded: {:?}", last_tx.as_ref().map(|t| t.to_string()));
            self.last = last_tx;
        }
        Ok(())
    }
}

impl<P: AsRef<Path>> TransactionLog for SyncroizedFileLog<P> {
    type AppendError = SyncError;

    fn last<'a>(&'a self) -> Option<&'a Transaction> {
       self.last.as_ref()
    }

    fn append(&mut self, tx: Transaction) -> Result<(), Self::AppendError> {
        //self.syncronize()?;
        self.last = Some(tx);
        self.written = false;
        self.syncronize()?;
        Ok(())
    }

}