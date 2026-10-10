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
    fmt::{self, Display, Formatter, UpperExp, Write},
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

#[cfg(any(feature = "chrono", feature = "jiff", feature = "time"))]
#[inline]
fn validate_year(year: i32) -> Result<(), SqlLiteralError> {
    if !(1..=9999).contains(&year) {
        return Err(SqlLiteralError::DateOutOfRange {
            year,
        });
    }

    Ok(())
}

#[cfg(any(feature = "chrono", feature = "jiff", feature = "time"))]
fn push_time(
    hour: u32,
    minute: u32,
    second: u32,
    nanosecond: u32,
    out: &mut impl Write,
) -> fmt::Result {
    write!(out, "{hour:02}:{minute:02}:{second:02}")?;

    let mut nnnnnnn = (nanosecond / 100) * 100;

    if nnnnnnn > 0 {
        out.write_char('.')?;

        let digits = nnnnnnn.ilog10() + 1;

        for _ in digits..9 {
            out.write_char('0')?;
        }

        // trim trailing zeros
        while nnnnnnn.is_multiple_of(10) {
            nnnnnnn /= 10;
        }

        write!(out, "{nnnnnnn}")?;
    }

    Ok(())
}

#[cfg(any(feature = "chrono", feature = "jiff", feature = "time"))]
fn push_time_zone_offset(seconds: i32, out: &mut impl Write) -> fmt::Result {
    let (sign, abs_seconds) = if seconds >= 0 { ('+', seconds) } else { ('-', -seconds) };

    let hours = abs_seconds / 3600;
    let minutes = (abs_seconds % 3600) / 60;

    write!(out, "{sign}{hours:02}:{minutes:02}")
}

#[cfg(any(feature = "chrono", feature = "jiff", feature = "time"))]
macro_rules! impl_date_time_as_string {
    ($ty:ty, $f:ident $(, $validate:ident)?) => {
        impl $crate::SqlServerLiteral for $ty {
            #[inline]
            fn append_sql_literal(&self, out: &mut String) -> Result<(), $crate::SqlLiteralError> {
                $($validate(self)?;)?

                out.push('\'');
                $f(self, out).unwrap();
                out.push('\'');

                Ok(())
            }

            #[inline]
            fn append_sql_literal_fmt(
                &self,
                out: &mut ::std::fmt::Formatter<'_>,
            ) -> Result<(), $crate::SqlLiteralError> {
                $($validate(self)?;)?

                ::std::fmt::Write::write_char(out, '\'')?;
                $f(self, out)?;
                ::std::fmt::Write::write_char(out, '\'')?;

                Ok(())
            }
        }

        $crate::impl_dyn_wrapper!($ty);
    };
}

#[cfg(any(feature = "chrono", feature = "jiff", feature = "time"))]
use impl_date_time_as_string;

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
    bytes: [u8; 32],
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

fn push_float_literal<T: Display + UpperExp>(
    value: T,
    abs: f64,
    cast_type: Option<&str>,
    out: &mut impl Write,
) -> fmt::Result {
    const ZEROS: &str = "0000000000000000000000000000000000000";

    // Rust prints at most 17 significant digits, so plain text in this range never needs more than 38 digits.
    if abs == 0.0 || (1e-21..1e21).contains(&abs) {
        return write!(out, "{value}");
    }

    let mut buffer = FloatBuffer {
        bytes: [0; 32], len: 0
    };
    // Scientific notation of an f32 or f64 value needs at most 24 bytes, including the sign.
    write!(buffer, "{value:E}").expect("float buffer is too small");
    let s = std::str::from_utf8(&buffer.bytes[..buffer.len]).unwrap();
    let (sign, unsigned) = s.split_at(usize::from(s.starts_with('-')));
    let (mantissa, exponent) = unsigned.split_once('E').unwrap();
    let (first, rest) = mantissa.split_at(1);
    let rest = rest.trim_start_matches('.');
    let exponent: i32 = exponent.parse().unwrap();
    let digits = 1 + rest.len() as i32;

    // Outside the range above, the exponent is at least 21 or negative, so plain text has either no fraction or no integer part.
    let zeros = if exponent > 0 { exponent + 1 - digits } else { -exponent - 1 };

    if digits + zeros <= 38 {
        out.write_str(sign)?;

        if exponent > 0 {
            out.write_str(first)?;
            out.write_str(rest)?;
            out.write_str(&ZEROS[..zeros as usize])
        } else {
            out.write_str("0.")?;
            out.write_str(&ZEROS[..zeros as usize])?;
            out.write_str(first)?;
            out.write_str(rest)
        }
    } else if let Some(cast_type) = cast_type {
        out.write_str("CAST(")?;
        out.write_str(s)?;
        out.write_str(" AS ")?;
        out.write_str(cast_type)?;
        out.write_char(')')
    } else {
        out.write_str(s)
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
    ($($t:ty => $cast_type:expr),* $(,)?) => {
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

                    push_float_literal(*self, f64::from(self.abs()), $cast_type, out).unwrap();

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

                    push_float_literal(*self, f64::from(self.abs()), $cast_type, out)?;

                    Ok(())
                }
            }

            impl_dyn_wrapper!($t);
        )*
    }
}
impl_float!(f32 => Some("real"), f64 => None);

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

