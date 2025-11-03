use std::fmt::{self, Formatter, Write};

use chrono::prelude::*;

use super::{SqlLiteralError, SqlServerLiteral};
use crate::impl_dyn_wrapper;

// ----- Date & Time -----

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
        while nnnnnnn % 10 == 0 {
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

    // seconds should be zero for SQL Server (or ignore it in release)
    debug_assert!(
        abs_seconds % 60 == 0,
        "the seconds part of {fixed_offset:?} should be zero for SQL Server"
    );

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
    use std::sync::Once;

    static INIT: Once = Once::new();
    static mut TIME_ZONE_STRING: String = String::new();

    INIT.call_once(|| {
        println!("call once");
        let fixed_offset = date_time.offset().fix();

        #[allow(static_mut_refs)]
        unsafe {
            TIME_ZONE_STRING.reserve(7);

            TIME_ZONE_STRING.push(' ');
            push_time_zone(&fixed_offset, &mut TIME_ZONE_STRING).unwrap(); // should not panic because the output is a string
        }
    });

    let ndt = date_time.naive_local();

    push_naive_date_time(&ndt, out)?;

    #[allow(static_mut_refs)]
    out.write_str(unsafe { TIME_ZONE_STRING.as_str() })
}

#[cfg(not(feature = "stable-local"))]
#[inline]
fn push_date_time_local(date_time: &DateTime<Local>, out: &mut impl Write) -> fmt::Result {
    let date_time = date_time.fixed_offset();

    push_date_time_fixed_offset(&date_time, out)
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
impl_date_time_as_string!(NaiveDate, push_naive_date);
impl_date_time_as_string!(NaiveTime, push_naive_time);
impl_date_time_as_string!(NaiveDateTime, push_naive_date_time);
impl_date_time_as_string!(DateTime<FixedOffset>, push_date_time_fixed_offset);
impl_date_time_as_string!(DateTime<Utc>, push_date_time_utc);
impl_date_time_as_string!(DateTime<Local>, push_date_time_local);
