use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, FixedOffset, NaiveDateTime, prelude::*};

pub fn time() -> u64 {
    // SystemTime::now() is always >= UNIX_EPOCH on any sane system
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before UNIX epoch (1970) -- clock misconfigured")
        .as_secs()
}

/// Returns a UNIX timestamp of the current time in UTC (i64)
pub fn utc_timestamp() -> i64 {
    Utc::now().timestamp()
}

pub fn naive_from_unix(time: i64) -> Option<NaiveDateTime> {
    DateTime::from_timestamp(time, 0).map(|dt| dt.naive_utc())
}

pub fn convert_datetime(datetime: NaiveDateTime) -> DateTime<FixedOffset> {
    // UTC offset of 0 is always valid
    FixedOffset::east_opt(0)
        .expect("UTC offset 0 is always valid")
        .from_utc_datetime(&datetime)
}
