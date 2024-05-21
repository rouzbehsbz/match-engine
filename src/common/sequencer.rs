use std::sync::atomic::{AtomicU64, Ordering};

pub struct Sequencer {
    index: AtomicU64,
}

impl Sequencer {
    pub fn new() -> Self {
        Self {
            index: AtomicU64::new(0),
        }
    }

    pub fn get(&self) -> u64 {
        self.index.load(Ordering::Relaxed)
    }

    pub fn next(&self) -> u64 {
        self.index.fetch_add(1, Ordering::Relaxed);
        self.index.load(Ordering::Relaxed)
    }
}
