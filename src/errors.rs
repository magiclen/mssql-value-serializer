use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

/// Errors that occur when serializing a SQL Server literal.
#[derive(Debug)]
pub enum SqlLiteralError {
    /// The floating-point value is not finite (`NaN` or `infinity`).
    FloatNotFinite,
}

impl Display for SqlLiteralError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FloatNotFinite => f.write_str("float value is not finite"),
        }
    }
}

impl Error for SqlLiteralError {}

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

impl Error for SqlLiteralErrorWithIndex {}
