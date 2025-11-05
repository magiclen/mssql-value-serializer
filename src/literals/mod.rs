#[cfg(feature = "bigdecimal")]
mod bigdecimal;
#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "num-bigint")]
mod num_bigint;
#[cfg(feature = "rust_decimal")]
mod rust_decimal;
#[cfg(feature = "time")]
mod time;
#[cfg(feature = "uuid")]
mod uuid;

use std::fmt::{self, Formatter, Write};

use crate::{
    impl_dyn_wrapper, impl_dyn_wrapper_slice, SqlLiteralError, SqlServerLiteral,
    SqlServerLiteralDynWrapper,
};

// ----- Booleans -----

impl SqlServerLiteral for bool {
    #[inline]
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        out.push(if *self { '1' } else { '0' });

        Ok(())
    }

    #[inline]
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
        out.write_char(if *self { '1' } else { '0' }).unwrap();

        Ok(())
    }
}

impl_dyn_wrapper!(bool);

// ----- Numbers -----

macro_rules! impl_int {
    ($($t:ty),* $(,)*) => {
        $(
            impl SqlServerLiteral for $t {
                #[inline]
                fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
                    write!(out, "{}", *self).unwrap();

                    Ok(())
                }

                #[inline]
                fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
                    write!(out, "{}", *self).unwrap();

                    Ok(())
                }
            }

            impl_dyn_wrapper!($t);
        )*
    }
}
impl_int!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

macro_rules! impl_float {
    ($($t:ty),* $(,)?) => {
        $(
            impl SqlServerLiteral for $t {
                #[inline]
                fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
                    if !self.is_finite() {
                        return Err(SqlLiteralError::FloatNotFinite);
                    }

                    write!(out, "{}", *self).unwrap();

                    Ok(())
                }

                #[inline]
                fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
                    if !self.is_finite() {
                        return Err(SqlLiteralError::FloatNotFinite);
                    }

                    write!(out, "{}", *self).unwrap();

                    Ok(())
                }
            }

            impl_dyn_wrapper!($t);
        )*
    }
}
impl_float!(f32, f64);

// ----- Strings -----

fn push_nstring_literal_char(ch: &char, out: &mut impl Write) -> fmt::Result {
    out.write_str("N'")?;

    if *ch == '\'' {
        out.write_char('\'')?;
    }

    out.write_char(*ch)?;

    out.write_char('\'')?;

    Ok(())
}

impl SqlServerLiteral for char {
    #[inline]
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        push_nstring_literal_char(self, out).unwrap();

        Ok(())
    }

    #[inline]
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
        push_nstring_literal_char(self, out).unwrap();

        Ok(())
    }
}

impl_dyn_wrapper!(char);

fn push_nstring_literal(s: &str, out: &mut impl Write) -> fmt::Result {
    out.write_str("N'")?;

    for ch in s.chars() {
        if ch == '\'' {
            out.write_char('\'')?;
        }

        out.write_char(ch)?;
    }

    out.write_char('\'')?;

    Ok(())
}

macro_rules! impl_string {
    ($($t:ty),* $(,)*) => {
        $(
            impl SqlServerLiteral for $t {
                #[inline]
                fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
                    push_nstring_literal(self, out).unwrap();

                    Ok(())
                }

                #[inline]
                fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
                    push_nstring_literal(self, out).unwrap();

                    Ok(())
                }
            }
        )*
    };
}
impl_string!(str, &str, String);
impl_dyn_wrapper_slice!(str);
impl_dyn_wrapper!(String);

// ----- Blob -----

fn push_hex_bytes(bytes: &[u8], out: &mut impl Write) -> fmt::Result {
    if bytes.is_empty() {
        return Err(fmt::Error);
    }

    out.write_str("0x")?;

    for b in bytes {
        write!(out, "{:02X}", b)?;
    }

    Ok(())
}

fn push_hex_bytes_to_string(bytes: &[u8], out: &mut String) -> fmt::Result {
    if bytes.is_empty() {
        return Err(fmt::Error);
    }

    out.push_str("0x");

    let len = out.len();
    let hex_len = bytes.len() * 2;

    out.reserve(hex_len);

    unsafe {
        let v = out.as_mut_vec();

        v.set_len(len + hex_len);

        const_hex::encode_to_slice_upper(bytes, &mut v[len..]).unwrap(); // should not panic because we have calculated the length
    }

    Ok(())
}

macro_rules! impl_u8_array {
    ($($t:ty),* $(,)*) => {
        $(
            impl SqlServerLiteral for $t {
                #[inline]
                fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
                    push_hex_bytes_to_string(self, out).unwrap();

                    Ok(())
                }

                #[inline]
                fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
                    push_hex_bytes(self, out).unwrap();

                    Ok(())
                }
            }
        )*
    };
}
impl_u8_array!([u8], &[u8], Vec<u8>);
impl_dyn_wrapper_slice!([u8]);
impl_dyn_wrapper!(Vec<u8>);

// ----- NULL -----

impl<T: SqlServerLiteral> SqlServerLiteral for Option<T> {
    #[inline]
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        match self {
            Some(v) => v.append_sql_literal(out),
            None => {
                out.push_str("NULL");

                Ok(())
            },
        }
    }

    #[inline]
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
        match self {
            Some(v) => v.append_sql_literal_fmt(out),
            None => {
                out.write_str("NULL").unwrap();

                Ok(())
            },
        }
    }
}

impl<T: SqlServerLiteral + 'static> From<Option<T>> for SqlServerLiteralDynWrapper<'_> {
    #[inline]
    fn from(value: Option<T>) -> Self {
        Self::Owned(Box::new(value))
    }
}

impl<'a, T: ?Sized> From<&'a Option<&'a T>> for SqlServerLiteralDynWrapper<'a>
where
    &'a T: SqlServerLiteral,
{
    #[inline]
    fn from(value: &'a Option<&'a T>) -> Self {
        Self::Borrowed(value)
    }
}
