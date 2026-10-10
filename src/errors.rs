use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

/// Errors that occur when serializing a SQL Server literal.
#[derive(Debug)]
#[non_exhaustive]
pub enum SqlLiteralError {
    /// The floating-point value is not finite (`NaN` or `infinity`).
    FloatNotFinite,
    /// The output writer failed.
    Write(fmt::Error),
    /// SQL Server cannot store a subnormal floating-point value.
    FloatSubnormal,
    /// The value needs more than 38 decimal digits or an unsupported scale.
    NumericOutOfRange,
    /// The time zone offset is not a whole minute or is outside -14:00 to +14:00.
    InvalidTimeZoneOffset { seconds: i32 },
    /// SQL Server does not support leap seconds.
    LeapSecondUnsupported,
    /// The year is outside 1 to 9999.
    DateOutOfRange { year: i32 },
}

impl Display for SqlLiteralError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FloatNotFinite => f.write_str("float value is not finite"),
            Self::Write(_) => f.write_str("output writer failed"),
            Self::FloatSubnormal => {
                f.write_str("SQL Server does not support subnormal float values")
            },
            Self::NumericOutOfRange => {
                f.write_str("numeric value is outside the SQL Server decimal range")
            },
            Self::InvalidTimeZoneOffset {
                seconds,
            } => write!(f, "time zone offset of {seconds} seconds is not supported by SQL Server"),
            Self::LeapSecondUnsupported => f.write_str("SQL Server does not support leap seconds"),
            Self::DateOutOfRange {
                year,
            } => write!(f, "year {year} is outside the SQL Server date range of 1 to 9999"),
        }
    }
}

impl From<fmt::Error> for SqlLiteralError {
    #[inline]
    fn from(error: fmt::Error) -> Self {
        Self::Write(error)
    }
}

impl Error for SqlLiteralError {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Write(error) => Some(error),
            _ => None,
        }
    }
}

#[inline]
pub(crate) fn into_fmt_result(result: Result<(), SqlLiteralError>) -> fmt::Result {
    match result {
        Ok(()) => Ok(()),
        Err(SqlLiteralError::Write(error)) => Err(error),
        Err(error) => panic!("cannot format SQL Server literal: {error}"),
    }
}

#[inline]
pub(crate) fn into_debug_fmt_result(
    result: Result<(), SqlLiteralError>,
    f: &mut Formatter<'_>,
) -> fmt::Result {
    match result {
        Ok(()) => Ok(()),
        Err(SqlLiteralError::Write(error)) => Err(error),
        // Debug output is often used in logs and panic messages, so it should not panic.
        Err(error) => write!(f, "<invalid: {error}>"),
    }
}

/// Errors that occur when serializing a SQL Server literal for a value list.
#[derive(Debug)]
pub struct SqlLiteralErrorWithIndex {
    pub index: usize,
    pub error: SqlLiteralError,
}

impl Display for SqlLiteralErrorWithIndex {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("index = {}, ", self.index))?;

        Display::fmt(&self.error, f)
    }
}

impl Error for SqlLiteralErrorWithIndex {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}
