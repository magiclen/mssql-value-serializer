use std::fmt::{self, Debug, Display, Write};

use mssql_value_serializer::{
    SqlLiteralError, SqlServerCharWrapper, SqlServerLiteral, SqlServerLiteralDynWrapper,
    SqlServerLiteralForValueListWrapper, SqlServerLiteralWrapper,
};

struct FailingWriter {
    remaining: usize,
    failed:    bool,
}

impl Write for FailingWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        assert!(!self.failed, "writing continued after a writer error");

        if s.len() > self.remaining {
            self.failed = true;
            return Err(fmt::Error);
        }

        self.remaining -= s.len();
        Ok(())
    }
}

fn test_writer_errors(value: impl Display) {
    let len = value.to_string().len();

    for remaining in 0..len {
        let mut out = FailingWriter {
            remaining,
            failed: false,
        };
        assert_eq!(Err(fmt::Error), write!(out, "{value}"));
    }
}

fn test_debug_writer_errors(value: impl Debug) {
    let len = format!("{value:?}").len();

    for remaining in 0..len {
        let mut out = FailingWriter {
            remaining,
            failed: false,
        };
        assert_eq!(Err(fmt::Error), write!(out, "{value:?}"));
    }
}

#[test]
fn test_literal_writer_errors() {
    test_writer_errors(SqlServerLiteralWrapper::new(true));
    test_writer_errors(SqlServerLiteralWrapper::new(123i32));
    test_writer_errors(SqlServerLiteralWrapper::new(8.8f64));
    test_writer_errors(SqlServerLiteralWrapper::new(1e-30f64));
    test_writer_errors(SqlServerLiteralWrapper::new(1e100f64));
    test_writer_errors(SqlServerLiteralWrapper::new(f32::MIN_POSITIVE));
    test_writer_errors(SqlServerLiteralWrapper::new('\''));
    test_writer_errors(SqlServerLiteralWrapper::new("O'Brien中"));
    test_writer_errors(SqlServerLiteralWrapper::new([0u8, 255].as_slice()));
    test_writer_errors(SqlServerLiteralWrapper::new(vec![0u8; 257]));
    test_writer_errors(SqlServerLiteralWrapper::new(None::<bool>));
    test_writer_errors(SqlServerCharWrapper::new('\''));
    test_writer_errors(SqlServerCharWrapper::new("O'Brien"));

    for value in
        [SqlServerLiteralDynWrapper::from("O'Brien"), SqlServerLiteralDynWrapper::from(true)]
    {
        test_writer_errors(&value);
        test_debug_writer_errors(&value);
    }
}

#[test]
fn test_value_list_writer_errors() {
    let values = SqlServerLiteralForValueListWrapper::new(vec![true, false]);
    test_writer_errors(&values);
    test_debug_writer_errors(&values);

    let values: Vec<Box<dyn SqlServerLiteral>> = vec![Box::new(true), Box::new(false)];
    let values = SqlServerLiteralForValueListWrapper::new(values);
    test_writer_errors(&values);

    let values: Vec<&dyn SqlServerLiteral> = vec![&true, &false];
    let values = SqlServerLiteralForValueListWrapper::new(values);
    test_writer_errors(&values);
}

#[test]
fn test_invalid_float_errors() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = value.append_sql_literal(&mut String::new()).unwrap_err();
        assert!(matches!(error, SqlLiteralError::FloatNotFinite));
    }
}

#[test]
#[should_panic(expected = "cannot format SQL Server literal: float value is not finite")]
fn test_invalid_float_display() {
    let mut out = String::new();
    write!(out, "{}", SqlServerLiteralWrapper::new(f64::NAN)).unwrap();
}

#[test]
fn test_invalid_float_debug() {
    let value = SqlServerLiteralDynWrapper::from(f64::NAN);
    assert_eq!("Owned(<invalid: float value is not finite>)", format!("{value:?}"));

    let values = SqlServerLiteralForValueListWrapper::new(vec![1.0f64, f64::NAN]);
    assert_eq!(
        "SqlServerLiteralForValueListWrapper(1, <invalid: float value is not finite>)",
        format!("{values:?}")
    );
}

#[test]
fn test_feature_writer_errors() {
    #[cfg(feature = "chrono")]
    test_writer_errors(SqlServerLiteralWrapper::new(
        chrono::NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
    ));
    #[cfg(feature = "jiff")]
    test_writer_errors(SqlServerLiteralWrapper::new(jiff::civil::Date::new(2025, 1, 2).unwrap()));
    #[cfg(feature = "time")]
    test_writer_errors(SqlServerLiteralWrapper::new(
        time::Date::from_calendar_date(2025, time::Month::January, 2).unwrap(),
    ));
    #[cfg(feature = "rust_decimal")]
    test_writer_errors(SqlServerLiteralWrapper::new(rust_decimal::Decimal::from(123)));
    #[cfg(feature = "bigdecimal")]
    {
        let value = bigdecimal::BigDecimal::from(123);
        test_writer_errors(SqlServerLiteralWrapper::new(value.to_ref()));
        test_writer_errors(SqlServerLiteralWrapper::new(value));
    }
    #[cfg(feature = "num-bigint")]
    {
        test_writer_errors(SqlServerLiteralWrapper::new(num_bigint::BigInt::from(-123)));
        test_writer_errors(SqlServerLiteralWrapper::new(num_bigint::BigUint::from(123u32)));
    }
    #[cfg(feature = "uuid")]
    test_writer_errors(SqlServerLiteralWrapper::new(uuid::Uuid::nil()));
}
