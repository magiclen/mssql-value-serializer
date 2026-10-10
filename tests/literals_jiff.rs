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

#[test]
fn test_jiff_year_ranges() {
    test_literal("'0001-01-01'", Date::new(1, 1, 1).unwrap());
    test_literal("'9999-12-31'", Date::new(9999, 12, 31).unwrap());
    test_literal(
        "'0001-01-01 00:00:00 +00:00'",
        "0001-01-01T00:00:00Z".parse::<Timestamp>().unwrap(),
    );

    let check =
        |error: &SqlLiteralError| assert!(matches!(error, SqlLiteralError::DateOutOfRange { .. }));
    test_literal_error(Date::new(0, 12, 31).unwrap(), check);
    test_literal_error("0000-12-31T23:59:59Z".parse::<Timestamp>().unwrap(), check);
}

#[test]
fn test_jiff_timestamp_year_boundary() {
    let boundary = -62_135_596_800;
    test_literal_error(Timestamp::new(boundary, -1).unwrap(), |error| {
        assert!(matches!(error, SqlLiteralError::DateOutOfRange {
            year: 0
        }));
    });

    for nanoseconds in [0, 1] {
        test_literal(
            "'0001-01-01 00:00:00 +00:00'",
            Timestamp::new(boundary, nanoseconds).unwrap(),
        );
    }
}

#[test]
fn test_jiff_zoned_year_ranges() {
    let offset = TimeZone::fixed(Offset::from_seconds(60).unwrap());
    let value = Date::new(1, 1, 1).unwrap().at(0, 1, 0, 0).to_zoned(offset.clone()).unwrap();
    test_literal("'0001-01-01 00:01:00 +00:01'", value);

    let value = Date::new(1, 1, 1).unwrap().at(0, 0, 0, 0).to_zoned(offset).unwrap();
    test_literal_error(value, |error| {
        assert!(matches!(error, SqlLiteralError::DateOutOfRange {
            year: 0
        }))
    });

    let offset = TimeZone::fixed(Offset::from_seconds(-60).unwrap());
    let value = Date::new(0, 12, 31).unwrap().at(23, 59, 0, 0).to_zoned(offset).unwrap();
    test_literal_error(value, |error| {
        assert!(matches!(error, SqlLiteralError::DateOutOfRange {
            year: 0
        }))
    });
}
