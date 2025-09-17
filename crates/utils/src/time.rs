use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, FixedOffset, NaiveDateTime, prelude::*};

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

pub fn naive_from_unix(time: i64) -> NaiveDateTime {
    NaiveDateTime::from_timestamp_opt(time, 0).expect("convert datetime")
}

pub fn convert_datetime(datetime: NaiveDateTime) -> DateTime<FixedOffset> {
    DateTime::<FixedOffset>::from_utc(
        datetime,
        FixedOffset::east_opt(0).expect("create fixed offset"),
    )
}