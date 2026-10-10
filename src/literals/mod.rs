#[cfg(feature = "bigdecimal")]
mod bigdecimal;
#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "jiff")]
mod jiff;
#[cfg(feature = "num-bigint")]
mod num_bigint;
#[cfg(feature = "rust_decimal")]
mod rust_decimal;
#[cfg(feature = "time")]
mod time;
#[cfg(feature = "uuid")]
mod uuid;

use std::{
    fmt::{self, Display, Formatter, Write},
    rc::Rc,
    sync::Arc,
};

use crate::{
    SqlLiteralError, SqlServerLiteral, SqlServerLiteralDynWrapper, impl_dyn_wrapper,
    impl_dyn_wrapper_slice,
};

#[cfg(any(feature = "chrono", feature = "jiff", feature = "time"))]
#[inline]
fn validate_time_zone_offset(seconds: i32) -> Result<(), SqlLiteralError> {
    if seconds % 60 != 0 || !(-50_400..=50_400).contains(&seconds) {
        return Err(SqlLiteralError::InvalidTimeZoneOffset {
            seconds,
        });
    }

    Ok(())
}

// ----- Booleans -----

impl SqlServerLiteral for bool {
    #[inline]
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        out.push(if *self { '1' } else { '0' });

        Ok(())
    }

    #[inline]
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
        out.write_char(if *self { '1' } else { '0' })?;

        Ok(())
    }
}

impl_dyn_wrapper!(bool);

// ----- Numbers -----

const MAX_DECIMAL_INTEGER: u128 = 10u128.pow(38) - 1;

struct FloatBuffer {
    bytes: [u8; 512],
    len:   usize,
}

impl Write for FloatBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len.checked_add(s.len()).ok_or(fmt::Error)?;
        self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

fn push_float_literal(value: impl Display, sql_type: &str, out: &mut impl Write) -> fmt::Result {
    let mut buffer = FloatBuffer {
        bytes: [0; 512], len: 0
    };
    // A plain f32 or f64 value needs at most 327 bytes, including the sign.
    write!(buffer, "{value}").expect("float buffer is too small");
    let s = std::str::from_utf8(&buffer.bytes[..buffer.len]).unwrap();
    let unsigned = s.trim_start_matches('-');
    let (integer, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    let precision = (integer.trim_start_matches('0').len() + fraction.len()).max(1);

    if precision <= 38 {
        out.write_str(s)
    } else {
        out.write_str("CAST('")?;
        out.write_str(s)?;
        out.write_str("' AS ")?;
        out.write_str(sql_type)?;
        out.write_char(')')
    }
}

macro_rules! impl_int {
    ($($t:ty $(=> $magnitude:expr)?),* $(,)*) => {
        $(
            impl SqlServerLiteral for $t {
                #[inline]
                fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
                    $(if ($magnitude)(*self) > MAX_DECIMAL_INTEGER {
                        return Err(SqlLiteralError::NumericOutOfRange);
                    })?

                    write!(out, "{}", *self).unwrap();

                    Ok(())
                }

                #[inline]
                fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
                    $(if ($magnitude)(*self) > MAX_DECIMAL_INTEGER {
                        return Err(SqlLiteralError::NumericOutOfRange);
                    })?

                    write!(out, "{}", *self)?;

                    Ok(())
                }
            }

            impl_dyn_wrapper!($t);
        )*
    }
}
impl_int!(i8, i16, i32, i64, i128 => i128::unsigned_abs, isize, u8, u16, u32, u64, u128 => |value| value, usize);

macro_rules! impl_float {
    ($($t:ty => $sql_type:literal),* $(,)?) => {
        $(
            impl SqlServerLiteral for $t {
                #[inline]
                fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
                    if !self.is_finite() {
                        return Err(SqlLiteralError::FloatNotFinite);
                    }

                    if self.is_subnormal() {
                        return Err(SqlLiteralError::FloatSubnormal);
                    }

                    push_float_literal(*self, $sql_type, out).unwrap();

                    Ok(())
                }

                #[inline]
                fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
                    if !self.is_finite() {
                        return Err(SqlLiteralError::FloatNotFinite);
                    }

                    if self.is_subnormal() {
                        return Err(SqlLiteralError::FloatSubnormal);
                    }

                    push_float_literal(*self, $sql_type, out)?;

                    Ok(())
                }
            }

            impl_dyn_wrapper!($t);
        )*
    }
}
impl_float!(f32 => "real", f64 => "float(53)");

