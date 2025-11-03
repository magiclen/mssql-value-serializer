#![cfg(feature = "chrono")]

mod common;

use std::str::FromStr;

use chrono::prelude::*;
use common::test_literal;

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
