/*!
# mssql-value-serializer

Convert Rust values into SQL Server-compatible literal expressions, enabling dynamic SQL generation without parameter count limitations.

Prepared statements are commonly used to improve performance and security. By separating the SQL command from the data values, the database can cache execution plans and protect against SQL injection. Each variable passed to the query becomes a parameter, allowing safe reuse of the same statement with different values. However, because SQL Server enforces a strict 2100-parameter limit, queries that bind large collections of parameters—such as long `IN` lists or bulk inserts—can easily exceed this cap and trigger the **too many parameters** error.

## Usage

```rust
use mssql_value_serializer::{SqlServerLiteralWrapper, SqlServerLiteralDynWrapper, SqlServerLiteralForValueListWrapper};

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
* `chrono` or `time`: Adds support for SQL Server date and time types.
    * `chrono` and `stable-local`: If your local timezone does not observe daylight saving time (DST), enable this feature to use a fixed offset for `DateTime<Local>`, improving formatting performance.
* `rust_decimal` or `bigdecimal`: Adds support for SQL Server decimal/numeric types.
* `num-bigint`: Adds support for SQL Server decimal/numeric types (only integers).
* `uuid`: Adds support for SQL Server UNIQUEIDENTIFIER type.
*/

#![allow(unexpected_cfgs)]
#![cfg_attr(docsrs_1_92, feature(doc_cfg))]

mod errors;
mod functions;
mod literals;
mod traits;
mod wrappers;

pub use errors::*;
pub use functions::*;
pub use traits::*;
pub use wrappers::*;
