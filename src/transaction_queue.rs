use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver}
use std::thread::JoinHandle;
use std::collections::VecDeque;

use transaction::{Transaction, TransactionBuilder};
use transaction_log::{TransactionLog, SyncroizedFileLog}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueuedTransaction {
    pub text: String,
    pub gid: u8,
    pub pid: u8,
}

#[derive(Debug)]
pub enum QueueMessage {
    Queue(QueuedTransaction),
    Flush,
    Finish
}


#[derive(Debug)]
pub struct TransactionQueue {
    channel_send: Sender,
    thread: JoinHandle<()>,
}

impl TransactionQueue {
    const QUEUE_SIZE: usize = 64;

    pub fn new(log: SyncroizedFileLog) -> Self {
        let (sender, receiver) = mpsc::channel<QueueMessage>();
        QueuedTransaction {
            channel_send: sender,
            thread: thread::spawn(move || {
                let mut queue = VecDeque::with_capacity::<QueuedTransaction>();
                loop {
                    let msg = receiver.recv().unrwap();
                    match msg {
                        Queue(qtx) => {
                            queue.push_back(qtx);
                        },
                        Flush => {
                            TransactionQueue::flush(&mut log, &mut queue);
                        },
                        Finish => {
                            TransactionQueue::flush(&mut log, &mut queue);
                            break;
                        },
                    }
                }
            }),
        }
    }

    pub fn channel_sender(&self) -> Sender<QueueMessage> {
        self.sender.clone()
    }

    fn flush(log: &mut SyncroizedFileLog, q: &mut VecDeque<QueuedTransaction>) {
        log.syncronize();
        for QueuedTransaction{text: text, gid: gid, pid: pid} in queue.drain(..) {
            let id = log.next_id();
            let tx = Transaction::build()
                    .with_id(id)
                    .with_current_timestamp()
                    .with_group_id(gid)
                    .with_process_id(pid)
                    .with_text(text)
                    .try_finish_with_log(log)
                    .unwrap();
            log.append(tx);
            log.syncronize();
        }
    }

}