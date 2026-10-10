#![cfg(feature = "bigdecimal")]

mod common;

use std::str::FromStr;

use bigdecimal::BigDecimal;
use common::{test_literal, test_literal_error};
use mssql_value_serializer::SqlLiteralError;

#[test]
fn test_bigdecimal() {
    test_literal("1234567890", BigDecimal::from_str("1234567890").unwrap());
    test_literal("1234567890", BigDecimal::from_str("1234567890").unwrap().to_ref());
}

#[test]
fn test_bigdecimal_precision() {
    for text in [
        "0.0000001234567890123456789",
        "-0.0000001234567890123456789",
        "1.2300",
        "99999999999999999999999999999999999999",
        "0.00000000000000000000000000000000000001",
    ] {
        let value = BigDecimal::from_str(text).unwrap();
        test_literal(text, value.to_ref());
        test_literal(text, value);
    }
    let value = BigDecimal::from_str("1e20").unwrap();
    test_literal("100000000000000000000", value.to_ref());
    test_literal("100000000000000000000", value);
    let value = BigDecimal::from_str("0.00").unwrap();
    test_literal("0.00", value.to_ref());
    test_literal("0.00", value);
    for (scale, expect) in [(38, format!("0.{}", "0".repeat(38))), (-37, "0".repeat(38))] {
        let value = BigDecimal::new(0.into(), scale);
        test_literal(&expect, value.to_ref());
        test_literal(&expect, value);
    }
}

#[test]
fn test_bigdecimal_ranges() {
    let check =
        |error: &SqlLiteralError| assert!(matches!(error, SqlLiteralError::NumericOutOfRange));
    for text in [
        "1e38",
        "1e-39",
        "100000000000000000000000000000000000000",
        "1.00000000000000000000000000000000000000",
    ] {
        let value = BigDecimal::from_str(text).unwrap();
        test_literal_error(value.to_ref(), check);
        test_literal_error(value, check);
    }
    for scale in [-38, 39, i64::MIN, i64::MAX] {
        for coefficient in [0, 1] {
            let value = BigDecimal::new(coefficient.into(), scale);
            test_literal_error(value.to_ref(), check);
            test_literal_error(value, check);
        }
    }
}