// ----- Strings -----

pub(crate) fn push_string_literal_char(ch: &char, out: &mut impl Write) -> fmt::Result {
    out.write_char('\'')?;

    if *ch == '\'' {
        out.write_char('\'')?;
    }

    out.write_char(*ch)?;

    out.write_char('\'')?;

    Ok(())
}

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
        push_nstring_literal_char(self, out)?;

        Ok(())
    }
}

impl_dyn_wrapper!(char);

pub(crate) fn push_string_literal(s: &str, out: &mut impl Write) -> fmt::Result {
    out.write_char('\'')?;
    push_escaped_string(s, out)?;
    out.write_char('\'')
}

fn push_nstring_literal(s: &str, out: &mut impl Write) -> fmt::Result {
    out.write_str("N'")?;
    push_escaped_string(s, out)?;
    out.write_char('\'')
}

#[inline]
fn push_escaped_string(s: &str, out: &mut impl Write) -> fmt::Result {
    let Some(first_quote) = s.find('\'') else {
        return out.write_str(s);
    };

    if first_quote < 16 && s[first_quote + 1..].find('\'').is_some_and(|distance| distance < 16) {
        for ch in s.chars() {
            if ch == '\'' {
                out.write_char('\'')?;
            }

            out.write_char(ch)?;
        }

        return Ok(());
    }

    let mut start = 0;

    for (index, _) in s.match_indices('\'') {
        out.write_str(&s[start..=index])?;
        out.write_char('\'')?;
        start = index + 1;
    }

    out.write_str(&s[start..])
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
                    push_nstring_literal(self, out)?;

                    Ok(())
                }
            }
        )*
    };
}
impl_string!(str, &str, String, Rc<String>, Arc<String>);
impl_dyn_wrapper_slice!(str);
impl_dyn_wrapper!(String, Rc<String>, Arc<String>);

// ----- Blob -----

fn push_hex_bytes(bytes: &[u8], out: &mut impl Write) -> fmt::Result {
    out.write_str("0x")?;

    for b in bytes {
        write!(out, "{:02X}", b)?;
    }

    Ok(())
}

fn push_hex_bytes_to_string(bytes: &[u8], out: &mut String) -> fmt::Result {
    let hex_len = bytes.len().checked_mul(2).expect("binary literal is too large");
    let append_len = hex_len.checked_add(2).expect("binary literal is too large");

    out.reserve(append_len);
    out.push_str("0x");

    if bytes.is_empty() {
        return Ok(());
    }

    if bytes.len() >= 65_536 {
        push_large_hex_bytes_to_string(bytes, hex_len, out);
        return Ok(());
    }

    let len = out.len();
    // SAFETY: The new bytes are ASCII before encoding, and the encoder only writes ASCII into the tail.
    unsafe {
        let v = out.as_mut_vec();

        v.resize(len + hex_len, b'0');

        const_hex::encode_to_slice_upper(bytes, &mut v[len..]).unwrap(); // should not panic because we have calculated the length
    }

    Ok(())
}

// Keep the large stack buffer off the small-input path.
#[inline(never)]
fn push_large_hex_bytes_to_string(bytes: &[u8], hex_len: usize, out: &mut String) {
    let len = out.len();
    let mut buffer = [0u8; 16_384];

    // SAFETY: The encoder writes ASCII into initialized storage, and the copies initialize the full tail before set_len.
    unsafe {
        let v = out.as_mut_vec();
        let spare = &mut v.spare_capacity_mut()[..hex_len];

        for (input, destination) in
            bytes.chunks(buffer.len() / 2).zip(spare.chunks_mut(buffer.len()))
        {
            let encoded = &mut buffer[..destination.len()];
            const_hex::encode_to_slice_upper(input, encoded).expect("hex buffer length is correct");
            destination
                .as_mut_ptr()
                .cast::<u8>()
                .copy_from_nonoverlapping(encoded.as_ptr(), encoded.len());
        }

        v.set_len(len + hex_len);
    }
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
                    push_hex_bytes(self, out)?;

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
                out.write_str("NULL")?;

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
