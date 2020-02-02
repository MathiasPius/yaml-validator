use crate::error::{PathSegment, SchemaErrorKind};
use crate::Validate;
use std::convert::TryFrom;
use yaml_rust::{Yaml, YamlLoader};

fn load_simple(source: &'static str) -> Yaml {
    YamlLoader::load_from_str(source).unwrap().remove(0)
}

mod errors {
    use super::*;
    use crate::*;
    #[test]
    fn test_error_path() {
        let yaml = load_simple(
            r#"
            items:
              test:
                type: integer
              something:
                type: object
                items:
                  level2:
                    type: object
                    items:
                      leaf: hello
            "#,
        );

        let err = SchemaObject::try_from(&yaml).unwrap_err();

        debug_assert_eq!(
            format!("{}", err),
            "#.items.something.items.level2.items.leaf: field \'type\' missing\n",
        );
    }

    #[test]
    fn test_error_path_validation() {
        let yaml = load_simple(
            r#"
            items:
              test:
                type: integer
              something:
                type: object
                items:
                  level2:
                    type: array
                    items:
                      type: object
                      items:
                        num:
                          type: integer
            "#,
        );

        let schema = SchemaObject::try_from(&yaml).unwrap();
        let document = load_simple(
            r#"
            test: 20
            something:
              level2:
                - num: abc
                - num:
                    hash: value
                - num:
                    - array: hello
                - num: 10
                - num: jkl
            "#,
        );

        let err = schema.validate(&document).unwrap_err();

        debug_assert_eq!(
            format!("{}", err),
            r#"#.something.level2[0].num: wrong type, expected integer got string
#.something.level2[1].num: wrong type, expected integer got hash
#.something.level2[2].num: wrong type, expected integer got array
#.something.level2[4].num: wrong type, expected integer got string
"#
        );
    }
}

mod schemaobject {
    use super::*;
    use crate::SchemaObject;
    #[test]
    fn from_yaml() {
        SchemaObject::try_from(&load_simple(
            r#"
            items:
              something:
                type: string
        "#,
        ))
        .unwrap();
    }

