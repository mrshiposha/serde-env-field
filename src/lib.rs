use std::{
    fmt::{self, Debug},
    ops::*,
    str::FromStr,
};

use serde::{de::Error, Deserialize, Serialize};
use serde_untagged::UntaggedEnumVisitor;

/// A field that deserializes either as `T` or as `String`
/// with all environment variables expanded via `shellexpand`.
///
/// Requires `T` to implement the `FromStr` trait
/// for deserialization from String after environment variables expansion.
///
/// Works nicely with `Option` and `#[serde(default)]`.
pub struct EnvField<T>(T);

impl<T> EnvField<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for EnvField<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Serialize> Serialize for EnvField<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
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
            .string(|str_data| match shellexpand::env(&str_data) {
                Ok(expanded) => expanded.parse().map(Self).map_err(Error::custom),
                Err(err) => Err(Error::custom(err)),
            })
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
