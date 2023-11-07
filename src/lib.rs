//! This crate provides the [`EnvField<T>`] type capable of deserializing the `T` type
//! from a string with environment variables expanded.
//!
//! During deserialization, the `EnvField` will try to deserialize the data as a string and expand all
//! the environment variables. After the expansion, the resulting string will be used
//! to construct the `T` value.
//! By default, the `EnvField` will construct the `T` value using the `FromStr` trait.
//! However, it is possible to make it use the `Deserialize` trait using the [`UseDeserialize`] marker.
//!
//! If the supplied data was not a string, the `EnvField`
//! will attempt to deserialize the `T` type directly from the data.
//!
//! The `EnvField` works nicely with `Option`, `Vec`, and `#[serde(default)]`.
//!
//! Also, the crate provides the [`env_field_wrap`] attribute that wraps
//! all the fields of a struct or an enum with the `EnvField` type.
//! The attribute also honors the optional and vector fields.
//!
//! #### `EnvField` Example
//!
//! ```
//! # use serde::{Serialize, Deserialize};
//! # use serde_env_field::EnvField;
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
//! #### `env_field_wrap` Example
//!
//! ```
//! # use serde::{Serialize, Deserialize};
//! # use serde_env_field::env_field_wrap;
//! #[env_field_wrap]
//! #[derive(Serialize, Deserialize)]
//! struct Example {
//!     name: String,
//!     size: usize,
//!     num: i32,
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
//!
//! ```
//!
//! See the description of the [`EnvField`] and the [`env_field_wrap`] for details.

#![warn(missing_docs)]

use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    ops::*,
    str::FromStr,
};

use serde::{
    de::{self, value::StringDeserializer, Error},
    Deserialize, Serialize,
};
use serde_untagged::{de::Error as UntaggedError, UntaggedEnumVisitor};

/// The `env_field_wrap` wraps all the fields of a struct or an enum with the [`EnvField`] type.
///
/// The [`Option<T>`] fields will remain optional, with only the `T` type wrapped with the `EnvField`.
///
/// Similarly, the [`Vec<T>`] fields will remain vectors, with only the `T` type wrapped.
///
/// It is possible to skip a field using the `#[env_field_wrap(skip)]` attribute.
/// The fields that already have the `EnvField` type skipped automatically.
///
/// Also, one can wrap a generic type similarly to an `Option` field
/// using the `#[env_field_wrap(generics_only)]` attribute.
///
/// **NOTE:** If you are using the `#[derive(Deserialize)]`,
/// the `#[env_field_wrap]` attribute must appear **before** it.
/// Otherwise, it won't work.
///
/// ### Examples
///
/// #### Basic
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::env_field_wrap;
/// #[env_field_wrap]
/// #[derive(Serialize, Deserialize)]
/// struct Example {
///     name: String,
///     size: usize,
///     num: i32,
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
/// # use serde_env_field::env_field_wrap;
/// #[env_field_wrap]
/// #[derive(Serialize, Deserialize)]
/// struct Example {
///     required: i32,
///     optional: Option<i32>,
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
/// #### Vector fields
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::env_field_wrap;
/// #[env_field_wrap]
/// #[derive(Serialize, Deserialize)]
/// struct Example {
///     seq: Vec<i32>,
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
/// #### Skip a field
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::env_field_wrap;
/// #[env_field_wrap]
/// #[derive(Serialize, Deserialize)]
/// struct Example {
///    wrapped: String,
///
///    #[env_field_wrap(skip)]
///    skipped: String,
/// }
///
/// std::env::set_var("WRAPPED", "From Env");
/// let de: Example = toml::from_str(r#"
///     wrapped = "$WRAPPED"
///     skipped = "$SKIPPED"
/// "#).unwrap();
///
/// assert_eq!(&de.wrapped, "From Env");
/// assert_eq!(&de.skipped, "$SKIPPED");
///
/// ```
///
/// #### Skip an enum variant
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::env_field_wrap;
/// #[env_field_wrap]
/// #[derive(Serialize, Deserialize)]
/// enum Example {
///     Wrapped(String),
///
///     #[env_field_wrap(skip)]
///     Skipped {
///         inner_str: String,
///     },
/// }
///
/// std::env::set_var("WRAPPED", "From Env");
/// let de: Example = serde_json::from_str(r#"
///     {
///         "Wrapped": "$WRAPPED"
///     }
/// "#).unwrap();
///
/// assert!(matches![de, Example::Wrapped(s) if &s == "From Env"]);
///
/// let de: Example = serde_json::from_str(r#"
///     {
///         "Skipped": {
///             "inner_str": "$WRAPPED"
///         }
///     }
/// "#).unwrap();
///
/// assert!(matches![de, Example::Skipped { inner_str } if inner_str == "$WRAPPED"]);
///
/// ```
///
/// #### Wrap generics only
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::{env_field_wrap, EnvField, UseDeserialize};
/// #[env_field_wrap]
/// #[derive(Serialize, Deserialize)]
/// struct Example {
///     // Will become
///     //    `Generics<EnvField<String>, EnvField<i32>, EnvField<Variants, UseDeserialize>>`
///     // instead of
///     //    `EnvField<Generics<String, i32, EnvField<Variants, UseDeserialize>>>`.
///     //
///     // Note:
///     //  * if a generic is already wrapped into the `EnvField`, it *won't* be wrapped again.
///     //  * the `Generics` don't need to implement the `FromStr` in this case.
///     #[env_field_wrap(generics_only)]
///     generics: Generics<String, i32, EnvField<Variants, UseDeserialize>>,
/// }
///
/// #[derive(Serialize, Deserialize)]
/// struct Generics<A, B, C> {
///     a: A,
///     b: B,
///     c: C,
/// }
///
/// #[derive(Serialize, Deserialize, Debug)]
/// #[serde(rename_all = "kebab-case")]
/// enum Variants {
///     FirstVariant,
///     SecondVariant,
/// }
///
/// std::env::set_var("GENERICS_STR", "env string");
/// std::env::set_var("GENERICS_I32", "517");
/// std::env::set_var("GENERICS_VARIANT", "first-variant");
/// let de: Example = toml::from_str(r#"
///     [generics]
///     a = "$GENERICS_STR"
///     b = "$GENERICS_I32"
///     c = "$GENERICS_VARIANT"
/// "#).unwrap();
///
/// assert_eq!(&de.generics.a, "env string");
/// assert_eq!(de.generics.b, 517);
/// assert!(matches!(*de.generics.c, Variants::FirstVariant));
///
/// ```
pub use serde_env_field_wrap::env_field_wrap;

