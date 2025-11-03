use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug)]
pub enum SqlLiteralError {
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

#[derive(Debug)]
pub struct AppendSQLLiteralInValueListError {
    pub index: usize,
    pub error: SqlLiteralError,
}

impl Display for AppendSQLLiteralInValueListError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("index = {}, ", self.index))?;

        Display::fmt(&self.error, f)
    }
}

impl Error for AppendSQLLiteralInValueListError {}
