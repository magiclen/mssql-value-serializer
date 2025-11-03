#![cfg(feature = "num-bigint")]

mod common;

use std::str::FromStr;

use common::test_literal;
use num_bigint::{BigInt, BigUint};

#[test]
fn test_bigdecimal() {
    test_literal("1234567890", BigInt::from_str("1234567890").unwrap());
    test_literal("1234567890", BigUint::from_str("1234567890").unwrap());
}
