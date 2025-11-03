#![allow(dead_code)]

use mssql_value_serializer::SqlServerLiteral;

pub fn test_literal<T: SqlServerLiteral>(expect: &'static str, literal: T) {
    let mut s = String::new();

    literal.append_sql_literal(&mut s).unwrap();

    assert_eq!(expect, s.as_str());
}