/// A field that deserializes either as `T` or as `String`
/// with all environment variables expanded via the [`shellexpand`] crate.
///
/// By default, it requires `T` to implement the `FromStr` trait
/// for deserialization from `String` after environment variables expansion.
///
/// You can use the [`UseDeserialize`] to bypass the `FromStr` and deserialize the `T`
/// directly from the string with all environment variables expanded.
///
/// The `EnvField` serializes transparently as the `T` type if the `T` is serializable.
///
/// Works nicely with `Option`, `Vec`, and `#[serde(default)]`.
///
/// Note: if you want to wrap all the fields of a struct or an enum
/// with the `EnvField`, you might want to use the [`env_field_wrap`] attribute.
///
/// ### Examples
///
/// #### Basic
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::EnvField;
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
/// #### Deserialization without `FromStr`
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::EnvField;
/// use serde_env_field::UseDeserialize;
///
/// #[derive(Serialize, Deserialize)]
/// struct Example {
///     variant: EnvField<Variants, UseDeserialize>
/// }
///
/// #[derive(Serialize, Deserialize)]
/// #[serde(rename_all = "kebab-case")]
/// enum Variants {
///     AUsefullVariant,
///     AnotherCoolVariant,
/// }
///
/// let de: Example = toml::from_str(r#"
///     variant = "a-usefull-variant"
/// "#).unwrap();
/// assert!(matches!(*de.variant, Variants::AUsefullVariant));
///
/// std::env::set_var("SELECTED_VARIANT", "another-cool-variant");
/// let de: Example = toml::from_str(r#"
///     variant = "$SELECTED_VARIANT"
/// "#).unwrap();
/// assert!(matches!(*de.variant, Variants::AnotherCoolVariant));
/// ```
///
/// #### Deserialization with `FromStr`
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::EnvField;
/// # use std::str::FromStr;
/// # use std::num::ParseIntError;
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
pub struct EnvField<T, Variant = UseFromStr>(T, PhantomData<Variant>);

/// A marker type for passing into the [`EnvField<T>`] type as a second parameter.
///
/// The `EnvField` will use the [`FromStr`] trait for constructing the `T` type
/// after the environment variables expansion.
///
/// This is the default for the `EnvField`.
pub struct UseFromStr;

/// A marker type for passing into the [`EnvField<T>`] type as a second parameter.
///
/// The `EnvField` will use the [`Deserialize`] trait for constructing the `T` type
/// after the environment variables expansion.
/// I.e., the `T` will be deserialized directly from the string with all environment variables expanded.
///
/// ### Example
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use serde_env_field::{EnvField, UseDeserialize};
/// #[derive(Serialize, Deserialize)]
/// struct Example {
///     variant: EnvField<Variants, UseDeserialize>
/// }
///
/// #[derive(Serialize, Deserialize)]
/// #[serde(rename_all = "kebab-case")]
/// enum Variants {
///     AUsefullVariant,
///     AnotherCoolVariant,
/// }
///
/// let de: Example = toml::from_str(r#"
///     variant = "a-usefull-variant"
/// "#).unwrap();
/// assert!(matches!(*de.variant, Variants::AUsefullVariant));
///
/// std::env::set_var("SELECTED_VARIANT", "another-cool-variant");
/// let de: Example = toml::from_str(r#"
///     variant = "$SELECTED_VARIANT"
/// "#).unwrap();
/// assert!(matches!(*de.variant, Variants::AnotherCoolVariant));
/// ```
pub struct UseDeserialize;

