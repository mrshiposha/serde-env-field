//! This crate provides the [`EnvField<T>`] type capable of deserializing the `T` type
//! from a string with environment variables if the `T` implements the `FromStr` trait.
//!
//! During deserialization, the `EnvField` will try to deserialize the data as a string and expand all
//! the environment variables. After the expansion, the resulting string will be used
//! to construct the `T` type using the `FromStr` trait.
//!
//! If the supplied data was not a string, the `EnvField`
//! will attempt to deserialize the `T` type directly from the data.
//!
//! The `EnvField` works nicely with `Option`, `Vec`, and `#[serde(default)]`.
//!
//! #### Example
//!
//! ```
//! # use serde::{Serialize, Deserialize};
//! # use serde_env_field::EnvField;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Example {
//!     name: EnvField<String>,
//!     size: EnvField<usize>,
//!     num: EnvField<i32>,
//! }
//!
//! std::env::set_var("SIZE", "100");
//!
//! let de: Example = toml::from_str(r#"
//!     name = "${NAME:-Default Name}"
//!
//!     size = "$SIZE"
//!
//!     num = 42
//! "#).unwrap();
//!
//! assert_eq!(&de.name, "Default Name");
//! assert_eq!(de.size, 100);
//! assert_eq!(de.num, 42);
//! ```
//!
//! See the [`EnvField`] description for details.

use std::{
    fmt::{self, Debug},
    ops::*,
    str::FromStr,
};

use serde::{
    de::{self, Error},
    Deserialize, Serialize,
};
use serde_untagged::{de::Error as UntaggedError, UntaggedEnumVisitor};

pub use serde_env_field_wrap::env_field_wrap;

