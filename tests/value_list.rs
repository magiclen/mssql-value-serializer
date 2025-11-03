use mssql_value_serializer::append_sql_literal_for_value_list;

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
