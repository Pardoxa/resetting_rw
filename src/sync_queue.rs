use std::{collections::VecDeque, sync::Mutex};



struct SyncQueue<T>{
    queue: Mutex<VecDeque<T>>
}

impl<T> SyncQueue<T>
where T: Sync
{
    pub fn new(queue: VecDeque<T>) -> Self
    {
        Self { queue: Mutex::new(queue) }
    }

    pub fn pop(&self) -> Option<T>
    {
        let mut lock = self.queue
            .lock()
            .unwrap();
        let item = lock.pop_front();
        drop(lock);
        item
    }

    pub fn push(&self, item: T)
    {
        let mut lock = self.queue
            .lock()
            .unwrap();
        lock.push_back(item);
        drop(lock);
    }
}