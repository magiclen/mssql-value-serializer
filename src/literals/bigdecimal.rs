use std::fmt::{Formatter, Write};

use bigdecimal::{BigDecimal, BigDecimalRef};

use super::{SqlLiteralError, SqlServerLiteral};
use crate::{SqlServerLiteralDynWrapper, impl_dyn_wrapper};

impl SqlServerLiteral for BigDecimal {
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

impl_dyn_wrapper!(BigDecimal);

impl SqlServerLiteral for BigDecimalRef<'_> {
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

impl From<BigDecimalRef<'static>> for SqlServerLiteralDynWrapper<'static> {
    #[inline]
    fn from(value: BigDecimalRef<'static>) -> Self {
        Self::Owned(Box::new(value))
    }
}

impl<'a> From<&'a BigDecimalRef<'a>> for SqlServerLiteralDynWrapper<'a> {
    #[inline]
    fn from(value: &'a BigDecimalRef<'a>) -> Self {
        Self::Borrowed(value)
    }
}
