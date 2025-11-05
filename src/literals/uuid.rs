use std::fmt::{Formatter, Write};

use uuid::Uuid;

use super::{SqlLiteralError, SqlServerLiteral};
use crate::impl_dyn_wrapper;

impl SqlServerLiteral for Uuid {
    #[inline]
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        write!(out, "{:X}", self).unwrap();

        Ok(())
    }

    #[inline]
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
        write!(out, "{:X}", self).unwrap();

        Ok(())
    }
}

impl_dyn_wrapper!(Uuid);
