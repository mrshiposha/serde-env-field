# serde-env-field
[![](https://docs.rs/serde-env-field/badge.svg)](https://docs.rs/serde-env-field/) [![](https://img.shields.io/crates/v/serde-env-field.svg)](https://crates.io/crates/serde-env-field) [![](https://img.shields.io/crates/d/serde-env-field.svg)](https://crates.io/crates/serde-env-field)

This crate provides the `EnvField<T>` type capable of deserializing the `T` type
from a string with environment variables if the `T` implements the `FromStr` trait.

During deserialization, the `EnvField` will try to deserialize the data as a string and expand all
the environment variables. After the expansion, the resulting string will be used
to construct the `T` type using the `FromStr` trait.

If the supplied data was not a string, the `EnvField`
will attempt to deserialize the `T` type directly from the data.

See the [EnvField](https://docs.rs/serde_env_field/struct.EnvField.html) documentation for details.
