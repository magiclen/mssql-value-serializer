mssql-value-serializer
====================

[![CI](https://github.com/magiclen/mssql-value-serializer/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/mssql-value-serializer/actions/workflows/ci.yml)

Convert Rust values into SQL Server-compatible literal expressions, enabling dynamic SQL generation without parameter count limitations.

Prepared statements are commonly used to improve performance and security. By separating the SQL command from the data values, the database can cache execution plans and protect against SQL injection. Each variable passed to the query becomes a parameter, allowing safe reuse of the same statement with different values. However, because SQL Server enforces a strict 2100-parameter limit, queries that bind large collections of parameters—such as long `IN` lists or bulk inserts—can easily exceed this cap and trigger the **too many parameters** error.

## Usage

```rust
use mssql_value_serializer::{SqlServerCharWrapper, SqlServerLiteralWrapper, SqlServerLiteralDynWrapper, SqlServerLiteralForValueListWrapper};

let sql = format!("
    SELECT
        *
    FROM
        [table]
    WHERE
        [name] = {name}
", name = SqlServerLiteralWrapper::new("David"));

assert_eq!("
    SELECT
        *
    FROM
        [table]
    WHERE
        [name] = N'David'
", sql);

let sql = format!("
    SELECT
        *
    FROM
        [table]
    WHERE
        [name] = {name}
", name = SqlServerCharWrapper::new("David")); // use `SqlServerCharWrapper` to format a value into a non-Unicode character string

assert_eq!("
    SELECT
        *
    FROM
        [table]
    WHERE
        [name] = 'David'
", sql);

let sql = format!("
    INSERT INTO [table]([id], [name], [disabled])
        VALUES
            ({values})
", values = SqlServerLiteralForValueListWrapper::new(vec![SqlServerLiteralDynWrapper::from(2u32), SqlServerLiteralDynWrapper::from("David"), SqlServerLiteralDynWrapper::from(false)]));

assert_eq!("
    INSERT INTO [table]([id], [name], [disabled])
        VALUES
            (2, N'David', 0)
", sql);
```

## Optional Features

* `serde`: Implements `serde::Serialize` for wrapper types, enabling SQL Server literal serialization behavior.
* `chrono`, `jiff`, or `time`: Adds support for SQL Server date and time types.
    * `chrono` and `stable-local`: Caches the first UTC offset used by `DateTime<Local>` to speed up formatting; values with other offsets are formatted directly.
* `rust_decimal` or `bigdecimal`: Adds support for SQL Server decimal/numeric types.
* `num-bigint`: Adds support for SQL Server decimal/numeric types (only integers).
* `uuid`: Adds support for SQL Server UNIQUEIDENTIFIER type.

## Literal Behavior

* Displaying an invalid value with a wrapper panics; use the append methods or Serde to handle value errors.
* `Debug` on `SqlServerLiteralDynWrapper` and `SqlServerLiteralForValueListWrapper` shows value errors instead of panicking.
* Writer errors are returned as `SqlLiteralError::Write` and do not cause a panic.
* Normal floating-point values use plain decimal text when it fits SQL Server's 38-digit decimal limit.
* Larger or smaller floating-point values use scientific notation, which SQL Server reads as `float`; `f32` values are also cast to `real`.
* Scientific notation produces floating-point values, not exact decimal values.
* NaN, infinity, and subnormal floating-point values are rejected.
* BigDecimal values use exact plain decimal text and keep their original scale and trailing zeros.
* BigDecimal, BigInt, BigUint, i128, and u128 values outside the 38-digit decimal range are rejected before writing.
* BigDecimal scale is checked even for zero, so an unsupported scale is rejected instead of being normalized.
* Time zone offsets must be whole minutes from -14:00 to +14:00, and leap seconds are rejected.
* Fractional seconds are truncated to 100 ns, and years outside 1 to 9999 are rejected.
* Values with a UTC offset must have valid local and UTC years; other date limits of the target SQL Server type are not checked.
* Date and time literals are unambiguous for `date`, `time`, `datetime2`, and `datetimeoffset`; for `datetime` and `smalldatetime`, the date order depends on `SET DATEFORMAT`, and more than three fractional digits can fail to convert.
* A value list keeps earlier output if a later value fails.
* Negative zero keeps its literal text; its sign bit is not guaranteed after SQL storage.

## Crates.io

https://crates.io/crates/mssql-value-serializer

## Documentation

https://docs.rs/mssql-value-serializer

## License

[MIT](LICENSE)
