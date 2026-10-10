use std::fmt::{self, Write};

use chrono::prelude::*;

use super::{
    SqlLiteralError, impl_date_time_as_string, push_time, push_time_zone_offset,
    validate_time_zone_offset, validate_year,
};

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
    // Check the UTC date first because naive_local panics when the local date time is out of range.
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

#[inline]
fn push_naive_time(naive_time: &NaiveTime, out: &mut impl Write) -> fmt::Result {
    push_time(
        naive_time.hour(),
        naive_time.minute(),
        naive_time.second(),
        naive_time.nanosecond(),
        out,
    )
}

#[inline]
fn push_naive_date_time(naive_date_time: &NaiveDateTime, out: &mut impl Write) -> fmt::Result {
    let date = naive_date_time.date();
    let time = naive_date_time.time();

    push_naive_date(&date, out)?;
    out.write_char(' ')?;
    push_naive_time(&time, out)
}

#[inline]
fn push_date_time_fixed_offset(
    date_time: &DateTime<FixedOffset>,
    out: &mut impl Write,
) -> fmt::Result {
    let ndt = date_time.naive_local();

    push_naive_date_time(&ndt, out)?;
    out.write_char(' ')?;
    push_time_zone_offset(date_time.offset().local_minus_utc(), out)
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

    let seconds = date_time.offset().fix().local_minus_utc();

    let (cached_seconds, time_zone_string) = TIME_ZONE_STRING.get_or_init(|| {
        let mut time_zone_string = String::with_capacity(7);

        time_zone_string.push(' ');
        push_time_zone_offset(seconds, &mut time_zone_string).unwrap(); // should not panic because the output is a string

        (seconds, time_zone_string)
    });

    let ndt = date_time.naive_local();

    push_naive_date_time(&ndt, out)?;

    if *cached_seconds == seconds {
        out.write_str(time_zone_string)
    } else {
        out.write_char(' ')?;
        push_time_zone_offset(seconds, out)
    }
}

#[cfg(not(feature = "stable-local"))]
#[inline]
fn push_date_time_local(date_time: &DateTime<Local>, out: &mut impl Write) -> fmt::Result {
    let date_time = date_time.fixed_offset();

    push_date_time_fixed_offset(&date_time, out)
}

impl_date_time_as_string!(NaiveDate, push_naive_date, validate_naive_date);
impl_date_time_as_string!(NaiveTime, push_naive_time, validate_naive_time);
impl_date_time_as_string!(NaiveDateTime, push_naive_date_time, validate_naive_date_time);
impl_date_time_as_string!(DateTime<FixedOffset>, push_date_time_fixed_offset, validate_date_time);
impl_date_time_as_string!(DateTime<Utc>, push_date_time_utc, validate_date_time);
impl_date_time_as_string!(DateTime<Local>, push_date_time_local, validate_date_time);
