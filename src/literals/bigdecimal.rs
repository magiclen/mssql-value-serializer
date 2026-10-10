use std::fmt::Formatter;

use bigdecimal::{BigDecimal, BigDecimalRef};

use super::{SqlLiteralError, SqlServerLiteral};
use crate::{SqlServerLiteralDynWrapper, impl_dyn_wrapper};

#[inline]
fn validate_decimal(digits: u64, scale: i64) -> Result<(), SqlLiteralError> {
    let digits = i128::from(digits.max(1));
    let scale = i128::from(scale);
    let precision = if scale >= 0 { digits.max(scale) } else { digits - scale };

    if precision > 38 {
        return Err(SqlLiteralError::NumericOutOfRange);
    }

    Ok(())
}

impl SqlServerLiteral for BigDecimal {
    #[inline]
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        validate_decimal(self.decimal_digit_count(), self.fractional_digit_count())?;
        self.write_plain_string(out).unwrap();

        Ok(())
    }

    #[inline]
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
        validate_decimal(self.decimal_digit_count(), self.fractional_digit_count())?;
        self.write_plain_string(out)?;

        Ok(())
    }
}

impl_dyn_wrapper!(BigDecimal);

impl SqlServerLiteral for BigDecimalRef<'_> {
    #[inline]
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        validate_decimal(self.count_digits(), self.fractional_digit_count())?;
        self.to_owned().write_plain_string(out).unwrap();

        Ok(())
    }

    #[inline]
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
        validate_decimal(self.count_digits(), self.fractional_digit_count())?;
        self.to_owned().write_plain_string(out)?;

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