/// A field that deserializes either as `T` or as `String`
/// with all environment variables expanded via the [`shellexpand`] crate.
///
/// Requires `T` to implement the `FromStr` trait
/// for deserialization from String after environment variables expansion.
///
/// Works nicely with `Option`, `Vec`, and `#[serde(default)]`.
///
/// ### Examples
///
/// #### Basic
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::EnvField;
///
/// #[derive(Serialize, Deserialize)]
/// struct Example {
///     name: EnvField<String>,
///     size: EnvField<usize>,
///     num: EnvField<i32>,
/// }
///
/// std::env::set_var("SIZE", "100");
///
/// let de: Example = toml::from_str(r#"
///     name = "${NAME:-Default Name}"
///
///     size = "$SIZE"
///
///     num = 42
/// "#).unwrap();
///
/// assert_eq!(&de.name, "Default Name");
/// assert_eq!(de.size, 100);
/// assert_eq!(de.num, 42);
///
/// ```
///
/// #### Optional fields
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::EnvField;
///
/// #[derive(Serialize, Deserialize)]
/// struct Example {
///     required: EnvField<i32>,
///     optional: Option<EnvField<i32>>,
/// }
///
/// let de: Example = toml::from_str(r#"
///     required = 512
/// "#).unwrap();
///
/// assert_eq!(de.required, 512);
/// assert!(de.optional.is_none());
///
/// std::env::set_var("OPTIONAL", "-1024");
/// let de: Example = toml::from_str(r#"
///     required = 512
///     optional = "$OPTIONAL"
/// "#).unwrap();
///
/// assert_eq!(de.required, 512);
/// assert_eq!(de.optional.unwrap(), -1024);
///
/// let de: Example = toml::from_str(r#"
///     required = 512
///     optional = 42
/// "#).unwrap();
///
/// assert_eq!(de.required, 512);
/// assert_eq!(de.optional.unwrap(), 42);
///
/// ```
///
/// #### Sequences
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::EnvField;
///
/// #[derive(Serialize, Deserialize)]
/// struct Example {
///     seq: Vec<EnvField<i32>>,
/// }
///
/// std::env::set_var("NUM", "1000");
/// let de: Example = toml::from_str(r#"
///     seq = [
///         12, "$NUM", 145,
///     ]
/// "#).unwrap();
///
/// assert_eq!(de.seq[0], 12);
/// assert_eq!(de.seq[1], 1000);
/// assert_eq!(de.seq[2], 145);
///
/// ```
///
/// #### Defaults
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::EnvField;
/// use derive_more::FromStr;
///
/// #[derive(Serialize, Deserialize)]
/// struct Example {
///     #[serde(default)]
///     num: EnvField<NumWithDefault>,
/// }
///
/// #[derive(Serialize, Deserialize, FromStr)]
/// #[serde(transparent)]
/// struct NumWithDefault(i32);
/// impl Default for NumWithDefault {
///     fn default() -> Self {
///         Self(42)
///     }
/// }
///
/// let de: Example = toml::from_str("").unwrap();
/// assert_eq!(de.num.0, 42);
///
/// let de: Example = toml::from_str(r#"
///     num = 100
/// "#).unwrap();
/// assert_eq!(de.num.0, 100);
///
/// std::env::set_var("SOME_NUM", "555");
/// let de: Example = toml::from_str(r#"
///     num = "$SOME_NUM"
/// "#).unwrap();
/// assert_eq!(de.num.0, 555);
///
/// ```
///
/// #### Custom `FromStr`
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::EnvField;
/// # use std::str::FromStr;
/// # use std::num::ParseIntError;
///
/// #[derive(Serialize, Deserialize)]
/// struct Example {
///     inner: EnvField<Inner>,
/// }
///
/// #[derive(Serialize, Deserialize)]
/// struct Inner {
///     // We can use `EnvField` in inner structs
///     num: EnvField<i32>,
///
///     sym: EnvField<String>,
/// }
///
/// impl FromStr for Inner {
///     type Err = String;
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         let mut split = s.split(';');
///
///         let num = split
///             .next()
///             .unwrap()
///             .parse()
///             .map_err(|err: ParseIntError| err.to_string())?;
///
///         let sym = split
///             .next()
///             .unwrap()
///             .to_string()
///             .into();
///
///         Ok(Self {
///             num,
///             sym
///         })
///     }
/// }
///
/// std::env::set_var("INNER_NUM", "2048");
/// std::env::set_var("INNER_SYM", "Hi");
/// let de: Example = toml::from_str(r#"
///     inner = "$INNER_NUM;$INNER_SYM"
/// "#).unwrap();
///
/// assert_eq!(de.inner.num, 2048);
/// assert_eq!(&de.inner.sym, "Hi");
///
///
/// let de: Example = toml::from_str(r#"
///     [inner]
///     num = -500
///     sym = "Hello"
/// "#).unwrap();
///
/// assert_eq!(de.inner.num, -500);
/// assert_eq!(&de.inner.sym, "Hello");
///
/// ```
///
#[repr(transparent)]
#[derive(Serialize)]
#[serde(transparent)]
pub struct EnvField<T>(T);

impl<T> EnvField<T> {
    /// Unwraps the value, consuming the env field.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> EnvField<T>
where
    T: FromStr,
    <T as FromStr>::Err: fmt::Display,
{
    fn env_expand_and_parse(str_data: &str) -> Result<Self, UntaggedError> {
        match shellexpand::env(&str_data) {
            Ok(expanded) => expanded.parse().map(Self).map_err(Error::custom),
            Err(err) => Err(Error::custom(err)),
        }
    }
}

impl<T> From<T> for EnvField<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

macro_rules! deserialize_value {
    ($de:ident) => {
        |v| T::deserialize(de::value::$de::new(v)).map(Self)
    };
}

impl<'de, T> Deserialize<'de> for EnvField<T>
where
    T: Deserialize<'de> + FromStr,
    <T as FromStr>::Err: fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        UntaggedEnumVisitor::new()
            .string(Self::env_expand_and_parse)
            .borrowed_str(Self::env_expand_and_parse)
            .bool(deserialize_value!(BoolDeserializer))
            .i8(deserialize_value!(I8Deserializer))
            .i16(deserialize_value!(I16Deserializer))
            .i32(deserialize_value!(I32Deserializer))
            .i64(deserialize_value!(I64Deserializer))
            .i128(deserialize_value!(I128Deserializer))
            .u8(deserialize_value!(U8Deserializer))
            .u16(deserialize_value!(U16Deserializer))
            .u32(deserialize_value!(U32Deserializer))
            .u64(deserialize_value!(U64Deserializer))
            .u128(deserialize_value!(U128Deserializer))
            .f32(deserialize_value!(F32Deserializer))
            .f64(deserialize_value!(F64Deserializer))
            .char(deserialize_value!(CharDeserializer))
            .bytes(deserialize_value!(BytesDeserializer))
            .borrowed_bytes(deserialize_value!(BorrowedBytesDeserializer))
            .seq(|seq| seq.deserialize().map(Self))
            .map(|map| map.deserialize().map(Self))
            .deserialize(deserializer)
    }
}

