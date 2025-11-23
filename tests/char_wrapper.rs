mod common;

use std::{borrow::Cow, rc::Rc, sync::Arc};

use common::test_literal;
use mssql_value_serializer::SqlServerCharWrapper;

#[test]
fn test_char_wrapper() {
    test_literal("'a'", SqlServerCharWrapper::from('a'));
    test_literal("'abc'", SqlServerCharWrapper::from("abc"));
    test_literal("'abc'", SqlServerCharWrapper::from(String::from("abc")));
    test_literal("'abc'", SqlServerCharWrapper::from(Rc::new(String::from("abc"))));
    test_literal("'abc'", SqlServerCharWrapper::from(Arc::new(String::from("abc"))));
    test_literal("'abc'", SqlServerCharWrapper::from(Cow::from("abc")));
}
