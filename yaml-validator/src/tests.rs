use super::{YamlSchema, YamlContext};

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
    let _rd = YamlSchema::from_str(DIFFERENT_TYPES);
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
"#;

#[test]
fn validate_yaml_schema() {
    let schema = YamlSchema::from_str(YAML_SCHEMA);

    schema.validate_str(&YAML_SCHEMA, None).unwrap();
}

const MISSING_NAME_FIELD_IN_SCHEMA: &'static str = r#"---
schema:
  - hello: world
"#;

#[test]
fn test_missing_fields_in_schema() {
    let schema = YamlSchema::from_str(YAML_SCHEMA);

    let err = schema
        .validate_str(&MISSING_NAME_FIELD_IN_SCHEMA, None)
        .expect_err("this should fail");
    assert_eq!(
        format!("{}", err),
        "$.schema[0]: missing field, `name` not found"
    );
}

const WRONG_TYPE_FOR_NAME_FIELD_IN_SCHEMA: &'static str = r#"---
schema:
  - name: 200
"#;

#[test]
fn test_wrong_type_for_field_in_schema() {
    let schema = YamlSchema::from_str(YAML_SCHEMA);

    let err = schema
        .validate_str(&WRONG_TYPE_FOR_NAME_FIELD_IN_SCHEMA, None)
        .expect_err("this should fail");
    assert_eq!(
        format!("{}", err),
        "$.schema[0].name: wrong type, expected `string` got `Number(PosInt(200))`"
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
    let schema = YamlSchema::from_str(STRING_LIMIT_SCHEMA);

    assert_eq!(
        format!(
            "{}",
            schema
                .validate_str(&STRING_LIMIT_TOO_LONG, None)
                .expect_err("this should fail")
        ),
        "$.somestring: string validation error: string too long, max is 20, but string is 29"
    );

    assert_eq!(
        format!(
            "{}",
            schema
                .validate_str(&STRING_LIMIT_TOO_SHORT, None)
                .expect_err("this should fail")
        ),
        "$.somestring: string validation error: string too short, min is 10, but string is 5"
    );

    assert!(schema.validate_str(STRING_LIMIT_JUST_RIGHT, None).is_ok());
}

const DICTIONARY_WITH_SET_TYPES_SCHEMA: &'static str = r#"---
schema:
  - name: dict
    type: dictionary
    key:
      type: string
    value:
      type: number
"#;

const DICTIONARY_WITH_CORRECT_TYPES: &'static str = r#"---
dict:
  hello: 10
  world: 20
"#;

const DICTIONARY_WITH_WRONG_TYPES: &'static str = r#"---
dict:
  hello: world
  world: hello
"#;

#[test]
fn test_dictionary_validation() {
    let schema = YamlSchema::from_str(DICTIONARY_WITH_SET_TYPES_SCHEMA);

    assert!(schema.validate_str(&DICTIONARY_WITH_CORRECT_TYPES, None).is_ok());
    assert_eq!(
        format!(
            "{}",
            schema
                .validate_str(&DICTIONARY_WITH_WRONG_TYPES, None)
                .expect_err("this should fail")
        ),
        "$.dict.hello: wrong type, expected `number` got `String(\"world\")`"
    );
}


const SCHEMA_WITH_URI: &'static str = r#"---
uri: myuri/v1
schema:
  - name: testproperty
    type: number
"#;

const SCHEMA_WITH_REFERENCE: &'static str = r#"---
schema:
  - name: propref
    type: reference
    uri: myuri/v1
"#;

const YAML_FILE_WITH_REFERENCE: &'static str = r#"---
propref:
  testproperty: 10
"#;

#[test]
fn test_schema_reference() {
    let context = YamlContext::from_schemas(vec![
        YamlSchema::from_str(SCHEMA_WITH_URI),
    ]);

    let schema = YamlSchema::from_str(SCHEMA_WITH_REFERENCE);
    schema.validate_str(&YAML_FILE_WITH_REFERENCE, Some(&context)).unwrap();
}