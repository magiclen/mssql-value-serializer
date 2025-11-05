#![cfg(feature = "uuid")]

mod common;

use std::str::FromStr;

use common::test_literal;
use uuid::Uuid;

#[test]
fn test_uuid() {
    test_literal(
        "6F9619FF-8B86-D011-B42D-00C04FC964FF",
        Uuid::from_str("6F9619FF-8B86-D011-B42D-00C04FC964FF").unwrap(),
    );
}
