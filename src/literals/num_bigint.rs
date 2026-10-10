use std::fmt::{Formatter, Write};

use num_bigint::{BigInt, BigUint};

use super::{SqlLiteralError, SqlServerLiteral};
use crate::impl_dyn_wrapper;

#[inline]
fn validate_integer(value: &BigUint) -> Result<(), SqlLiteralError> {
    match u128::try_from(value) {
        Ok(value) if value <= super::MAX_DECIMAL_INTEGER => Ok(()),
        _ => Err(SqlLiteralError::NumericOutOfRange),
    }
}

impl SqlServerLiteral for BigInt {
    #[inline]
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        validate_integer(self.magnitude())?;

        write!(out, "{}", self).unwrap();

        Ok(())
    }

    #[inline]
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
        validate_integer(self.magnitude())?;

        write!(out, "{}", self)?;

        Ok(())
    }
}

impl_dyn_wrapper!(BigInt);

impl SqlServerLiteral for BigUint {
    #[inline]
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        validate_integer(self)?;

        write!(out, "{}", self).unwrap();

        Ok(())
    }

    #[inline]
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
        validate_integer(self)?;

        write!(out, "{}", self)?;

        Ok(())
    }
}

impl_dyn_wrapper!(BigUint);
