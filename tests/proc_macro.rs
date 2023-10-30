extern crate alloc;

use std::{env, str::FromStr, unreachable};

use derive_more::FromStr;
use indoc::indoc;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_env_field::env_field_wrap;

fn de_se_de_test<T: Serialize + DeserializeOwned>(
    source_text: &'static str,
    check_value: impl Fn(&T),
    expected_serialized: &'static str,
) {
    let deserialized: T = toml::from_str(source_text).unwrap();
    check_value(&deserialized);

    let serialized = toml::to_string_pretty(&deserialized).unwrap();
    assert_eq!(serialized, expected_serialized);

    let deserialized_again: T = toml::from_str(&serialized).unwrap();
    check_value(&deserialized_again);
}

fn de_se_de_json_test<T: Serialize + DeserializeOwned>(
    source_text: &'static str,
    check_value: impl Fn(&T),
    expected_serialized: &'static str,
) {
    let deserialized: T = serde_json::from_str(source_text).unwrap();
    check_value(&deserialized);

    let serialized = serde_json::to_string_pretty(&deserialized).unwrap();
    assert_eq!(serialized, expected_serialized);

    let deserialized_again: T = serde_json::from_str(&serialized).unwrap();
    check_value(&deserialized_again);
}

#[test]
fn test_wrap_required_fields() {
    #[env_field_wrap]
    #[derive(Serialize, Deserialize)]
    struct Test {
        name: String,
        size: usize,
    }

    de_se_de_test::<Test>(
        r#"
            name = "${NAME_test_required:-Default Entry}"
            size = "${SIZE:-0}"
        "#,
        |de| {
            assert_eq!(&de.name, "Default Entry");
            assert_eq!(de.size, 0);
        },
        indoc! {r#"
            name = "Default Entry"
            size = 0
        "#},
    );

    env::set_var("NAME_test_required", "Example Name");
    de_se_de_test::<Test>(
        r#"
            name = "${NAME_test_required:-Default Entry}"
            size = 44
        "#,
        |de| {
            assert_eq!(&de.name, "Example Name");
            assert_eq!(de.size, 44);
        },
        indoc! {r#"
            name = "Example Name"
            size = 44
        "#},
    );

    de_se_de_test::<Test>(
        r#"
            name = "Not-Var"
            size = 42
        "#,
        |de| {
            assert_eq!(&de.name, "Not-Var");
            assert_eq!(de.size, 42);
        },
        indoc! {r#"
            name = "Not-Var"
            size = 42
        "#},
    );

    env::set_var("SIZE_test_required", "1023");
    de_se_de_test::<Test>(
        r#"
            name = "Not-Var"
            size = "$SIZE_test_required"
        "#,
        |de| {
            assert_eq!(&de.name, "Not-Var");
            assert_eq!(de.size, 1023);
        },
        indoc! {r#"
            name = "Not-Var"
            size = 1023
        "#},
    );
}

#[test]
fn test_wrap_optional_fields() {
    #[env_field_wrap]
    #[derive(Serialize, Deserialize)]
    struct Test {
        name: Option<String>,
        size: std::option::Option<usize>,
        weight: core::option::Option<i32>,
    }

    de_se_de_test::<Test>(
        "",
        |de| {
            assert!(&de.name.is_none());
            assert!(de.size.is_none());
            assert!(de.weight.is_none());
        },
        "",
    );

    env::set_var("NAME_test_optional", "Name from Env");
    de_se_de_test::<Test>(
        r#"
            name = "$NAME_test_optional"
        "#,
        |de| {
            assert_eq!(de.name.as_ref().unwrap(), "Name from Env");
            assert!(de.size.is_none());
        },
        indoc! {r#"
            name = "Name from Env"
        "#},
    );

    de_se_de_test::<Test>(
        r#"
            size = 514
        "#,
        |de| {
            assert!(de.name.is_none());
            assert_eq!(de.size.unwrap(), 514);
            assert!(de.weight.is_none());
        },
        indoc! {r#"
            size = 514
        "#},
    );

    env::set_var("WEIGHT_test_optional", "-222");
    de_se_de_test::<Test>(
        r#"
            name = "Not-Var"
            size = "${SIZE_test_optional:-12}"
            weight = "$WEIGHT_test_optional"
        "#,
        |de| {
            assert_eq!(de.name.as_ref().unwrap(), "Not-Var");
            assert_eq!(de.size.unwrap(), 12);
            assert_eq!(de.weight.unwrap(), -222);
        },
        indoc! {r#"
            name = "Not-Var"
            size = 12
            weight = -222
        "#},
    );
}

#[test]
fn test_wrap_default_fields() {
    #[env_field_wrap]
    #[derive(Serialize, Deserialize)]
    struct Test {
        #[serde(default)]
        number_with_default: TheAnswerByDefault,
    }

    #[derive(Serialize, Deserialize, FromStr)]
    #[serde(transparent)]
    pub struct TheAnswerByDefault(i32);
    impl Default for TheAnswerByDefault {
        fn default() -> Self {
            Self(42)
        }
    }

    de_se_de_test::<Test>(
        "",
        |de| {
            assert_eq!(de.number_with_default.0, 42);
        },
        indoc! {r#"
            number_with_default = 42
        "#},
    );

    de_se_de_test::<Test>(
        "number_with_default = 512",
        |de| {
            assert_eq!(de.number_with_default.0, 512);
        },
        indoc! {r#"
            number_with_default = 512
        "#},
    );

    de_se_de_test::<Test>(
        r#"number_with_default = "${NUMBER_test_default:-144}""#,
        |de| {
            assert_eq!(de.number_with_default.0, 144);
        },
        indoc! {r#"
            number_with_default = 144
        "#},
    );

    env::set_var("NUMBER_test_default", "777");
    de_se_de_test::<Test>(
        r#"number_with_default = "${NUMBER_test_default:-144}""#,
        |de| {
            assert_eq!(de.number_with_default.0, 777);
        },
        indoc! {r#"
            number_with_default = 777
        "#},
    );
}

#[test]
fn test_wrap_seq_fields() {
    #[env_field_wrap]
    #[derive(Serialize, Deserialize)]
    struct Test {
        numbers: Vec<i32>,
        strings: std::vec::Vec<String>,
        bools: alloc::vec::Vec<bool>,
    }

    env::set_var("NUMBER1_test_seq", "-1024");
    env::set_var("TWO_test_seq", "Str from Env");
    env::set_var("BOOL_test_seq", "true");
    de_se_de_test::<Test>(
        r#"
            numbers = [
                42,
                "$NUMBER1_test_seq",
                "${NUMBER2_test_seq:-48}",
                -512
            ]
            strings = [
                "ONE",
                "$TWO_test_seq"
            ]
            bools = [
                "$BOOL_test_seq",
                false,
                true
            ]
        "#,
        |de| {
            assert!(de.numbers.iter().eq([42, -1024, 48, -512,].iter()));

            assert!(de
                .strings
                .iter()
                .map(|e| e.as_str())
                .eq(["ONE", "Str from Env",].into_iter()));

            assert!(de.bools.iter().eq([true, false, true].iter()));
        },
        indoc! {r#"
            numbers = [
                42,
                -1024,
                48,
                -512,
            ]
            strings = [
                "ONE",
                "Str from Env",
            ]
            bools = [
                true,
                false,
                true,
            ]
        "#},
    );

    env::set_var("NUMBER1_test_seq", "111");
    env::set_var("NUMBER2_test_seq", "-64");
    de_se_de_test::<Test>(
        r#"
            numbers = [
                42,
                "$NUMBER1_test_seq",
                "${NUMBER2_test_seq:-48}",
                -512
            ]
            strings = [
                "Another",
                "$TWO_test_seq"
            ]
            bools = [
                "$BOOL_test_seq",
                false,
                true
            ]
        "#,
        |de| {
            assert!(de.numbers.iter().eq([42, 111, -64, -512,].iter()));

            assert!(de
                .strings
                .iter()
                .map(|e| e.as_str())
                .eq(["Another", "Str from Env",].into_iter()));

            assert!(de.bools.iter().eq([true, false, true].iter()));
        },
        indoc! {r#"
            numbers = [
                42,
                111,
                -64,
                -512,
            ]
            strings = [
                "Another",
                "Str from Env",
            ]
            bools = [
                true,
                false,
                true,
            ]
        "#},
    );
}

#[test]
fn test_wrap_map_fields() {
    #[env_field_wrap]
    #[derive(Serialize, Deserialize)]
    struct Test {
        map: Map,
    }

    #[derive(Serialize, Deserialize)]
    struct Map {
        n: i32,
        s: String,
        b: bool,
    }

    impl FromStr for Map {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut segments = s.split(';');

            let n = segments.next().unwrap().parse().unwrap();
            let s = segments.next().unwrap().to_string();
            let b = segments.next().unwrap().parse().unwrap();

            Ok(Self { n, s, b })
        }
    }

    de_se_de_test::<Test>(
        r#"
            map.n = 44
            map.s = "Hello World"
            map.b = false
        "#,
        |de| {
            assert_eq!(de.map.n, 44);
            assert_eq!(&de.map.s, "Hello World");
            assert_eq!(de.map.b, false);
        },
        indoc! {r#"
            [map]
            n = 44
            s = "Hello World"
            b = false
        "#},
    );

    env::set_var("MAP_test_map", "1111;Test Env String;true");
    de_se_de_test::<Test>(
        r#"
            map = "$MAP_test_map"
        "#,
        |de| {
            assert_eq!(de.map.n, 1111);
            assert_eq!(&de.map.s, "Test Env String");
            assert_eq!(de.map.b, true);
        },
        indoc! {r#"
            [map]
            n = 1111
            s = "Test Env String"
            b = true
        "#},
    );
}

#[test]
fn test_wrap_primitives() {
    #[env_field_wrap]
    #[derive(Serialize, Deserialize)]
    struct Test {
        b: bool,
        c: char,
        s: String,
        ni8: i8,
        ni16: i16,
        ni32: i32,
        ni64: i64,
        nu8: u8,
        nu16: u16,
        nu32: u32,
        nu64: u64,
        nf32: f32,
        nf64: f64,
    }

    de_se_de_test::<Test>(
        r#"
            b = true
            c = 'A'
            s = "Hello"
            ni8 = -128
            ni16 = -1024
            ni32 = 0x20000
            ni64 = 0x2000000
            nu8 = 128
            nu16 = 1024
            nu32 = 0x20000
            nu64 = 0x2000000
            nf32 = 42
            nf64 = 64.0
        "#,
        |de| {
            assert_eq!(de.b, true);
            assert_eq!(de.c, 'A');
            assert_eq!(&de.s, "Hello");
            assert_eq!(de.ni8, -128);
            assert_eq!(de.ni16, -1024);
            assert_eq!(de.ni32, 0x20000);
            assert_eq!(de.ni64, 0x2000000);
            assert_eq!(de.nu8, 128);
            assert_eq!(de.nu16, 1024);
            assert_eq!(de.nu32, 0x20000);
            assert_eq!(de.nu64, 0x2000000);
            assert_eq!(de.nf32, 42.0);
            assert_eq!(de.nf64, 64.0);
        },
        indoc! {r#"
            b = true
            c = "A"
            s = "Hello"
            ni8 = -128
            ni16 = -1024
            ni32 = 131072
            ni64 = 33554432
            nu8 = 128
            nu16 = 1024
            nu32 = 131072
            nu64 = 33554432
            nf32 = 42.0
            nf64 = 64.0
        "#},
    );

    env::set_var("BOOL_test_primitive", "false");
    env::set_var("CHAR_test_primitive", "S");
    env::set_var("STR_test_primitive", "Goodbye");
    env::set_var("I8_test_primitive", "-100");
    env::set_var("I16_test_primitive", "-20000");
    env::set_var("I32_test_primitive", "-3000000");
    env::set_var("I64_test_primitive", "-4000000000");
    env::set_var("U8_test_primitive", "100");
    env::set_var("U16_test_primitive", "20000");
    env::set_var("U32_test_primitive", "3000000");
    env::set_var("U64_test_primitive", "4000000000");
    env::set_var("F32_test_primitive", "-114.0");
    env::set_var("F64_test_primitive", "115");

    de_se_de_test::<Test>(
        r#"
            b = "$BOOL_test_primitive"
            c = '$CHAR_test_primitive'
            s = "$STR_test_primitive"
            ni8 = "$I8_test_primitive"
            ni16 = "$I16_test_primitive"
            ni32 = "$I32_test_primitive"
            ni64 = "$I64_test_primitive"
            nu8 = "$U8_test_primitive"
            nu16 = "$U16_test_primitive"
            nu32 = "$U32_test_primitive"
            nu64 = "$U64_test_primitive"
            nf32 = "$F32_test_primitive"
            nf64 = "$F64_test_primitive"
        "#,
        |de| {
            assert_eq!(de.b, false);
            assert_eq!(de.c, 'S');
            assert_eq!(&de.s, "Goodbye");
            assert_eq!(de.ni8, -100);
            assert_eq!(de.ni16, -20000);
            assert_eq!(de.ni32, -3000000);
            assert_eq!(de.ni64, -4000000000);
            assert_eq!(de.nu8, 100);
            assert_eq!(de.nu16, 20000);
            assert_eq!(de.nu32, 3000000);
            assert_eq!(de.nu64, 4000000000);
            assert_eq!(de.nf32, -114.0);
            assert_eq!(de.nf64, 115.0);
        },
        indoc! {r#"
            b = false
            c = "S"
            s = "Goodbye"
            ni8 = -100
            ni16 = -20000
            ni32 = -3000000
            ni64 = -4000000000
            nu8 = 100
            nu16 = 20000
            nu32 = 3000000
            nu64 = 4000000000
            nf32 = -114.0
            nf64 = 115.0
        "#},
    );
}

#[test]
fn test_wrap_skip() {
    #[env_field_wrap]
    #[derive(Serialize, Deserialize)]
    struct Test {
        wrapped: String,

        #[env_field_wrap(skip)]
        skipped: String,
    }

    env::set_var("WRAPPED", "From Env");
    de_se_de_test::<Test>(
        r#"
            wrapped = "$WRAPPED"
            skipped = "$SKIPPED"
        "#,
        |de| {
            assert_eq!(&de.wrapped, "From Env");
            assert_eq!(&de.skipped, "$SKIPPED");
        },
        indoc! {r#"
            wrapped = "From Env"
            skipped = "$SKIPPED"
        "#},
    );
}

#[test]
fn test_wrap_generics_only() {
    #[env_field_wrap]
    #[derive(Serialize, Deserialize, Debug)]
    struct Test {
        #[env_field_wrap(generics_only)]
        generics: TwoGenerics<String, i32>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct TwoGenerics<A, B> {
        a: A,
        b: B,
    }

    env::set_var("GENERICS_STR", "env string");
    env::set_var("GENERICS_I32", "517");
    de_se_de_test::<Test>(
        r#"
            [generics]
            a = "$GENERICS_STR"
            b = "$GENERICS_I32"
        "#,
        |de| {
            assert_eq!(&de.generics.a, "env string");
            assert_eq!(de.generics.b, 517);
        },
        indoc! {r#"
            [generics]
            a = "env string"
            b = 517
        "#},
    );

    toml::from_str::<Test>(
        r#"
        generics = "from str"
    "#,
    )
    .unwrap_err();
}

#[test]
fn test_wrap_tuple_struct() {
    #[env_field_wrap]
    #[derive(Serialize, Deserialize)]
    struct Test(
        #[env_field_wrap(generics_only)] TwoGenerics<i32, bool>,
        #[env_field_wrap(skip)] String,
        Option<i32>,
        Vec<bool>,
    );

    #[derive(Serialize, Deserialize)]
    struct TwoGenerics<A, B> {
        a: A,
        b: B,
    }

    env::set_var("NUM_tup", "333");
    env::set_var("BOOL_tup", "true");
    de_se_de_json_test::<Test>(
        r#"
            [
              {
                "a": "$NUM_tup",
                "b": "$BOOL_tup"
              },
              "$WRAPPED_tup",
              "$NUM_tup",
              ["$BOOL_tup", false, false]
            ]
        "#,
        |de| {
            assert_eq!(de.0.a, 333);
            assert_eq!(de.0.b, true);
            assert_eq!(&de.1, "$WRAPPED_tup");
            assert_eq!(de.2.unwrap(), 333);
            assert!(de.3.iter().eq([true, false, false].iter()));
        },
        indoc! {
            r#"
            [
              {
                "a": 333,
                "b": true
              },
              "$WRAPPED_tup",
              333,
              [
                true,
                false,
                false
              ]
            ]"#
        },
    );
}

#[test]
fn test_wrap_enum() {
    #[env_field_wrap]
    #[derive(Serialize, Deserialize)]
    enum Test {
        Wrapped(String),

        #[env_field_wrap(skip)]
        Skipped {
            inner_str: String,
        },

        Inner(
            #[env_field_wrap(skip)] String,
            Option<i32>,
            Vec<bool>,
            #[env_field_wrap(generics_only)] TwoGenerics<i32, bool>,
        ),
    }

    #[derive(Serialize, Deserialize)]
    struct TwoGenerics<A, B> {
        a: A,
        b: B,
    }

    env::set_var("WRAPPED_enum", "Enum String From Env");
    de_se_de_test::<Test>(
        r#"
            Wrapped = "$WRAPPED_enum"
        "#,
        |de| {
            assert!(matches![de, Test::Wrapped(s) if s == "Enum String From Env"]);
        },
        indoc! {r#"
            Wrapped = "Enum String From Env"
        "#},
    );

    de_se_de_json_test::<Test>(
        r#"
            {
                "Skipped": {
                    "inner_str": "$WRAPPED_enum"
                }
            }
        "#,
        |de| {
            assert!(matches![de, Test::Skipped { inner_str } if inner_str == "$WRAPPED_enum"]);
        },
        indoc! {
            r#"
            {
              "Skipped": {
                "inner_str": "$WRAPPED_enum"
              }
            }"#
        },
    );

    env::set_var("NUM_enum", "117");
    env::set_var("BOOL_enum", "false");
    de_se_de_json_test::<Test>(
        r#"
            {
                "Inner": [
                    "$WRAPPED_enum",
                    "$NUM_enum",
                    [true, true, "$BOOL_enum"],
                    {
                        "a": "$NUM_enum",
                        "b": "$BOOL_enum"
                    }
                ]
            }
        "#,
        |de| {
            assert!(matches![de, Test::Inner(_, _, _, _)]);
            let Test::Inner(skipped, o, v, g) = de else {
                unreachable!()
            };

            assert_eq!(skipped, "$WRAPPED_enum");
            assert_eq!(o.unwrap(), 117);
            assert!(v.iter().eq([true, true, false].iter()));
            assert_eq!(g.a, 117);
            assert_eq!(g.b, false);
        },
        indoc! {
            r#"
            {
              "Inner": [
                "$WRAPPED_enum",
                117,
                [
                  true,
                  true,
                  false
                ],
                {
                  "a": 117,
                  "b": false
                }
              ]
            }"#
        },
    );
}
