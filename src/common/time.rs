use std::time::{SystemTime, UNIX_EPOCH};

pub type Timestamp = u64;

pub struct Time;

impl Time {
    pub fn get_current_timestamp() -> Timestamp {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}
