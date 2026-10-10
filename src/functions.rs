use std::fmt::Formatter;

use crate::{SqlLiteralError, SqlLiteralErrorWithIndex, SqlServerLiteral};

/// Appends a comma-separated list of SQL Server literals to the provided output string.
///
/// # Example
///
/// ```rust
/// use mssql_value_serializer::append_sql_literal_for_value_list;
///
/// let mut sql = String::new();
///
/// append_sql_literal_for_value_list(&[&1u8, &2u16, &true], &mut sql).unwrap();
///
/// assert_eq!("1, 2, 1", sql);
/// ```
pub fn append_sql_literal_for_value_list(
    values: &[&dyn SqlServerLiteral],
    out: &mut String,
) -> Result<(), SqlLiteralErrorWithIndex> {
    let mut iter = values.iter().enumerate();

    if let Some((index, value)) = iter.next() {
        value.append_sql_literal(out).map_err(|error| SqlLiteralErrorWithIndex {
            index,
            error,
        })?;

        for (index, value) in iter {
            out.push_str(", ");

            value.append_sql_literal(out).map_err(|error| SqlLiteralErrorWithIndex {
                index,
                error,
            })?;
        }
    }

    Ok(())
}

#[inline]
pub(crate) fn append_sql_literal_for_value_list_from_iter<'a, T>(
    mut iter: impl Iterator<Item = &'a T>,
    out: &mut String,
) -> Result<(), SqlLiteralError>
where
    T: ?Sized + SqlServerLiteral + 'a, {
    if let Some(value) = iter.next() {
        value.append_sql_literal(out)?;

        for value in iter {
            out.push_str(", ");

            value.append_sql_literal(out)?;
        }
    }

    Ok(())
}

#[inline]
pub(crate) fn append_sql_literal_for_value_list_from_iter_fmt<'a, T>(
    mut iter: impl Iterator<Item = &'a T>,
    out: &mut Formatter<'_>,
) -> Result<(), SqlLiteralError>
where
    T: ?Sized + SqlServerLiteral + 'a, {
    if let Some(value) = iter.next() {
        value.append_sql_literal_fmt(out)?;

        for value in iter {
            out.write_str(", ")?;

            value.append_sql_literal_fmt(out)?;
        }
    }

    Ok(())
}
