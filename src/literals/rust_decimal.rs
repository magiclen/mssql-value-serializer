use std::fmt::{Formatter, Write};

use rust_decimal::Decimal;

use super::{SqlLiteralError, SqlServerLiteral};
use crate::impl_dyn_wrapper;

impl SqlServerLiteral for Decimal {
    #[inline]
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        write!(out, "{}", self).unwrap();

        Ok(())
    }

    #[inline]
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
        write!(out, "{}", self).unwrap();

        Ok(())
    }
}

impl_dyn_wrapper!(Decimal);
