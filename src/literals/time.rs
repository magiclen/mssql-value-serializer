use std::fmt::{self, Write};

use time::{Date, OffsetDateTime, PlainDateTime, Time, UtcDateTime};

use super::{
    SqlLiteralError, impl_date_time_as_string, push_time, push_time_zone_offset,
    validate_time_zone_offset, validate_year,
};

// ----- Date & Time -----

#[inline]
fn validate_naive_date(naive_date: &Date) -> Result<(), SqlLiteralError> {
    validate_year(naive_date.year())
}

#[inline]
fn validate_naive_date_time(naive_date_time: &PlainDateTime) -> Result<(), SqlLiteralError> {
    validate_year(naive_date_time.year())
}

#[inline]
fn validate_date_time_utc(date_time: &UtcDateTime) -> Result<(), SqlLiteralError> {
    validate_year(date_time.year())
}

#[inline]
fn validate_date_time_fixed_offset(date_time: &OffsetDateTime) -> Result<(), SqlLiteralError> {
    validate_time_zone_offset(date_time.offset().whole_seconds())?;
    validate_year(date_time.year())?;

    // With a valid local year, UTC conversion can only overflow into year 10000.
    let utc = date_time.checked_to_utc().ok_or(SqlLiteralError::DateOutOfRange {
        year: 10_000
    })?;

    validate_year(utc.year())
}

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

#[inline]
fn push_naive_time(naive_time: &Time, out: &mut impl Write) -> fmt::Result {
    push_time(
        naive_time.hour().into(),
        naive_time.minute().into(),
        naive_time.second().into(),
        naive_time.nanosecond(),
        out,
    )
}

#[inline]
fn push_naive_date_time(naive_date_time: &PlainDateTime, out: &mut impl Write) -> fmt::Result {
    let date = naive_date_time.date();
    let time = naive_date_time.time();

    push_naive_date(&date, out)?;
    out.write_char(' ')?;
    push_naive_time(&time, out)
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

    push_naive_date(&date, out)?;
    out.write_char(' ')?;
    push_naive_time(&time, out)?;

    out.write_char(' ')?;
    push_time_zone_offset(date_time.offset().whole_seconds(), out)
}

impl_date_time_as_string!(Date, push_naive_date, validate_naive_date);
impl_date_time_as_string!(Time, push_naive_time);
impl_date_time_as_string!(PlainDateTime, push_naive_date_time, validate_naive_date_time);
impl_date_time_as_string!(
    OffsetDateTime,
    push_date_time_fixed_offset,
    validate_date_time_fixed_offset
);
impl_date_time_as_string!(UtcDateTime, push_date_time_utc, validate_date_time_utc);