impl<T: Clone> Clone for EnvField<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Copy> Copy for EnvField<T> {}

impl<T: FromStr> FromStr for EnvField<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl<T: Default> Default for EnvField<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

impl<T: Debug> Debug for EnvField<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Deref for EnvField<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for EnvField<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: PartialEq> PartialEq<T> for EnvField<T> {
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other)
    }
}

impl<T: PartialEq<str>> PartialEq<str> for EnvField<T> {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl<T: PartialEq> PartialEq for EnvField<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T: Eq> Eq for EnvField<T> {}

impl<T: PartialOrd> PartialOrd<T> for EnvField<T> {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<T: PartialOrd> PartialOrd for EnvField<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Ord> Ord for EnvField<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

macro_rules! impl_unary_op {
    ($trait:ident, $method:ident) => {
        impl<T: $trait> $trait for EnvField<T> {
            type Output = <T as $trait>::Output;

            fn $method(self) -> Self::Output {
                self.0.$method()
            }
        }
    };
}

macro_rules! impl_binary_op {
    ($trait:ident, $method:ident) => {
        impl<T: $trait> $trait<T> for EnvField<T> {
            type Output = <T as $trait>::Output;

            fn $method(self, rhs: T) -> Self::Output {
                self.0.$method(rhs)
            }
        }

        impl<T: $trait> $trait for EnvField<T> {
            type Output = <T as $trait>::Output;

            fn $method(self, rhs: Self) -> Self::Output {
                self.0.$method(rhs.0)
            }
        }
    };
}

macro_rules! impl_binary_assign_op {
    ($trait:ident, $method:ident) => {
        impl<T: $trait> $trait<T> for EnvField<T> {
            fn $method(&mut self, rhs: T) {
                self.0.$method(rhs);
            }
        }

        impl<T: $trait> $trait for EnvField<T> {
            fn $method(&mut self, rhs: Self) {
                self.0.$method(rhs.0);
            }
        }
    };
}

impl_unary_op!(Neg, neg);
impl_unary_op!(Not, not);

impl_binary_op!(Add, add);
impl_binary_op!(Sub, sub);
impl_binary_op!(Mul, mul);
impl_binary_op!(Div, div);
impl_binary_op!(Rem, rem);
impl_binary_op!(BitAnd, bitand);
impl_binary_op!(BitOr, bitor);
impl_binary_op!(BitXor, bitxor);
impl_binary_op!(Shl, shl);
impl_binary_op!(Shr, shr);

impl_binary_assign_op!(AddAssign, add_assign);
impl_binary_assign_op!(SubAssign, sub_assign);
impl_binary_assign_op!(MulAssign, mul_assign);
impl_binary_assign_op!(DivAssign, div_assign);
impl_binary_assign_op!(RemAssign, rem_assign);
impl_binary_assign_op!(BitAndAssign, bitand_assign);
impl_binary_assign_op!(BitOrAssign, bitor_assign);
impl_binary_assign_op!(BitXorAssign, bitxor_assign);
impl_binary_assign_op!(ShlAssign, shl_assign);
impl_binary_assign_op!(ShrAssign, shr_assign);
