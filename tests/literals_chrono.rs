#![cfg(feature = "chrono")]

mod common;

use std::str::FromStr;

use chrono::prelude::*;
use common::{test_literal, test_literal_error};
use mssql_value_serializer::SqlLiteralError;

#[test]
fn test_chrono() {
    test_literal("'2025-01-02'", NaiveDate::from_ymd_opt(2025, 1, 2).unwrap());
    test_literal("'03:04:05.0000678'", NaiveTime::from_hms_nano_opt(3, 4, 5, 67890).unwrap());
    test_literal("'03:04:05.06789'", NaiveTime::from_hms_nano_opt(3, 4, 5, 67890000).unwrap());
    test_literal(
        "'2025-01-02 03:04:05.06789'",
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            NaiveTime::from_hms_nano_opt(3, 4, 5, 67890000).unwrap(),
        ),
    );
    test_literal(
        "'2025-01-02 03:04:05.06789 -01:02'",
        DateTime::<FixedOffset>::from_str("2025-01-02T03:04:05.06789-01:02").unwrap(),
    );
    test_literal(
        "'2025-01-02 03:04:05.06789 +00:00'",
        DateTime::<Utc>::from_str("2025-01-02T03:04:05.06789Z").unwrap(),
    );
    // test_literal(
    //     "'2025-01-02 11:04:05.06789 +08:00'",
    //     DateTime::<Local>::from_str("2025-01-02T03:04:05.06789Z").unwrap(),
    // );
}

#[test]
fn test_chrono_offsets() {
    let utc = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap().and_hms_opt(12, 0, 0).unwrap();

    for (seconds, expect) in [
        (50_400, "'2025-01-03 02:00:00 +14:00'"),
        (-50_400, "'2025-01-01 22:00:00 -14:00'"),
        (-1_800, "'2025-01-02 11:30:00 -00:30'"),
    ] {
        let offset = FixedOffset::east_opt(seconds).unwrap();
        test_literal(expect, DateTime::<FixedOffset>::from_naive_utc_and_offset(utc, offset));
    }

    for seconds in [-50_460, -1, 1, 50_460] {
        let offset = FixedOffset::east_opt(seconds).unwrap();
        let value = DateTime::<FixedOffset>::from_naive_utc_and_offset(utc, offset);
        test_literal_error(value, |error| {
            assert!(matches!(error, SqlLiteralError::InvalidTimeZoneOffset { .. }))
        });
        let value = DateTime::<Local>::from_naive_utc_and_offset(utc, offset);
        test_literal_error(value, |error| {
            assert!(matches!(error, SqlLiteralError::InvalidTimeZoneOffset { .. }))
        });
    }
}

#[test]
fn test_chrono_local_offsets() {
    let utc = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap().and_hms_opt(12, 0, 0).unwrap();

    for (hours, expect) in [
        (8, "'2025-01-02 20:00:00 +08:00'"),
        (9, "'2025-01-02 21:00:00 +09:00'"),
        (8, "'2025-01-02 20:00:00 +08:00'"),
    ] {
        let offset = FixedOffset::east_opt(hours * 3_600).unwrap();
        test_literal(expect, DateTime::<Local>::from_naive_utc_and_offset(utc, offset));
    }
}

#[test]
fn test_chrono_leap_seconds() {
    for nanoseconds in [1_000_000_000, 1_500_000_000] {
        let time = NaiveTime::from_hms_nano_opt(23, 59, 59, nanoseconds).unwrap();
        let check = |error: &SqlLiteralError| {
            assert!(matches!(error, SqlLiteralError::LeapSecondUnsupported))
        };
        test_literal_error(time, check);
        let date_time = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap().and_time(time);
        test_literal_error(date_time, check);
        test_literal_error(DateTime::<Utc>::from_naive_utc_and_offset(date_time, Utc), check);
        let offset = FixedOffset::east_opt(0).unwrap();
        test_literal_error(
            DateTime::<FixedOffset>::from_naive_utc_and_offset(date_time, offset),
            check,
        );
        test_literal_error(DateTime::<Local>::from_naive_utc_and_offset(date_time, offset), check);
    }
}

#[test]
fn test_chrono_precision() {
    for (nanoseconds, expect) in
        [(99, "'03:04:05'"), (100, "'03:04:05.0000001'"), (999_999_999, "'03:04:05.9999999'")]
    {
        test_literal(expect, NaiveTime::from_hms_nano_opt(3, 4, 5, nanoseconds).unwrap());
    }
}

#[test]
fn test_chrono_leap_second_before_minute_end() {
    let value = NaiveTime::from_hms_opt(12, 34, 5).unwrap().with_nanosecond(1_500_000_000).unwrap();
    test_literal_error(value, |error| {
        assert!(matches!(error, SqlLiteralError::LeapSecondUnsupported))
    });
}
