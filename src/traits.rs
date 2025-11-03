use std::{borrow::Cow, collections::HashSet, fmt::Formatter};

use crate::{
    append_sql_literal_for_value_list_from_box_iter,
    append_sql_literal_for_value_list_from_box_iter_fmt,
    append_sql_literal_for_value_list_from_dyn_iter,
    append_sql_literal_for_value_list_from_dyn_iter_fmt,
    append_sql_literal_for_value_list_from_iter, append_sql_literal_for_value_list_from_iter_fmt,
    SqlLiteralError,
};

/// Represents a type that can be converted into a valid SQL Server literal.
///
/// # Example
///
/// ```rust
/// use std::{collections::HashSet, hash::Hash};
///
/// use mssql_value_serializer::SqlServerLiteral;
///
/// let value = "123456";
///
/// let mut sql = String::new();
///
/// value.append_sql_literal(&mut sql).unwrap();
///
/// assert_eq!("N'123456'", sql);
/// ```
pub trait SqlServerLiteral {
    /// Appends the SQL literal representation of this value to the given string.
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError>;

    /// Appends the SQL literal representation of this value to the given writer.
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError>;
}

impl<T: ?Sized + ToOwned + SqlServerLiteral> SqlServerLiteral for Cow<'_, T> {
    #[inline]
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        self.as_ref().append_sql_literal(out)
    }

    #[inline]
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
        self.as_ref().append_sql_literal_fmt(out)
    }
}

/// Represents a type that can serialize a collection of SQL Server literal values.
///
/// # Example
///
/// ```rust
/// use std::{collections::HashSet, hash::Hash};
///
/// use mssql_value_serializer::SqlServerLiteralForValueList;
///
/// let mut values = HashSet::new();
///
/// values.insert(1u8);
/// values.insert(2u8);
/// values.insert(1u8);
///
/// let mut sql = String::new();
///
/// values.append_sql_literal_for_value_list(&mut sql).unwrap();
///
/// if sql != "1, 2" && sql != "2, 1" {
///     panic!("expect \"1, 2\" or \"2, 1\"");
/// }
/// ```
pub trait SqlServerLiteralForValueList {
    /// Appends a comma-separated list of SQL Server literals representing the elements of this collection to the provided output string.
    fn append_sql_literal_for_value_list(&self, out: &mut String) -> Result<(), SqlLiteralError>;

    /// Appends a comma-separated list of SQL Server literals representing the elements of this collection to the provided output writer.
    fn append_sql_literal_for_value_list_fmt(
        &self,
        out: &mut Formatter<'_>,
    ) -> Result<(), SqlLiteralError>;
}

impl<T: ?Sized + ToOwned + SqlServerLiteralForValueList> SqlServerLiteralForValueList
    for Cow<'_, T>
{
    #[inline]
    fn append_sql_literal_for_value_list(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        self.as_ref().append_sql_literal_for_value_list(out)
    }

    #[inline]
    fn append_sql_literal_for_value_list_fmt(
        &self,
        out: &mut Formatter<'_>,
    ) -> Result<(), SqlLiteralError> {
        self.as_ref().append_sql_literal_for_value_list_fmt(out)
    }
}

impl<T: SqlServerLiteral> SqlServerLiteralForValueList for Vec<T> {
    #[inline]
    fn append_sql_literal_for_value_list(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        append_sql_literal_for_value_list_from_iter(self.iter(), out)
    }

    #[inline]
    fn append_sql_literal_for_value_list_fmt(
        &self,
        out: &mut Formatter<'_>,
    ) -> Result<(), SqlLiteralError> {
        append_sql_literal_for_value_list_from_iter_fmt(self.iter(), out)
    }
}

impl<T: SqlServerLiteral> SqlServerLiteralForValueList for HashSet<T> {
    #[inline]
    fn append_sql_literal_for_value_list(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        append_sql_literal_for_value_list_from_iter(self.iter(), out)
    }

    #[inline]
    fn append_sql_literal_for_value_list_fmt(
        &self,
        out: &mut Formatter<'_>,
    ) -> Result<(), SqlLiteralError> {
        append_sql_literal_for_value_list_from_iter_fmt(self.iter(), out)
    }
}

impl SqlServerLiteralForValueList for Vec<Box<dyn SqlServerLiteral>> {
    #[inline]
    fn append_sql_literal_for_value_list(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        append_sql_literal_for_value_list_from_box_iter(self.iter(), out)
    }

    #[inline]
    fn append_sql_literal_for_value_list_fmt(
        &self,
        out: &mut Formatter<'_>,
    ) -> Result<(), SqlLiteralError> {
        append_sql_literal_for_value_list_from_box_iter_fmt(self.iter(), out)
    }
}

impl SqlServerLiteralForValueList for Vec<&dyn SqlServerLiteral> {
    #[inline]
    fn append_sql_literal_for_value_list(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        append_sql_literal_for_value_list_from_dyn_iter(self.iter(), out)
    }

    #[inline]
    fn append_sql_literal_for_value_list_fmt(
        &self,
        out: &mut Formatter<'_>,
    ) -> Result<(), SqlLiteralError> {
        append_sql_literal_for_value_list_from_dyn_iter_fmt(self.iter(), out)
    }
}

impl SqlServerLiteralForValueList for &[&dyn SqlServerLiteral] {
    #[inline]
    fn append_sql_literal_for_value_list(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        append_sql_literal_for_value_list_from_dyn_iter(self.iter(), out)
    }

    #[inline]
    fn append_sql_literal_for_value_list_fmt(
        &self,
        out: &mut Formatter<'_>,
    ) -> Result<(), SqlLiteralError> {
        append_sql_literal_for_value_list_from_dyn_iter_fmt(self.iter(), out)
    }
}
