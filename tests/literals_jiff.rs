#![cfg(feature = "jiff")]

mod common;

use common::test_literal;
use jiff::{
    civil::{Date, DateTime, Time},
    tz::{Offset, TimeZone},
    Timestamp,
};

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
