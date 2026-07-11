mod common;

use std::{borrow::Cow, rc::Rc, sync::Arc};

use common::test_literal;

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
    test_literal("N'中'", '中');
    test_literal("N'中文字'", "中文字");
    test_literal("N'中文字'", String::from("中文字"));
    test_literal("N'中文字'", Rc::new(String::from("中文字")));
    test_literal("N'中文字'", Arc::new(String::from("中文字")));
    test_literal("N'中文字'", Cow::from("中文字"));
}

#[test]
fn test_blob() {
    test_literal("0x", [].as_slice());
    test_literal("0x0102030405", [1u8, 2, 3, 4, 5].as_slice());
    test_literal("0x0102030405", vec![1u8, 2, 3, 4, 5]);
    test_literal("0x0102030405", Cow::from([1u8, 2, 3, 4, 5].as_slice()));
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
