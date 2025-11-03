use std::borrow::Cow;

use mssql_value_serializer::{SqlServerLiteralDynWrapper, SqlServerLiteralForValueList};

#[allow(clippy::vec_init_then_push)]
#[test]
fn test_dyn_wrapper() {
    let mut v: Vec<SqlServerLiteralDynWrapper<'_>> = Vec::new();

    v.push(SqlServerLiteralDynWrapper::from(true));
    v.push(SqlServerLiteralDynWrapper::from(&false));
    v.push(SqlServerLiteralDynWrapper::from(10i32));
    v.push(SqlServerLiteralDynWrapper::from(&20i32));
    v.push(SqlServerLiteralDynWrapper::from('1'));
    v.push(SqlServerLiteralDynWrapper::from(&'2'));
    v.push(SqlServerLiteralDynWrapper::from("123"));
    v.push(SqlServerLiteralDynWrapper::from(&"456"));
    v.push(SqlServerLiteralDynWrapper::from(String::from("123")));
    let s = String::from("456");
    v.push(SqlServerLiteralDynWrapper::from(&s));
    v.push(SqlServerLiteralDynWrapper::from(Cow::from("123")));
    let s = Cow::from(String::from("456"));
    v.push(SqlServerLiteralDynWrapper::from(&s));
    v.push(SqlServerLiteralDynWrapper::from([1, 2, 3].as_slice()));
    let b = [4, 5, 6].as_slice();
    v.push(SqlServerLiteralDynWrapper::from(&b));
    v.push(SqlServerLiteralDynWrapper::from(vec![1, 2, 3]));
    let b = vec![4, 5, 6];
    v.push(SqlServerLiteralDynWrapper::from(&b));
    let b = Cow::from(vec![4, 5, 6]);
    v.push(SqlServerLiteralDynWrapper::from(&b));
    v.push(SqlServerLiteralDynWrapper::from(None::<&str>));
    v.push(SqlServerLiteralDynWrapper::from(&None::<&str>));

    let mut s = String::new();

    v.append_sql_literal_for_value_list(&mut s).unwrap();

    assert_eq!(
        "1, 0, 10, 20, N'1', N'2', N'123', N'456', N'123', N'456', N'123', N'456', 0x010203, \
         0x040506, 0x010203, 0x040506, 0x040506, NULL, NULL",
        s
    );
}
