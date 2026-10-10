#![cfg(feature = "jiff")]

mod common;

use common::{test_literal, test_literal_error};
use jiff::{
    Timestamp,
    civil::{Date, DateTime, Time},
    tz::{Offset, TimeZone},
};
use mssql_value_serializer::SqlLiteralError;

#[test]
fn test_jiff() {
    test_literal("'2025-01-02'", Date::new(2025, 1, 2).unwrap());
    test_literal("'03:04:05.0000678'", Time::new(3, 4, 5, 67890).unwrap());
    test_literal("'03:04:05.06789'", Time::new(3, 4, 5, 67890000).unwrap());

    let date_time =
        DateTime::from_parts(Date::new(2025, 1, 2).unwrap(), Time::new(3, 4, 5, 67890000).unwrap());

    test_literal("'2025-01-02 03:04:05.06789'", date_time);
    test_literal(
        "'2025-01-02 03:04:05.06789 -01:02'",
        date_time
            .to_zoned(TimeZone::fixed(Offset::from_seconds(-(60 * 60 + 2 * 60)).unwrap()))
            .unwrap(),
    );
    test_literal(
        "'2025-01-02 03:04:05.06789 +00:00'",
        "2025-01-02T03:04:05.06789Z".parse::<Timestamp>().unwrap(),
    );
}

#[test]
fn test_jiff_offsets() {
    let value =
        DateTime::from_parts(Date::new(2025, 1, 2).unwrap(), Time::new(12, 0, 0, 0).unwrap());

    for (seconds, expect) in [
        (50_400, "'2025-01-02 12:00:00 +14:00'"),
        (-50_400, "'2025-01-02 12:00:00 -14:00'"),
        (-1_800, "'2025-01-02 12:00:00 -00:30'"),
    ] {
        test_literal(
            expect,
            value.to_zoned(TimeZone::fixed(Offset::from_seconds(seconds).unwrap())).unwrap(),
        );
    }

    for seconds in [-50_460, -1, 1, 50_460] {
        let value =
            value.to_zoned(TimeZone::fixed(Offset::from_seconds(seconds).unwrap())).unwrap();
        test_literal_error(value, |error| {
            assert!(matches!(error, SqlLiteralError::InvalidTimeZoneOffset { .. }))
        });
    }
}

#[test]
fn test_jiff_precision() {
    for (nanoseconds, expect) in
        [(99, "'03:04:05'"), (100, "'03:04:05.0000001'"), (999_999_999, "'03:04:05.9999999'")]
    {
        test_literal(expect, Time::new(3, 4, 5, nanoseconds).unwrap());
    }
}
