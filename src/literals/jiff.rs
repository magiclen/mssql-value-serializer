use std::fmt::{self, Write};

use jiff::{
    Timestamp, Zoned,
    civil::{Date, DateTime, Time},
    tz::Offset,
};

use super::{
    SqlLiteralError, impl_date_time_as_string, push_time, push_time_zone_offset,
    validate_time_zone_offset, validate_year,
};

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

#[inline]
fn push_naive_time(naive_time: &Time, out: &mut impl Write) -> fmt::Result {
    // jiff returns signed integers, but these time parts are never negative.
    push_time(
        naive_time.hour() as u32,
        naive_time.minute() as u32,
        naive_time.second() as u32,
        naive_time.subsec_nanosecond() as u32,
        out,
    )
}

#[inline]
fn push_naive_date_time(naive_date_time: &DateTime, out: &mut impl Write) -> fmt::Result {
    let date = naive_date_time.date();
    let time = naive_date_time.time();

    push_naive_date(&date, out)?;
    out.write_char(' ')?;
    push_naive_time(&time, out)
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

    push_naive_date_time(&naive_date_time, out)?;
    out.write_char(' ')?;
    push_time_zone_offset(date_time.offset().seconds(), out)
}

impl_date_time_as_string!(Date, push_naive_date, validate_naive_date);
impl_date_time_as_string!(Time, push_naive_time);
impl_date_time_as_string!(DateTime, push_naive_date_time, validate_naive_date_time);
impl_date_time_as_string!(Timestamp, push_timestamp, validate_timestamp);
impl_date_time_as_string!(Zoned, push_zoned, validate_zoned);
