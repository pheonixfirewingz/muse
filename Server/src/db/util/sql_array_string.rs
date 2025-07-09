use std::ops::{Deref, DerefMut};
use arrayvec::ArrayString;
use sqlx::{Decode, Type};
use sqlx::error::BoxDynError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlArrayString<const N: usize>(ArrayString<N>);

// Implement TryFrom for our wrapper type (this is allowed!)
impl<const N: usize> TryFrom<String> for SqlArrayString<N> {
    type Error = arrayvec::CapacityError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(SqlArrayString(ArrayString::from(s.as_str()).map_err(|_| arrayvec::CapacityError::new(()))?))
    }
}

impl<const N: usize> SqlArrayString<N> {

    #[allow(unused)]
    pub fn new() -> Self {
        Self(ArrayString::new())
    }

    #[allow(unused)]
    
    pub fn from_str(s: &str) -> Result<Self, arrayvec::CapacityError> {
        Ok(Self(ArrayString::from(s).map_err(|_| arrayvec::CapacityError::new(()))?))
    }

    #[allow(unused)]
    pub fn into_inner(self) -> ArrayString<N> {
        self.0
    }

    #[allow(unused)]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<const N: usize> Deref for SqlArrayString<N> {
    type Target = ArrayString<N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for SqlArrayString<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> From<ArrayString<N>> for SqlArrayString<N> {
    fn from(arr: ArrayString<N>) -> Self {
        Self(arr)
    }
}

impl<const N: usize> From<&str> for SqlArrayString<N> {
    fn from(s: &str) -> Self {
        Self(ArrayString::from(s).expect("String too long for ArrayString"))
    }
}

// SQLx implementations for SQLite
impl<const N: usize> Type<sqlx::Sqlite> for SqlArrayString<N> {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'r, const N: usize> Decode<'r, sqlx::Sqlite> for SqlArrayString<N> {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let s: &str = <&str as Decode<sqlx::Sqlite>>::decode(value)?;
        let array_string = ArrayString::from(s)
            .map_err(|e| format!("String '{}' too long for ArrayString<{}>: {}", s, N, e))?;
        Ok(SqlArrayString(array_string))
    }
}