pub(crate) fn push_string_literal<const MAX_CHAR_LEN: usize>(
    s: &str,
    out: &mut impl Write,
) -> fmt::Result {
    out.write_char('\'')?;
    push_escaped_string::<MAX_CHAR_LEN>(s, out)?;
    out.write_char('\'')
}

fn push_nstring_literal<const MAX_CHAR_LEN: usize>(s: &str, out: &mut impl Write) -> fmt::Result {
    out.write_str("N'")?;
    push_escaped_string::<MAX_CHAR_LEN>(s, out)?;
    out.write_char('\'')
}

#[inline]
fn push_escaped_string<const MAX_CHAR_LEN: usize>(s: &str, out: &mut impl Write) -> fmt::Result {
    let Some(first_quote) = s.find('\'') else {
        return out.write_str(s);
    };

    let quote_gap = if s.len() <= 64 { 16 } else { 10 };

    // String can use char writes for long dense strings because its write calls are cheaper.
    if s.len() <= MAX_CHAR_LEN
        && first_quote < quote_gap
        && s.as_bytes()[first_quote + 1..].iter().take(quote_gap).position(|&byte| byte == b'\'').is_some_and(|distance| {
            // Long inputs need short char gaps even when their UTF-8 byte gaps are larger.
            // An ASCII quote is always at a UTF-8 boundary.
            s.len() <= 64 || s[first_quote + 1..first_quote + 1 + distance].chars().count() < 8
        })
        // Sample the middle and end to avoid choosing char writes from the prefix alone.
        && (s.len() <= 64
            || (s.as_bytes()[s.len() / 2..s.len() / 2 + quote_gap].contains(&b'\'')
                && s.as_bytes()[s.len() - quote_gap..].contains(&b'\'')))
    {
        if s.len() > 64 {
            return push_long_escaped_chars(s, out);
        }

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

// Keep the long char loop separate from string search code.
#[inline(never)]
fn push_long_escaped_chars(s: &str, out: &mut impl Write) -> fmt::Result {
    for ch in s.chars() {
        if ch == '\'' {
            out.write_char('\'')?;
        }

        out.write_char(ch)?;
    }

    Ok(())
}

macro_rules! impl_string {
    ($($t:ty),* $(,)*) => {
        $(
            impl SqlServerLiteral for $t {
                #[inline]
                fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
                    push_nstring_literal::<{ usize::MAX }>(self, out).unwrap();

                    Ok(())
                }

                #[inline]
                fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
                    push_nstring_literal::<64>(self, out)?;

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

    push_hex_chunks::<512>(bytes, out)
}

#[inline]
fn push_hex_chunks<const N: usize>(bytes: &[u8], out: &mut impl Write) -> fmt::Result {
    let mut buffer = [0u8; N];

    for chunk in bytes.chunks(N / 2) {
        let hex = const_hex::encode_to_str_upper(chunk, &mut buffer[..chunk.len() * 2])
            .expect("hex buffer length is correct");

        out.write_str(hex)?;
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
        push_large_hex_bytes_to_string(bytes, out);
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
fn push_large_hex_bytes_to_string(bytes: &[u8], out: &mut String) {
    push_hex_chunks::<16_384>(bytes, out).unwrap(); // should not panic because the output is a string
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
