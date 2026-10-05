#![cfg(feature = "uuid")]

mod common;

use std::str::FromStr;

use common::{test_literal, test_literals};
use mssql_value_serializer::SqlServerLiteral;
use uuid::Uuid;

#[test]
fn test_uuid() {
    test_literal(
        "'6F9619FF-8B86-D011-B42D-00C04FC964FF'",
        Uuid::from_str("6F9619FF-8B86-D011-B42D-00C04FC964FF").unwrap(),
    );
}

#[test]
fn test_uuid_from_lowercase_text() {
    test_literal(
        "'6F9619FF-8B86-D011-B42D-00C04FC964FF'",
        Uuid::from_str("6f9619ff-8b86-d011-b42d-00c04fc964ff").unwrap(),
    );
}

#[test]
fn test_nil_uuid() {
    test_literal(
        "'00000000-0000-0000-0000-000000000000'",
        Uuid::from_str("00000000-0000-0000-0000-000000000000").unwrap(),
    );
}

#[test]
fn test_uuid_in_value_list() {
    let first = Uuid::from_str("6F9619FF-8B86-D011-B42D-00C04FC964FF").unwrap();
    let second = Uuid::from_str("00000000-0000-0000-0000-000000000000").unwrap();

    test_literals(
        &["'6F9619FF-8B86-D011-B42D-00C04FC964FF', '00000000-0000-0000-0000-000000000000'"],
        vec![first, second],
    );
}

#[test]
fn test_uuid_preserves_existing_output() {
    let value = Uuid::from_str("6F9619FF-8B86-D011-B42D-00C04FC964FF").unwrap();
    let mut out = String::from("VALUES (");

    value.append_sql_literal(&mut out).unwrap();
    out.push(')');

    assert_eq!("VALUES ('6F9619FF-8B86-D011-B42D-00C04FC964FF')", out);
}
