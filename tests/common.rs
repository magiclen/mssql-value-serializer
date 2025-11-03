#![allow(dead_code)]

use std::fmt::Write;

use mssql_value_serializer::{
    SqlServerLiteral, SqlServerLiteralForValueList, SqlServerLiteralForValueListWrapper,
    SqlServerLiteralWrapper,
};

pub fn test_literal<T: SqlServerLiteral>(expect: &'static str, literal: T) {
    {
        let mut s = String::new();

        literal.append_sql_literal(&mut s).unwrap();

        assert_eq!(expect, s.as_str());
    }

    {
        let mut s = String::new();

        s.write_fmt(format_args!("{}", SqlServerLiteralWrapper::new(literal))).unwrap();

        assert_eq!(expect, s.as_str());
    }
}

pub fn test_literals<T: SqlServerLiteralForValueList>(expect: &[&'static str], literals: T) {
    let validate = |s: &str| match expect.len() {
        0 => (),
        1 => assert_eq!(expect[0], s),
        _ => {
            let mut i = 0usize;
            let mut pass = false;

            while i < expect.len() {
                if s.eq(expect[i]) {
                    pass = true;

                    break;
                }

                i += 1;
            }

            if !pass {
                assert_eq!(expect[i], s)
            }
        },
    };

    {
        let mut s = String::new();

        literals.append_sql_literal_for_value_list(&mut s).unwrap();

        validate(s.as_str());
    }

    {
        let mut s = String::new();

        s.write_fmt(format_args!("{}", SqlServerLiteralForValueListWrapper::new(literals)))
            .unwrap();

        validate(s.as_str());
    }
}
