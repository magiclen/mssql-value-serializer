#![cfg(feature = "time")]

mod common;

use common::{test_literal, test_literal_error};
use mssql_value_serializer::SqlLiteralError;
use time::{
    Date, Month, OffsetDateTime, PlainDateTime, Time, UtcDateTime, UtcOffset,
    format_description::well_known::Rfc3339,
};

#[test]
fn test_time() {
    test_literal("'2025-01-02'", Date::from_calendar_date(2025, Month::January, 2).unwrap());
    test_literal("'03:04:05.0000678'", Time::from_hms_nano(3, 4, 5, 67890).unwrap());
    test_literal("'03:04:05.06789'", Time::from_hms_nano(3, 4, 5, 67890000).unwrap());
    test_literal(
        "'2025-01-02 03:04:05.06789'",
        PlainDateTime::new(
            Date::from_calendar_date(2025, Month::January, 2).unwrap(),
            Time::from_hms_nano(3, 4, 5, 67890000).unwrap(),
        ),
    );
    test_literal(
        "'2025-01-02 03:04:05.06789 -01:02'",
        OffsetDateTime::parse("2025-01-02T03:04:05.06789-01:02", &Rfc3339).unwrap(),
    );
    test_literal(
        "'2025-01-02 03:04:05.06789 -00:30'",
        OffsetDateTime::parse("2025-01-02T03:04:05.06789-00:30", &Rfc3339).unwrap(),
    );
    test_literal(
        "'2025-01-02 03:04:05.06789 +00:00'",
        UtcDateTime::parse("2025-01-02T03:04:05.06789Z", &Rfc3339).unwrap(),
    );
}

#[test]
fn test_time_offsets() {
    let value = PlainDateTime::new(
        Date::from_calendar_date(2025, Month::January, 2).unwrap(),
        Time::from_hms(12, 0, 0).unwrap(),
    );

    for (seconds, expect) in [
        (50_400, "'2025-01-02 12:00:00 +14:00'"),
        (-50_400, "'2025-01-02 12:00:00 -14:00'"),
        (-1_800, "'2025-01-02 12:00:00 -00:30'"),
    ] {
        test_literal(expect, value.assume_offset(UtcOffset::from_whole_seconds(seconds).unwrap()));
    }

    for seconds in [-50_460, -1, 1, 50_460] {
        let value = value.assume_offset(UtcOffset::from_whole_seconds(seconds).unwrap());
        test_literal_error(value, |error| {
            assert!(matches!(error, SqlLiteralError::InvalidTimeZoneOffset { .. }))
        });
    }
}

#[test]
fn test_time_precision() {
    for (nanoseconds, expect) in
        [(99, "'03:04:05'"), (100, "'03:04:05.0000001'"), (999_999_999, "'03:04:05.9999999'")]
    {
        test_literal(expect, Time::from_hms_nano(3, 4, 5, nanoseconds).unwrap());
    }
}

#[test]
fn test_time_year_ranges() {
    test_literal("'0001-01-01'", Date::from_calendar_date(1, Month::January, 1).unwrap());
    test_literal("'9999-12-31'", Date::from_calendar_date(9999, Month::December, 31).unwrap());
    test_literal_error(Date::from_calendar_date(0, Month::January, 1).unwrap(), |error| {
        assert!(matches!(error, SqlLiteralError::DateOutOfRange { .. }))
    });
}

#[test]
fn test_time_offset_year_ranges() {
    let lower = Date::from_calendar_date(1, Month::January, 1).unwrap();
    let upper = Date::from_calendar_date(9999, Month::December, 31).unwrap();
    let positive = UtcOffset::from_whole_seconds(60).unwrap();
    let negative = UtcOffset::from_whole_seconds(-60).unwrap();

    test_literal(
        "'0001-01-01 00:01:00 +00:01'",
        lower.with_hms(0, 1, 0).unwrap().assume_offset(positive),
    );
    test_literal(
        "'9999-12-31 23:58:59 -00:01'",
        upper.with_hms(23, 58, 59).unwrap().assume_offset(negative),
    );

    for (value, year) in [
        (lower.midnight().assume_offset(positive), 0),
        (upper.with_hms(23, 59, 59).unwrap().assume_offset(negative), 10_000),
        (
            Date::from_calendar_date(0, Month::December, 31)
                .unwrap()
                .with_hms(23, 59, 0)
                .unwrap()
                .assume_offset(negative),
            0,
        ),
    ] {
        test_literal_error(value, |error| {
            assert!(
                matches!(error, SqlLiteralError::DateOutOfRange { year: actual } if *actual == year)
            );
        });
    }
}