    #[test]
    fn extra_fields() {
        debug_assert_eq!(
            SchemaObject::try_from(&load_simple(
                r#"
            items:
              something:
                type: hello
            extra: extra field test
        "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::ExtraField { field: "extra" }.into(),
        );
    }

    #[test]
    fn malformed_items() {
        debug_assert_eq!(
            SchemaObject::try_from(&load_simple(
                r#"
            items:
              hello: world
        "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::FieldMissing { field: "type" }
                .with_path(vec![PathSegment::Name("hello"), PathSegment::Name("items")]),
        );
    }

    #[test]
    fn multiple_errors() {
        debug_assert_eq!(
            SchemaObject::try_from(&load_simple(
                r#"
            items:
              valid:
                type: string
              error 1:
                type: unknown1
              error 2:
                type: unknown2
        "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::Multiple {
                errors: vec![
                    SchemaErrorKind::UnknownType {
                        unknown_type: "unknown1"
                    }
                    .with_path(vec![
                        PathSegment::Name("error 1"),
                        PathSegment::Name("items"),
                    ]),
                    SchemaErrorKind::UnknownType {
                        unknown_type: "unknown2"
                    }
                    .with_path(vec![
                        PathSegment::Name("error 2"),
                        PathSegment::Name("items"),
                    ]),
                ]
            }
            .into()
        );
    }

    #[test]
    fn from_string() {
        debug_assert_eq!(
            SchemaObject::try_from(&load_simple("world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn from_integer() {
        debug_assert_eq!(
            SchemaObject::try_from(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn from_array() {
        debug_assert_eq!(
            SchemaObject::try_from(&load_simple(
                r#"
                - hello
                - world
            "#
            ))
            .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_string() {
        let schema = SchemaObject::default();

        debug_assert_eq!(
            schema.validate(&load_simple("hello world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn validate_integer() {
        let schema = SchemaObject::default();

        debug_assert_eq!(
            schema.validate(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn validate_array() {
        let schema = SchemaObject::default();

        debug_assert_eq!(
            schema
                .validate(&load_simple(
                    r#"
                - abc
                - 123
            "#
                ))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_hash() {
        let yaml = load_simple(
            r#"
            items:
              hello:
                type: string
              world:
                type: integer
            "#,
        );

        let schema = SchemaObject::try_from(&yaml).unwrap();

        schema
            .validate(&load_simple(
                r#"
            hello: world
            world: 20
        "#,
            ))
            .unwrap();
    }

    #[test]
    fn validate_noncompliant() {
        let yaml = load_simple(
            r#"
            items:
              hello:
                type: string
              world:
                type: integer
            "#,
        );

        let schema = SchemaObject::try_from(&yaml).unwrap();

        debug_assert_eq!(
            schema
                .validate(&load_simple(
                    r#"
            hello: 20
            world: world
        "#,
                ))
                .unwrap_err(),
            SchemaErrorKind::Multiple {
                errors: vec![
                    SchemaErrorKind::WrongType {
                        expected: "string",
                        actual: "integer"
                    }
                    .with_path(vec![PathSegment::Name("hello")]),
                    SchemaErrorKind::WrongType {
                        expected: "integer",
                        actual: "string"
                    }
                    .with_path(vec![PathSegment::Name("world")])
                ]
            }
            .into()
        );
    }
}

mod schemaarray {
    use super::*;
    use crate::SchemaArray;
    #[test]
    fn from_yaml() {
        SchemaArray::try_from(&load_simple(
            r#"
            items:
              type: string
        "#,
        ))
        .unwrap();
    }

    #[test]
    fn malformed_items() {
        debug_assert_eq!(
            SchemaArray::try_from(&load_simple(
                r#"
            items:
              - type: string
        "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "array"
            }
            .with_path(vec![PathSegment::Name("items")])
            .into(),
        );
    }

    #[test]
    fn from_string() {
        debug_assert_eq!(
            SchemaArray::try_from(&load_simple("world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn from_integer() {
        debug_assert_eq!(
            SchemaArray::try_from(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn from_array() {
        debug_assert_eq!(
            SchemaArray::try_from(&load_simple(
                r#"
                - hello
                - world
            "#
            ))
            .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_string() {
        let schema = SchemaArray::default();

        debug_assert_eq!(
            schema.validate(&load_simple("hello world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "array",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn validate_integer() {
        let schema = SchemaArray::default();

        debug_assert_eq!(
            schema.validate(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "array",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn validate_untyped_array() {
        SchemaArray::default()
            .validate(&load_simple(
                r#"
                - abc
                - 123
            "#,
            ))
            .unwrap();
    }

    #[test]
    fn validate_typed_array() {
        let yaml = load_simple(
            r#"
        items:
          type: integer
        "#,
        );

        debug_assert_eq!(
            SchemaArray::try_from(&yaml)
                .unwrap()
                .validate(&load_simple(
                    r#"
                - abc
                - 1
                - 2
                - 3
                - def
                - 4
                - hello: world
            "#,
                ))
                .unwrap_err(),
            SchemaErrorKind::Multiple {
                errors: vec![
                    SchemaErrorKind::WrongType {
                        expected: "integer",
                        actual: "string"
                    }
                    .with_path(vec![PathSegment::Index(0)]),
                    SchemaErrorKind::WrongType {
                        expected: "integer",
                        actual: "string"
                    }
                    .with_path(vec![PathSegment::Index(4)]),
                    SchemaErrorKind::WrongType {
                        expected: "integer",
                        actual: "hash"
                    }
                    .with_path(vec![PathSegment::Index(6)])
                ]
            }
            .into()
        );
    }

    #[test]
    fn validate_hash() {
        let schema = SchemaArray::default();

        debug_assert_eq!(
            schema.validate(&load_simple("hello: world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "array",
                actual: "hash"
            }
            .into()
        );
    }
}

mod schemastring {
    use super::*;
    use crate::SchemaString;
    #[test]
    fn from_yaml() {
        SchemaString::try_from(&load_simple("type: string")).unwrap();
    }

    #[test]
    fn from_string() {
        debug_assert_eq!(
            SchemaString::try_from(&load_simple("world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn from_integer() {
        debug_assert_eq!(
            SchemaString::try_from(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn from_array() {
        debug_assert_eq!(
            SchemaString::try_from(&load_simple(
                r#"
                - hello
                - world
            "#
            ))
            .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_string() {
        let schema = SchemaString::default();
        schema.validate(&load_simple("hello world")).unwrap();
    }

    #[test]
    fn validate_integer() {
        let schema = SchemaString::default();

        debug_assert_eq!(
            schema.validate(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "string",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn validate_array() {
        let schema = SchemaString::default();

        debug_assert_eq!(
            schema
                .validate(&load_simple(
                    r#"
                - abc
                - 123
            "#
                ))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "string",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_hash() {
        let schema = SchemaString::default();

        debug_assert_eq!(
            schema
                .validate(&load_simple(
                    r#"
                hello: world
            "#
                ))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "string",
                actual: "hash"
            }
            .into()
        );
    }
}

mod schemainteger {
    use super::*;
    use crate::SchemaInteger;
    #[test]
    fn from_yaml() {
        SchemaInteger::try_from(&load_simple("type: string")).unwrap();
    }

    #[test]
    fn from_string() {
        debug_assert_eq!(
            SchemaInteger::try_from(&load_simple("world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn from_integer() {
        debug_assert_eq!(
            SchemaInteger::try_from(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn from_array() {
        debug_assert_eq!(
            SchemaInteger::try_from(&load_simple(
                r#"
                - hello
                - world
            "#
            ))
            .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_string() {
        let schema = SchemaInteger::default();

        debug_assert_eq!(
            schema.validate(&load_simple("hello world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "integer",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn validate_integer() {
        let schema = SchemaInteger::default();

        schema.validate(&load_simple("10")).unwrap();
    }

    #[test]
    fn validate_array() {
        let schema = SchemaInteger::default();

        debug_assert_eq!(
            schema
                .validate(&load_simple(
                    r#"
                - abc
                - 123
            "#
                ))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "integer",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_hash() {
        let schema = SchemaInteger::default();

        debug_assert_eq!(
            schema
                .validate(&load_simple(
                    r#"
                hello: world
            "#
                ))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "integer",
                actual: "hash"
            }
            .into()
        );
    }
}
