use std::{
    borrow::Cow,
    fmt::{self, Debug, Display, Formatter},
    ops::Deref,
};

use crate::{SqlLiteralError, SqlServerLiteral};

struct SqlServerLiteralWrapperDebugFormatter<'a, T: SqlServerLiteral + ?Sized>(&'a T);

impl<'a, T: SqlServerLiteral + ?Sized> Debug for SqlServerLiteralWrapperDebugFormatter<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.append_sql_literal_fmt(f).map_err(|_| fmt::Error)
    }
}

/// A wrapper type for any value implementing [`SqlServerLiteral`].
///
/// This type allows you to easily convert a value into a type that implements [`Display`] nad [`serde::Serialize`], or can be stored as a trait object (`&dyn SqlServerLiteral`) while preserving its SQL Server literal behavior.
pub struct SqlServerLiteralWrapper<T: SqlServerLiteral>(T);

impl<T: SqlServerLiteral> SqlServerLiteralWrapper<T> {
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

impl<T: SqlServerLiteral> SqlServerLiteralWrapper<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(value)
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
