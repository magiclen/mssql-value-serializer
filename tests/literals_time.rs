#![cfg(feature = "time")]

mod common;

use common::test_literal;
use time::{
    format_description::well_known::Rfc3339, Date, Month, OffsetDateTime, PrimitiveDateTime, Time,
    UtcDateTime,
};

#[test]
fn test_time() {
    test_literal("'2025-01-02'", Date::from_calendar_date(2025, Month::January, 2).unwrap());
    test_literal("'03:04:05.0000678'", Time::from_hms_nano(3, 4, 5, 67890).unwrap());
    test_literal("'03:04:05.06789'", Time::from_hms_nano(3, 4, 5, 67890000).unwrap());
    test_literal(
        "'2025-01-02 03:04:05.06789'",
        PrimitiveDateTime::new(
            Date::from_calendar_date(2025, Month::January, 2).unwrap(),
            Time::from_hms_nano(3, 4, 5, 67890000).unwrap(),
        ),
    );
    test_literal(
        "'2025-01-02 03:04:05.06789 -01:02'",
        OffsetDateTime::parse("2025-01-02T03:04:05.06789-01:02", &Rfc3339).unwrap(),
    );
    test_literal(
        "'2025-01-02 03:04:05.06789 +00:00'",
        UtcDateTime::parse("2025-01-02T03:04:05.06789Z", &Rfc3339).unwrap(),
    );
    // test_literal(
    //     "'2025-01-02 11:04:05.06789 +08:00'",
    //     DateTime::<Local>::from_str("2025-01-02T03:04:05.06789Z").unwrap(),
    // );
}
