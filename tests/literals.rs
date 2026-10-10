mod common;

use std::{borrow::Cow, rc::Rc, sync::Arc};

use common::{test_literal, test_literal_error};
use mssql_value_serializer::SqlLiteralError;

#[test]
fn test_boolean() {
    test_literal("1", true);
    test_literal("0", false);
}

#[test]
fn test_numbers() {
    test_literal("8", 8i8);
    test_literal("8", 8i16);
    test_literal("8", 8i32);
    test_literal("8", 8i64);
    test_literal("8", 8i128);
    test_literal("8", 8isize);
    test_literal("8", 8u8);
    test_literal("8", 8u16);
    test_literal("8", 8u32);
    test_literal("8", 8u64);
    test_literal("8", 8u128);
    test_literal("8", 8usize);

    test_literal("8.8", 8.8f32);
    test_literal("8.8", 8.8f64);
}

#[test]
fn test_strings() {
    test_literal("N''", "");
    test_literal("N''''", '\'');
    test_literal("N'O''Brien'", "O'Brien");
    test_literal("N''''''", "''");
    test_literal("N'01234567890123456789''中文''text'", "01234567890123456789'中文'text");
    test_literal("N'中'", '中');
    test_literal("N'中文字'", "中文字");
    test_literal("N'中文字'", String::from("中文字"));
    test_literal("N'中文字'", Rc::new(String::from("中文字")));
    test_literal("N'中文字'", Arc::new(String::from("中文字")));
    test_literal("N'中文字'", Cow::from("中文字"));
}

#[test]
fn test_float_ranges() {
    test_literal("0", 0.0f32);
    test_literal("-0", -0.0f32);
    test_literal("0", 0.0f64);
    test_literal("-0", -0.0f64);
    test_literal(&format!("1{}", "0".repeat(37)), 1e37f64);
    test_literal(&format!("0.{}1", "0".repeat(37)), 1e-38f64);

    for value in [
        1e38f64,
        1e100,
        -1e100,
        1e-39,
        1e-100,
        f64::MAX,
        -f64::MAX,
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
    ] {
        test_literal(&format!("CAST('{value}' AS float(53))"), value);
    }

    for value in [f32::MAX, -f32::MAX, f32::MIN_POSITIVE, -f32::MIN_POSITIVE] {
        test_literal(&format!("CAST('{value}' AS real)"), value);
    }

    for bits in [1, f32::MIN_POSITIVE.to_bits() - 1] {
        for value in [f32::from_bits(bits), -f32::from_bits(bits)] {
            test_literal_error(value, |error| {
                assert!(matches!(error, SqlLiteralError::FloatSubnormal))
            });
        }
    }
    for bits in [1, f64::MIN_POSITIVE.to_bits() - 1] {
        for value in [f64::from_bits(bits), -f64::from_bits(bits)] {
            test_literal_error(value, |error| {
                assert!(matches!(error, SqlLiteralError::FloatSubnormal))
            });
        }
    }
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        test_literal_error(value, |error| {
            assert!(matches!(error, SqlLiteralError::FloatNotFinite))
        });
    }
}

#[test]
fn test_integer_ranges() {
    let max = 10u128.pow(38) - 1;
    test_literal(&max.to_string(), max);
    test_literal(&max.to_string(), max as i128);
    test_literal(&format!("-{max}"), -(max as i128));

    for value in [max + 1, u128::MAX] {
        test_literal_error(value, |error| {
            assert!(matches!(error, SqlLiteralError::NumericOutOfRange))
        });
    }
    for value in [(max + 1) as i128, -((max + 1) as i128), i128::MIN, i128::MAX] {
        test_literal_error(value, |error| {
            assert!(matches!(error, SqlLiteralError::NumericOutOfRange))
        });
    }
}

#[test]
fn test_blob() {
    test_literal("0x", [].as_slice());
    test_literal("0x0102030405", [1u8, 2, 3, 4, 5].as_slice());
    test_literal("0x0102030405", vec![1u8, 2, 3, 4, 5]);
    test_literal("0x0102030405", Cow::from([1u8, 2, 3, 4, 5].as_slice()));
}

#[test]
fn test_blob_append() {
    use mssql_value_serializer::SqlServerLiteral;

    let bytes: Vec<u8> = (0..=255).collect();
    let expect = format!("中文字0x{}", const_hex::encode_upper(&bytes));

    for capacity in [0, expect.len() * 2] {
        let mut s = String::with_capacity(capacity);
        s.push_str("中文字");
        bytes.append_sql_literal(&mut s).unwrap();
        assert_eq!(expect, s);

        s.clear();
        s.push_str("中文字");
        bytes.append_sql_literal(&mut s).unwrap();
        assert_eq!(expect, s);
    }
}

#[test]
fn test_blob_large_append() {
    use mssql_value_serializer::SqlServerLiteral;

    let bytes = vec![0xABu8; 65_537];
    let expect = format!("中文字0x{}", "AB".repeat(bytes.len()));

    for capacity in [0, expect.len() * 2] {
        let mut s = String::with_capacity(capacity);
        s.push_str("中文字");
        bytes.append_sql_literal(&mut s).unwrap();
        assert_eq!(expect, s);
    }
}

#[test]
fn test_null() {
    test_literal("NULL", None::<bool>);
    test_literal("NULL", None::<i8>);
    test_literal("NULL", None::<f32>);
    test_literal("NULL", None::<&str>);
    test_literal("NULL", None::<String>);
    test_literal("NULL", None::<Cow<'_, str>>);
    test_literal("NULL", None::<&[u8]>);
    test_literal("NULL", None::<Vec<u8>>);
    test_literal("NULL", None::<Cow<'_, [u8]>>);
}
