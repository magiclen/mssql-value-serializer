mod common;

use std::collections::HashSet;

use common::test_literals;
use mssql_value_serializer::{append_sql_literal_for_value_list, SqlServerLiteral};

#[test]
fn test_append_sql_literal_for_value_list_empty() {
    let mut s = String::new();

    append_sql_literal_for_value_list(&[], &mut s).unwrap();

    assert!(s.is_empty());
}

#[test]
fn test_append_sql_literal_for_value_list() {
    let mut s = String::new();

    append_sql_literal_for_value_list(
        &[&true, &123, &45.6, &"789", &b"blob data".as_slice(), &None::<&str>],
        &mut s,
    )
    .unwrap();

    assert_eq!("1, 123, 45.6, N'789', 0x626C6F622064617461, NULL", s);
}

#[test]
fn test_append_sql_literal_for_value_list_static_vec() {
    let vec: Vec<i32> = vec![1, 2];

    test_literals(&["1, 2"], vec);
}

#[test]
fn test_append_sql_literal_for_value_list_static_hast_set() {
    let mut set: HashSet<i32> = HashSet::new();

    set.insert(1);
    set.insert(2);

    test_literals(&["1, 2", "2, 1"], set);
}

#[test]
fn test_append_sql_literal_for_value_list_dyn_vec_box() {
    let vec: Vec<Box<dyn SqlServerLiteral>> = vec![Box::new(1i32), Box::new(2u32)];

    test_literals(&["1, 2"], vec);
}

#[test]
fn test_append_sql_literal_for_value_list_dyn_vec_ref() {
    let vec: Vec<&dyn SqlServerLiteral> = vec![&1i32, &2u32];

    test_literals(&["1, 2"], vec.as_slice());
    test_literals(&["1, 2"], vec);
}
