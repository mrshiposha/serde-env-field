use std::{assert_eq, env};

use derive_more::FromStr;
use serde::{Deserialize, Serialize};
use serde_env_field::EnvField;

#[derive(Serialize, Deserialize)]
struct Test {
    name: EnvField<String>,
    size: EnvField<usize>,

    optional_description: Option<EnvField<String>>,

    #[serde(default)]
    number_with_default: EnvField<TheAnswerByDefault>,
}

#[derive(Serialize, Deserialize, FromStr)]
pub struct TheAnswerByDefault {
    the_value: i32,
}
impl Default for TheAnswerByDefault {
    fn default() -> Self {
        Self { the_value: 42 }
    }
}

#[test]
fn test_env_field() {
    let source = r#"
        name = "${NAME:-Default Entry}"
        size = "${SIZE:-0}"
    "#;

    let deserialized: Test = toml::from_str(source).unwrap();
    assert_eq!(&deserialized.name, "Default Entry");
    assert_eq!(deserialized.size, 0);
    assert!(deserialized.optional_description.is_none());
    assert_eq!(deserialized.number_with_default.the_value, 42);

    let serialized = toml::to_string_pretty(&deserialized).unwrap();
    assert_eq!(
        serialized,
        r#"name = "Default Entry"
size = 0

[number_with_default]
the_value = 42
"#
    );

    env::set_var("NAME", "Custom Name");
    env::set_var("SIZE", "1023");

    let deserialized: Test = toml::from_str(source).unwrap();
    assert_eq!(&deserialized.name, "Custom Name");
    assert_eq!(deserialized.size, 1023);
    assert!(deserialized.optional_description.is_none());
    assert_eq!(deserialized.number_with_default.the_value, 42);

    let serialized = toml::to_string_pretty(&deserialized).unwrap();
    assert_eq!(
        serialized,
        r#"name = "Custom Name"
size = 1023

[number_with_default]
the_value = 42
"#
    );

    let source = r#"
        name = "${NAME:-Default Entry}"
        size = "${SIZE:-0}"

        optional_description = "The most default entry ever"
        number_with_default.the_value = 112
    "#;

    let deserialized: Test = toml::from_str(source).unwrap();
    assert_eq!(&deserialized.name, "Custom Name");
    assert_eq!(deserialized.size, 1023);
    assert_eq!(
        deserialized.optional_description.as_ref().unwrap(),
        "The most default entry ever"
    );
    assert_eq!(deserialized.number_with_default.the_value, 112);

    let serialized = toml::to_string_pretty(&deserialized).unwrap();
    assert_eq!(
        serialized,
        r#"name = "Custom Name"
size = 1023
optional_description = "The most default entry ever"

[number_with_default]
the_value = 112
"#
    );
}
