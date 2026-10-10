use std::fmt::{self, Formatter, Write};

use jiff::{
    Timestamp, Zoned,
    civil::{Date, DateTime, Time},
    tz::Offset,
};

use super::{SqlLiteralError, SqlServerLiteral, validate_time_zone_offset, validate_year};
use crate::impl_dyn_wrapper;

// ----- Date & Time -----

#[inline]
fn validate_naive_date(naive_date: &Date) -> Result<(), SqlLiteralError> {
    validate_year(i32::from(naive_date.year()))
}

#[inline]
fn validate_naive_date_time(naive_date_time: &DateTime) -> Result<(), SqlLiteralError> {
    validate_naive_date(&naive_date_time.date())
}

#[inline]
fn validate_timestamp(timestamp: &Timestamp) -> Result<(), SqlLiteralError> {
    // Include the boundary second because as_second truncates negative fractional seconds toward zero.
    if timestamp.as_second() <= -62_135_596_800 {
        return validate_naive_date_time(&Offset::UTC.to_datetime(*timestamp));
    }

    Ok(())
}

#[inline]
fn validate_zoned(date_time: &Zoned) -> Result<(), SqlLiteralError> {
    validate_time_zone_offset(date_time.offset().seconds())?;
    validate_timestamp(&date_time.timestamp())?;
    validate_naive_date_time(&date_time.datetime())
}

#[inline]
fn push_naive_date(naive_date: &Date, out: &mut impl Write) -> fmt::Result {
    write!(
        out,
        "{year:04}-{month:02}-{day:02}",
        year = naive_date.year(),
        month = naive_date.month(),
        day = naive_date.day()
    )
}

fn push_naive_time(naive_time: &Time, out: &mut impl Write) -> fmt::Result {
    write!(
        out,
        "{hour:02}:{minute:02}:{second:02}",
        hour = naive_time.hour(),
        minute = naive_time.minute(),
        second = naive_time.second(),
    )?;

    let mut nnnnnnn = (naive_time.subsec_nanosecond() / 100) * 100;

    if nnnnnnn > 0 {
        out.write_char('.')?;

        let digits = nnnnnnn.ilog10() + 1;

        for _ in digits..9 {
            out.write_char('0')?;
        }

        // trim trailing zeros
        while nnnnnnn % 10 == 0 {
            nnnnnnn /= 10;
        }

        write!(out, "{nnnnnnn}")?;
    }

    Ok(())
}

#[inline]
fn push_naive_date_time(naive_date_time: &DateTime, out: &mut impl Write) -> fmt::Result {
    let date = naive_date_time.date();
    let time = naive_date_time.time();

    push_naive_date(&date, out)?;
    out.write_char(' ')?;
    push_naive_time(&time, out)
}

fn push_time_zone(fixed_offset: &Offset, out: &mut impl Write) -> fmt::Result {
    let seconds = fixed_offset.seconds();

    let (sign, abs_seconds) = if seconds >= 0 { ('+', seconds) } else { ('-', -seconds) };

    let hours = abs_seconds / 3600;
    let minutes = (abs_seconds % 3600) / 60;

    write!(out, "{sign}{hours:02}:{minutes:02}")
}

#[inline]
fn push_timestamp(timestamp: &Timestamp, out: &mut impl Write) -> fmt::Result {
    let naive_date_time = Offset::UTC.to_datetime(*timestamp);

    push_naive_date_time(&naive_date_time, out)?;
    out.write_str(" +00:00")
}

#[inline]
fn push_zoned(date_time: &Zoned, out: &mut impl Write) -> fmt::Result {
    let naive_date_time = date_time.datetime();
    let time_zone = date_time.offset();

    push_naive_date_time(&naive_date_time, out)?;
    out.write_char(' ')?;
    push_time_zone(&time_zone, out)
}

macro_rules! impl_date_time_as_string {
    ($ty:ty, $f:ident $(, $validate:ident)?) => {
        impl SqlServerLiteral for $ty {
            #[inline]
            fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
                $($validate(self)?;)?

                out.push('\'');
                $f(self, out).unwrap();
                out.push('\'');

                Ok(())
            }

            #[inline]
            fn append_sql_literal_fmt(
                &self,
                out: &mut Formatter<'_>,
            ) -> Result<(), SqlLiteralError> {
                $($validate(self)?;)?

                out.write_char('\'')?;
                $f(self, out)?;
                out.write_char('\'')?;

                Ok(())
            }
        }

        impl_dyn_wrapper!($ty);
    };
}
impl_date_time_as_string!(Date, push_naive_date, validate_naive_date);
impl_date_time_as_string!(Time, push_naive_time);
impl_date_time_as_string!(DateTime, push_naive_date_time, validate_naive_date_time);
impl_date_time_as_string!(Timestamp, push_timestamp, validate_timestamp);
impl_date_time_as_string!(Zoned, push_zoned, validate_zoned);
