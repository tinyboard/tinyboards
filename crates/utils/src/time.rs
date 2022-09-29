use std::time::{SystemTime, UNIX_EPOCH};
use chrono::prelude::*;

pub fn time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}


/**
 *  Returns a UNIX Timestamp of the current time in the UTC TimeZone (i64)
 */
pub fn utc_timestamp() -> i64 {
    Utc::now().timestamp()
}