use std::{
    borrow::Cow,
    fmt::{self, Debug, Display, Formatter},
    ops::Deref,
};

use crate::{SqlLiteralError, SqlServerLiteral, SqlServerLiteralForValueList};

struct SqlServerLiteralWrapperDebugFormatter<'a, T: SqlServerLiteral + ?Sized>(&'a T);

impl<'a, T: SqlServerLiteral + ?Sized> Debug for SqlServerLiteralWrapperDebugFormatter<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.append_sql_literal_fmt(f).map_err(|_| fmt::Error)
    }
}

/// A wrapper type for any value implementing [`SqlServerLiteral`].
///
/// This type allows you to easily convert a value into one that implements both [`Display`] and [`serde::Serialize`], enabling SQL Server literal behavior.
///
/// # Examples
///
/// ```rust
/// use mssql_value_serializer::SqlServerLiteralWrapper;
///
/// let needle = "Some text";
///
/// let mut sql = format!(
///     "
///         SELECT
///             *
///         FROM
///             [TABLE]
///         WHERE
///             name = {value}
///     ",
///     value = SqlServerLiteralWrapper::new(needle)
/// );
///
/// assert_eq!(
///     "
///         SELECT
///             *
///         FROM
///             [TABLE]
///         WHERE
///             name = N'Some text'
///     ",
///     sql
/// );
/// ```
pub struct SqlServerLiteralWrapper<T: SqlServerLiteral>(T);

impl<T: SqlServerLiteral> SqlServerLiteralWrapper<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Consumes the wrapper and returns the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: SqlServerLiteral> Debug for SqlServerLiteralWrapper<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_tuple("SqlServerLiteralWrapper");

        debug.field(&SqlServerLiteralWrapperDebugFormatter(self));

        debug.finish()
    }
}

impl<T: SqlServerLiteral> Display for SqlServerLiteralWrapper<T> {
    /// Formats the wrapped value as a SQL Server literal.
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.append_sql_literal_fmt(f).map_err(|_| fmt::Error)
    }
}

impl<T: SqlServerLiteral> From<T> for SqlServerLiteralWrapper<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: SqlServerLiteral> Deref for SqlServerLiteralWrapper<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: SqlServerLiteral> SqlServerLiteral for SqlServerLiteralWrapper<T> {
    #[inline]
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        self.0.append_sql_literal(out)
    }

    #[inline]
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
        self.0.append_sql_literal_fmt(out)
    }
}

#[cfg(feature = "serde")]
impl<T: SqlServerLiteral> serde::Serialize for SqlServerLiteralWrapper<T> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer, {
        let mut s = String::new();

        self.0
            .append_sql_literal(&mut s)
            .map_err(|error| serde::ser::Error::custom(error.to_string()))?;

        serializer.serialize_str(s.as_str())
    }
}

pub enum SqlServerLiteralDynWrapper<'a> {
    Borrowed(&'a dyn SqlServerLiteral),
    Owned(Box<dyn SqlServerLiteral>),
}

impl Debug for SqlServerLiteralDynWrapper<'_> {
    /// Formats the wrapped value as a SQL Server literal.
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (name, r) = match self {
            Self::Borrowed(v) => ("Borrowed", *v),
            Self::Owned(v) => ("Owned", v.as_ref()),
        };

        let mut debug = f.debug_tuple(name);

        debug.field(&SqlServerLiteralWrapperDebugFormatter(r));

        debug.finish()
    }
}

impl Display for SqlServerLiteralDynWrapper<'_> {
    /// Formats the wrapped value as a SQL Server literal.
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Borrowed(v) => v.append_sql_literal_fmt(f).map_err(|_| fmt::Error),
            Self::Owned(v) => v.append_sql_literal_fmt(f).map_err(|_| fmt::Error),
        }
    }
}

impl From<Box<dyn SqlServerLiteral>> for SqlServerLiteralDynWrapper<'_> {
    #[inline]
    fn from(value: Box<dyn SqlServerLiteral>) -> Self {
        Self::Owned(value)
    }
}

impl<'a> From<&'a dyn SqlServerLiteral> for SqlServerLiteralDynWrapper<'a> {
    #[inline]
    fn from(value: &'a dyn SqlServerLiteral) -> Self {
        Self::Borrowed(value)
    }
}

impl<'a, T: ?Sized + ToOwned + SqlServerLiteral> From<&'a Cow<'a, T>>
    for SqlServerLiteralDynWrapper<'a>
where
    T::Owned: SqlServerLiteral,
    &'a T: SqlServerLiteral,
{
    #[inline]
    fn from(value: &'a Cow<'a, T>) -> Self {
        match value {
            Cow::Owned(v) => Self::Borrowed(v),
            Cow::Borrowed(v) => Self::Borrowed(v),
        }
    }
}

macro_rules! impl_dyn_wrapper {
    ($ty:ty) => {
        impl From<$ty> for $crate::SqlServerLiteralDynWrapper<'_> {
            #[inline]
            fn from(value: $ty) -> Self {
                Self::Owned(Box::new(value))
            }
        }

        impl<'a> From<&'a $ty> for $crate::SqlServerLiteralDynWrapper<'a> {
            #[inline]
            fn from(value: &'a $ty) -> Self {
                Self::Borrowed(value)
            }
        }

        impl<'a> From<::std::borrow::Cow<'a, $ty>> for $crate::SqlServerLiteralDynWrapper<'a> {
            #[inline]
            fn from(value: ::std::borrow::Cow<'a, $ty>) -> Self {
                match value {
                    ::std::borrow::Cow::Owned(v) => Self::Owned(Box::new(v)),
                    ::std::borrow::Cow::Borrowed(v) => Self::Borrowed(v),
                }
            }
        }
    };
    ($($ty:ty),+ $(,)*) => {
        $(
            $crate::impl_dyn_wrapper!($ty);
        )+
    };
}

