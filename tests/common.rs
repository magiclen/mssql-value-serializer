#![allow(dead_code)]

use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter, Write},
};

use mssql_value_serializer::{
    SqlLiteralError, SqlServerLiteral, SqlServerLiteralForValueList,
    SqlServerLiteralForValueListWrapper, SqlServerLiteralWrapper,
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

pub fn test_literal_error<T: SqlServerLiteral>(literal: T, check: impl Fn(&SqlLiteralError)) {
    let mut s = String::from("prefix");
    check(&literal.append_sql_literal(&mut s).unwrap_err());
    assert_eq!("prefix", s);

    struct Capture<'a, T>(&'a T, RefCell<Option<SqlLiteralError>>);

    impl<T: SqlServerLiteral> Display for Capture<'_, T> {
        fn fmt(&self, out: &mut Formatter<'_>) -> fmt::Result {
            *self.1.borrow_mut() = self.0.append_sql_literal_fmt(out).err();
            Ok(())
        }
    }

    let capture = Capture(&literal, RefCell::new(None));
    write!(s, "{capture}").unwrap();
    check(capture.1.borrow().as_ref().unwrap());
    assert_eq!("prefix", s);
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
