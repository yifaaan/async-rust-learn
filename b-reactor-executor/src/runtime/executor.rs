use crate::future::{Future, PollState};

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, Thread},
};

#[derive(Clone)]
pub struct Waker {
    /// executor thread
    thread: Thread,
    /// which task this Waker is associated with
    id: usize,
    /// shared with Executor
    ready_queue: Arc<Mutex<Vec<usize>>>,
}

impl Waker {
    pub fn wake(&self) {
        // 将与该waker关联的任务task id放入就绪队列
        self.ready_queue
            .lock()
            .map(|mut q| q.push(self.id))
            .unwrap();
        // 唤醒executor thread
        self.thread.unpark();
    }
}