macro_rules! impl_dyn_wrapper_slice {
    ($ty:ty) => {
        impl<'a> From<&'static $ty> for $crate::SqlServerLiteralDynWrapper<'a> {
            #[inline]
            fn from(value: &'static $ty) -> Self {
                Self::Owned(Box::new(value))
            }
        }

        impl<'a> From<&'a &'a $ty> for $crate::SqlServerLiteralDynWrapper<'a> {
            #[inline]
            fn from(value: &'a &'a $ty) -> Self {
                Self::Borrowed(value)
            }
        }

        impl<'a> From<::std::borrow::Cow<'a, $ty>> for $crate::SqlServerLiteralDynWrapper<'a> {
            #[inline]
            fn from(value: ::std::borrow::Cow<'a, $ty>) -> Self {
                // match value {
                //     Cow::Owned(v) => Self::Owned(Box::new(v)),
                //     Cow::Borrowed(v) => Self::Borrowed(v as &'a dyn SqlServerLiteral), // We can't use trait object like this. I don't know why.
                // }

                let value = value.into_owned();

                Self::Owned(Box::new(value))
            }
        }
    };
    ($($ty:ty),+ $(,)*) => {
        $(
            $crate::impl_dyn_wrapper_slice!($ty);
        )+
    };
}

pub(crate) use impl_dyn_wrapper;
pub(crate) use impl_dyn_wrapper_slice;

impl<'a> Deref for SqlServerLiteralDynWrapper<'a> {
    type Target = dyn SqlServerLiteral + 'a;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(v) => v.as_ref(),
            Self::Borrowed(v) => *v,
        }
    }
}

impl<'a> SqlServerLiteral for SqlServerLiteralDynWrapper<'a> {
    #[inline]
    fn append_sql_literal(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        self.deref().append_sql_literal(out)
    }

    #[inline]
    fn append_sql_literal_fmt(&self, out: &mut Formatter<'_>) -> Result<(), SqlLiteralError> {
        self.deref().append_sql_literal_fmt(out)
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Serialize for SqlServerLiteralDynWrapper<'a> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer, {
        let mut s = String::new();

        self.deref()
            .append_sql_literal(&mut s)
            .map_err(|error| serde::ser::Error::custom(error.to_string()))?;

        serializer.serialize_str(s.as_str())
    }
}

struct SqlServerLiteralForValueListWrapperDebugFormatter<
    'a,
    T: SqlServerLiteralForValueList + ?Sized,
>(&'a T);

impl<'a, T: SqlServerLiteralForValueList + ?Sized> Debug
    for SqlServerLiteralForValueListWrapperDebugFormatter<'a, T>
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.append_sql_literal_for_value_list_fmt(f).map_err(|_| fmt::Error)
    }
}

/// A wrapper type for any value implementing [`SqlServerLiteralForValueList`].
///
/// This type allows you to easily convert a value into one that implements both [`Display`] and [`serde::Serialize`], enabling it to exhibit SQL Server literal value list behavior.
///
/// # Examples
///
/// ```rust
/// use mssql_value_serializer::{SqlServerLiteral, SqlServerLiteralForValueListWrapper};
///
/// let needles: &[&str] = &["Some text", "Foo", "Bar"];
///
/// let mut sql = format!(
///     "
///         SELECT
///             *
///         FROM
///             [TABLE]
///         WHERE
///             name IN ({value})
///     ",
///     value = SqlServerLiteralForValueListWrapper::new(needles)
/// );
///
/// assert_eq!(
///     "
///         SELECT
///             *
///         FROM
///             [TABLE]
///         WHERE
///             name IN (N'Some text', N'Foo', N'Bar')
///     ",
///     sql
/// );
pub struct SqlServerLiteralForValueListWrapper<T: SqlServerLiteralForValueList>(T);

impl<T: SqlServerLiteralForValueList> SqlServerLiteralForValueListWrapper<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Consumes the wrapper and returns the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: SqlServerLiteralForValueList> Debug for SqlServerLiteralForValueListWrapper<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_tuple("SqlServerLiteralForValueListWrapper");

        debug.field(&SqlServerLiteralForValueListWrapperDebugFormatter(self));

        debug.finish()
    }
}

impl<T: SqlServerLiteralForValueList> Display for SqlServerLiteralForValueListWrapper<T> {
    /// Formats the wrapped value as a SQL Server literal.
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.append_sql_literal_for_value_list_fmt(f).map_err(|_| fmt::Error)
    }
}

impl<T: SqlServerLiteralForValueList> From<T> for SqlServerLiteralForValueListWrapper<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: SqlServerLiteralForValueList> Deref for SqlServerLiteralForValueListWrapper<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: SqlServerLiteralForValueList> SqlServerLiteralForValueList
    for SqlServerLiteralForValueListWrapper<T>
{
    #[inline]
    fn append_sql_literal_for_value_list(&self, out: &mut String) -> Result<(), SqlLiteralError> {
        self.0.append_sql_literal_for_value_list(out)
    }

    #[inline]
    fn append_sql_literal_for_value_list_fmt(
        &self,
        out: &mut Formatter<'_>,
    ) -> Result<(), SqlLiteralError> {
        self.0.append_sql_literal_for_value_list_fmt(out)
    }
}

#[cfg(feature = "serde")]
impl<T: SqlServerLiteralForValueList> serde::Serialize for SqlServerLiteralForValueListWrapper<T> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer, {
        let mut s = String::new();

        self.0
            .append_sql_literal_for_value_list(&mut s)
            .map_err(|error| serde::ser::Error::custom(error.to_string()))?;

        serializer.serialize_str(s.as_str())
    }
}
