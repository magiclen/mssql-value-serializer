use std::fmt::{self, Formatter, Write};

use chrono::prelude::*;

use super::{SqlLiteralError, SqlServerLiteral, validate_time_zone_offset, validate_year};
use crate::impl_dyn_wrapper;

// ----- Date & Time -----

#[inline]
fn validate_naive_date(date: &NaiveDate) -> Result<(), SqlLiteralError> {
    validate_year(date.year())
}

#[inline]
fn validate_naive_time(time: &NaiveTime) -> Result<(), SqlLiteralError> {
    if time.nanosecond() >= 1_000_000_000 {
        return Err(SqlLiteralError::LeapSecondUnsupported);
    }

    Ok(())
}

#[inline]
fn validate_naive_date_time(date_time: &NaiveDateTime) -> Result<(), SqlLiteralError> {
    validate_naive_date(&date_time.date())?;
    validate_naive_time(&date_time.time())
}

#[inline]
fn validate_date_time<Tz: TimeZone>(date_time: &DateTime<Tz>) -> Result<(), SqlLiteralError> {
    validate_time_zone_offset(date_time.offset().fix().local_minus_utc())?;
    let utc = date_time.naive_utc();

    validate_naive_time(&utc.time())?;
    validate_naive_date(&utc.date())?;
    validate_naive_date(&date_time.naive_local().date())
}

#[inline]
fn push_naive_date(naive_date: &NaiveDate, out: &mut impl Write) -> fmt::Result {
    write!(
        out,
        "{year:04}-{month:02}-{day:02}",
        year = naive_date.year(),
        month = naive_date.month(),
        day = naive_date.day()
    )
}

fn push_naive_time(naive_time: &NaiveTime, out: &mut impl Write) -> fmt::Result {
    write!(
        out,
        "{hour:02}:{minute:02}:{second:02}",
        hour = naive_time.hour(),
        minute = naive_time.minute(),
        second = naive_time.second(),
    )?;

    let mut nnnnnnn = (naive_time.nanosecond() / 100) * 100;

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

#[inline]
fn push_naive_date_time(naive_date_time: &NaiveDateTime, out: &mut impl Write) -> fmt::Result {
    let date = naive_date_time.date();
    let time = naive_date_time.time();

    push_naive_date(&date, out)?;
    out.write_char(' ')?;
    push_naive_time(&time, out)
}

fn push_time_zone(fixed_offset: &FixedOffset, out: &mut impl Write) -> fmt::Result {
    let seconds = fixed_offset.local_minus_utc();

    let (sign, abs_seconds) = if seconds >= 0 { ('+', seconds) } else { ('-', -seconds) };

    let hours = abs_seconds / 3600;
    let minutes = (abs_seconds % 3600) / 60;

    write!(out, "{sign}{hours:02}:{minutes:02}")
}

#[inline]
fn push_date_time_fixed_offset(
    date_time: &DateTime<FixedOffset>,
    out: &mut impl Write,
) -> fmt::Result {
    let ndt = date_time.naive_local();
    let time_zone = date_time.timezone();

    push_naive_date_time(&ndt, out)?;
    out.write_char(' ')?;
    push_time_zone(&time_zone, out)
}

#[inline]
fn push_date_time_utc(date_time: &DateTime<Utc>, out: &mut impl Write) -> fmt::Result {
    let ndt = date_time.naive_utc();

    push_naive_date_time(&ndt, out)?;
    out.write_str(" +00:00")
}

#[cfg(feature = "stable-local")]
fn push_date_time_local(date_time: &DateTime<Local>, out: &mut impl Write) -> fmt::Result {
    use std::sync::OnceLock;

    static TIME_ZONE_STRING: OnceLock<(i32, String)> = OnceLock::new();

    let fixed_offset = date_time.offset().fix();
    let seconds = fixed_offset.local_minus_utc();

    let (cached_seconds, time_zone_string) = TIME_ZONE_STRING.get_or_init(|| {
        let mut time_zone_string = String::with_capacity(7);

        time_zone_string.push(' ');
        push_time_zone(&fixed_offset, &mut time_zone_string).unwrap(); // should not panic because the output is a string

        (seconds, time_zone_string)
    });

    let ndt = date_time.naive_local();

    push_naive_date_time(&ndt, out)?;

    if *cached_seconds == seconds {
        out.write_str(time_zone_string)
    } else {
        out.write_char(' ')?;
        push_time_zone(&fixed_offset, out)
    }
}

#[cfg(not(feature = "stable-local"))]
#[inline]
fn push_date_time_local(date_time: &DateTime<Local>, out: &mut impl Write) -> fmt::Result {
    let date_time = date_time.fixed_offset();

    push_date_time_fixed_offset(&date_time, out)
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
impl_date_time_as_string!(NaiveDate, push_naive_date, validate_naive_date);
impl_date_time_as_string!(NaiveTime, push_naive_time, validate_naive_time);
impl_date_time_as_string!(NaiveDateTime, push_naive_date_time, validate_naive_date_time);
impl_date_time_as_string!(DateTime<FixedOffset>, push_date_time_fixed_offset, validate_date_time);
impl_date_time_as_string!(DateTime<Utc>, push_date_time_utc, validate_date_time);
impl_date_time_as_string!(DateTime<Local>, push_date_time_local, validate_date_time);
