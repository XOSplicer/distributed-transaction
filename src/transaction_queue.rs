
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::collections::VecDeque;
use std::thread;

use transaction::Transaction;
use transaction_log::TransactionLog;

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

use self::QueueMessage::*;

#[derive(Debug)]
pub struct TransactionQueue {
    channel_send: Sender<QueueMessage>,
    thread: JoinHandle<()>,
}

impl TransactionQueue {
    const QUEUE_SIZE: usize = 64;

    pub fn new<L: TransactionLog + Send + 'static>(mut log: L) -> Self {
        let (sender, receiver) = mpsc::channel::<QueueMessage>();
        TransactionQueue {
            channel_send: sender,
            thread: thread::Builder::new()
                .name("transaction_queue".to_owned())
                .spawn(move || {
                    let mut queue = VecDeque::with_capacity(TransactionQueue::QUEUE_SIZE);
                    loop {
                        let msg = receiver.recv().unwrap();
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
                }).unwrap(),
        }
    }

    pub fn sender(&self) -> Sender<QueueMessage> {
        self.channel_send.clone()
    }

    pub fn join(self) -> thread::Result<()> {
        self.thread.join()
    }

    fn flush<L: TransactionLog>(log: &mut L, q: &mut VecDeque<QueuedTransaction>) {
        for QueuedTransaction{ text, gid, pid } in q.drain(..) {
            let id = log.next_id().unwrap();
            let tx = Transaction::build()
                    .with_id(id)
                    .with_current_timestamp()
                    .with_group_id(gid)
                    .with_process_id(pid)
                    .with_text(text)
                    .try_finish_with_log(log)
                    .unwrap();
            log.append(tx).unwrap();
        }
    }

}
