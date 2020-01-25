use super::{YamlSchema, YamlValidator};
use serde_yaml::Value;
const DIFFERENT_TYPES: &'static str = r#"---
schema:
  - name: somestring
    type: string

  - name: counter
    type: number

  - name: somedict
    type: dictionary
    key: 
      type: string
    value: 
      type: dictionary
      key:
        type: string
      value:
        type: string
  - name: someobject
    type: object
    fields:
      - name: inside1
        type: string
      - name: inside2
        type: number
"#;

#[test]
fn deserialize_many_types() {
    let _rd= YamlSchema::from_str(DIFFERENT_TYPES).unwrap();
}

const YAML_SCHEMA: &'static str = r#"---
schema:
  - name: schema
    type: list
    inner:
      type: object
      fields:
        - name: name
          type: string
        - name: type
          type: string
        - name: inner
          type: object
          fields:
            - name: type
              type: string
            - name: fields
              type: list
              inner:
                type: dictionary
                key:
                  type: string
                value:
                  type: string
"#;

#[test]
fn validate_yaml_schema() {
    let schema= YamlSchema::from_str(YAML_SCHEMA);
    let yaml: Value = serde_yaml::from_str(YAML_SCHEMA).unwrap();

    assert!(schema.validate(&yaml).is_ok());
}

const MISSING_NAME_FIELD_IN_SCHEMA: &'static str = r#"---
schema:
  - hello: world
"#;

#[test]
fn test_missing_fields_in_schema() {
    let schema= YamlSchema::from_str(YAML_SCHEMA);
    let yaml: Value = serde_yaml::from_str(MISSING_NAME_FIELD_IN_SCHEMA).unwrap();

    let err = schema.validate(&yaml).expect_err("this should fail");
    assert_eq!(format!("{}", err), "missing field, `name` not found");
}

const WRONG_TYPE_FOR_NAME_FIELD_IN_SCHEMA: &'static str = r#"---
schema:
  - name: 200
"#;

#[test]
fn test_wrong_type_for_field_in_schema() {
    let schema= YamlSchema::from_str(YAML_SCHEMA);
    let yaml: Value =
        serde_yaml::from_str(WRONG_TYPE_FOR_NAME_FIELD_IN_SCHEMA).unwrap();

    let err = schema.validate(&yaml).expect_err("this should fail");
    assert_eq!(
        format!("{}", err),
        "wrong type, expected `string` got `Number(PosInt(200))`"
    );
}

const STRING_LIMIT_SCHEMA: &'static str = r#"---
schema:
  - name: somestring
    type: string
    max_length: 20
    min_length: 10
"#;

const STRING_LIMIT_TOO_SHORT: &'static str = "somestring: hello";
const STRING_LIMIT_TOO_LONG: &'static str = "somestring: hello world how are ya really";
const STRING_LIMIT_JUST_RIGHT: &'static str = "somestring: hello world";

#[test]
fn test_string_limits() {
    let schema= YamlSchema::from_str(STRING_LIMIT_SCHEMA).unwrap();
    let short: Value = serde_yaml::from_str(STRING_LIMIT_TOO_SHORT).unwrap();
    let long: Value = serde_yaml::from_str(STRING_LIMIT_TOO_LONG).unwrap();
    let just_right: Value = serde_yaml::from_str(STRING_LIMIT_JUST_RIGHT).unwrap();

    assert_eq!(
        format!("{}", schema.validate(&long).expect_err("this should fail")),
        "string validation error: string too long, max is 20, but string is 29"
    );

    assert_eq!(
        format!("{}", schema.validate(&short).expect_err("this should fail")),
        "string validation error: string too short, min is 10, but string is 5"
    );

    assert!(schema.validate(&just_right).is_ok());
}

const DICTIONARY_WITH_SET_TYPES_SCHEMA: &'static str = r#"---
schema:
  - name: dict
    type: dictionary
    key:
      type: string
    value:
      number
"#;

const DICTIONARY_WITH_WRONG_TYPES: &'static str = r#"---
dict:
  hello: 10
  world: 20
"#;

#[test]

fn test_dictionary_validation() {
  let schema = YamlSchema::from_str(STRING_LIMIT_SCHEMA);
}