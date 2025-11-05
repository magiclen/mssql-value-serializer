#![cfg(feature = "rust_decimal")]

mod common;

use std::str::FromStr;

use common::test_literal;
use rust_decimal::Decimal;

#[test]
fn test_rust_decimal() {
    test_literal("1234567890", Decimal::from_str("1234567890").unwrap());
}
