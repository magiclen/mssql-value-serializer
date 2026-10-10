#![cfg(feature = "num-bigint")]

mod common;

use std::str::FromStr;

use common::{test_literal, test_literal_error};
use mssql_value_serializer::SqlLiteralError;
use num_bigint::{BigInt, BigUint};

#[test]
fn test_num_bigint() {
    test_literal("1234567890", BigInt::from_str("1234567890").unwrap());
    test_literal("1234567890", BigUint::from_str("1234567890").unwrap());
}

#[test]
fn test_bigint_ranges() {
    let max = 10u128.pow(38) - 1;
    test_literal(&max.to_string(), BigUint::from(max));
    test_literal(&max.to_string(), BigInt::from(max));
    test_literal(&format!("-{max}"), -BigInt::from(max));
    let check =
        |error: &SqlLiteralError| assert!(matches!(error, SqlLiteralError::NumericOutOfRange));
    test_literal_error(BigUint::from(max + 1), check);
    test_literal_error(BigInt::from(max + 1), check);
    test_literal_error(-BigInt::from(max + 1), check);
}
