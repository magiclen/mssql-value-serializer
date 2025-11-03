use std::fmt::{self, Formatter, Write};

use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcDateTime, UtcOffset};

use super::{SqlLiteralError, SqlServerLiteral};
use crate::impl_dyn_wrapper;

// ----- Date & Time -----

#[inline]
fn push_naive_date(naive_date: &Date, out: &mut impl Write) -> fmt::Result {
    write!(
        out,
        "{year:04}-{month:02}-{day:02}",
        year = naive_date.year(),
        month = naive_date.month() as u8,
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

    let mut nnnnnnn = (naive_time.nanosecond() / 100) * 100;

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
fn push_naive_date_time(naive_date_time: &PrimitiveDateTime, out: &mut impl Write) -> fmt::Result {
    let date = naive_date_time.date();
    let time = naive_date_time.time();

    push_naive_date(&date, out)?;
    out.write_char(' ')?;
    push_naive_time(&time, out)
}

fn push_time_zone(fixed_offset: &UtcOffset, out: &mut impl Write) -> fmt::Result {
    let mut hours = fixed_offset.whole_hours();
    let mut minutes = fixed_offset.minutes_past_hour();

    let sign = if hours >= 0 {
        '+'
    } else {
        hours = -hours;
        minutes = -minutes;

        '-'
    };

    // seconds should be zero for SQL Server (or ignore it in release)
    debug_assert!(
        fixed_offset.seconds_past_minute() == 0,
        "the seconds part of {fixed_offset:?} should be zero for SQL Server"
    );

    write!(out, "{sign}{hours:02}:{minutes:02}")
}

#[inline]
fn push_date_time_utc(naive_date_time: &UtcDateTime, out: &mut impl Write) -> fmt::Result {
    let date = naive_date_time.date();
    let time = naive_date_time.time();

    push_naive_date(&date, out)?;
    out.write_char(' ')?;
    push_naive_time(&time, out)?;

    out.write_str(" +00:00")
}

#[inline]
fn push_date_time_fixed_offset(date_time: &OffsetDateTime, out: &mut impl Write) -> fmt::Result {
    let date = date_time.date();
    let time = date_time.time();
    let time_zone = date_time.offset();

    push_naive_date(&date, out)?;
    out.write_char(' ')?;
    push_naive_time(&time, out)?;

    out.write_char(' ')?;
    push_time_zone(&time_zone, out)
}

macro_rules! impl_date_time_as_string {
    ($ty:ty, $f:ident) => {
        impl SqlServerLiteral for $ty {
            #[inline]
            fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
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
                out.write_char('\'').unwrap();
                $f(self, out).unwrap();
                out.write_char('\'').unwrap();

                Ok(())
            }
        }

        impl_dyn_wrapper!($ty);
    };
}
impl_date_time_as_string!(Date, push_naive_date);
impl_date_time_as_string!(Time, push_naive_time);
impl_date_time_as_string!(PrimitiveDateTime, push_naive_date_time);
impl_date_time_as_string!(OffsetDateTime, push_date_time_fixed_offset);
impl_date_time_as_string!(UtcDateTime, push_date_time_utc);