impl<T: Serialize, V> Serialize for EnvField<T, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T, V> EnvField<T, V> {
    /// Unwraps the value, consuming the env field.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> EnvField<T, UseFromStr>
where
    T: FromStr,
    <T as FromStr>::Err: fmt::Display,
{
    fn env_expand_and_parse(str_data: &str) -> Result<Self, UntaggedError> {
        match shellexpand::env(&str_data) {
            Ok(expanded) => expanded
                .parse()
                .map(|v| Self(v, PhantomData))
                .map_err(Error::custom),
            Err(err) => Err(Error::custom(err)),
        }
    }
}

impl<'de, T> EnvField<T, UseDeserialize>
where
    T: Deserialize<'de>,
{
    fn env_expand_and_deserialize(str_data: &str) -> Result<Self, UntaggedError> {
        match shellexpand::env(&str_data) {
            Ok(expanded) => T::deserialize(StringDeserializer::new(expanded.into()))
                .map(|v| Self(v, PhantomData)),
            Err(err) => Err(Error::custom(err)),
        }
    }
}

impl<T, V> From<T> for EnvField<T, V> {
    fn from(value: T) -> Self {
        Self(value, PhantomData)
    }
}

macro_rules! deserialize_value {
    ($de:ident) => {
        |v| T::deserialize(de::value::$de::new(v)).map(|v| Self(v, PhantomData))
    };
}

impl<'de, T> Deserialize<'de> for EnvField<T, UseFromStr>
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
            .seq(|seq| seq.deserialize().map(|v| Self(v, PhantomData)))
            .map(|map| map.deserialize().map(|v| Self(v, PhantomData)))
            .deserialize(deserializer)
    }
}

impl<'de, T> Deserialize<'de> for EnvField<T, UseDeserialize>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        UntaggedEnumVisitor::new()
            .string(Self::env_expand_and_deserialize)
            .borrowed_str(Self::env_expand_and_deserialize)
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
            .seq(|seq| seq.deserialize().map(|v| Self(v, PhantomData)))
            .map(|map| map.deserialize().map(|v| Self(v, PhantomData)))
            .deserialize(deserializer)
    }
}

impl<T: Clone, V> Clone for EnvField<T, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T: Copy, V> Copy for EnvField<T, V> {}

impl<T: FromStr, V> FromStr for EnvField<T, V> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?, PhantomData))
    }
}

impl<T: Default, V> Default for EnvField<T, V> {
    fn default() -> Self {
        Self(T::default(), PhantomData)
    }
}

impl<T: Debug, V> Debug for EnvField<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T, V> Deref for EnvField<T, V> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, V> DerefMut for EnvField<T, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: PartialEq, V> PartialEq<T> for EnvField<T, V> {
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other)
    }
}

impl<T: PartialEq<str>, V> PartialEq<str> for EnvField<T, V> {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl<T: PartialEq, V> PartialEq for EnvField<T, V> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T: Eq, V> Eq for EnvField<T, V> {}

impl<T: PartialOrd, V> PartialOrd<T> for EnvField<T, V> {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<T: PartialOrd, V> PartialOrd for EnvField<T, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Ord, V> Ord for EnvField<T, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

macro_rules! impl_unary_op {
    ($trait:ident, $method:ident) => {
        impl<T: $trait, V> $trait for EnvField<T, V> {
            type Output = <T as $trait>::Output;

            fn $method(self) -> Self::Output {
                self.0.$method()
            }
        }
    };
}

macro_rules! impl_binary_op {
    ($trait:ident, $method:ident) => {
        impl<T: $trait, V> $trait<T> for EnvField<T, V> {
            type Output = <T as $trait>::Output;

            fn $method(self, rhs: T) -> Self::Output {
                self.0.$method(rhs)
            }
        }

        impl<T: $trait, V> $trait for EnvField<T, V> {
            type Output = <T as $trait>::Output;

            fn $method(self, rhs: Self) -> Self::Output {
                self.0.$method(rhs.0)
            }
        }
    };
}

macro_rules! impl_binary_assign_op {
    ($trait:ident, $method:ident) => {
        impl<T: $trait, V> $trait<T> for EnvField<T, V> {
            fn $method(&mut self, rhs: T) {
                self.0.$method(rhs);
            }
        }

        impl<T: $trait, V> $trait for EnvField<T, V> {
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
