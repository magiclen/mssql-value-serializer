#![cfg(feature = "bigdecimal")]

mod common;

use std::str::FromStr;

use bigdecimal::BigDecimal;
use common::test_literal;

#[test]
fn test_bigdecimal() {
    test_literal("1234567890", BigDecimal::from_str("1234567890").unwrap());
    test_literal("1234567890", BigDecimal::from_str("1234567890").unwrap().to_ref());
}
