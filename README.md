# serde-env-field
![CI](https://github.com/mrshiposha/serde-env-field/actions/workflows/rust.yml/badge.svg) [![](https://docs.rs/serde-env-field/badge.svg)](https://docs.rs/serde-env-field/) [![](https://img.shields.io/crates/v/serde-env-field.svg)](https://crates.io/crates/serde-env-field) [![](https://img.shields.io/crates/d/serde-env-field.svg)](https://crates.io/crates/serde-env-field)

This crate provides the `EnvField<T>` type capable of deserializing the `T` type
from a string with environment variables if the `T` implements the `FromStr` trait.

During deserialization, the `EnvField` will try to deserialize the data as a string and expand all
the environment variables. After the expansion, the resulting string will be used
to construct the `T` type using the `FromStr` trait.

If the supplied data was not a string, the `EnvField`
will attempt to deserialize the `T` type directly from the data.

The `EnvField` works nicely with `Option`, `Vec`, and `#[serde(default)]`.

#### Example

```rust
#[derive(Serialize, Deserialize)]
struct Example {
    name: EnvField<String>,
    size: EnvField<usize>,
    num: EnvField<i32>,
}

std::env::set_var("SIZE", "100");

let de: Example = toml::from_str(r#"
    name = "${NAME:-Default Name}"

    size = "$SIZE"

    num = 42
"#).unwrap();

assert_eq!(&de.name, "Default Name");
assert_eq!(de.size, 100);
assert_eq!(de.num, 42);
```


See the [EnvField](https://docs.rs/serde-env-field/latest/serde_env_field/struct.EnvField.html) documentation for details.
