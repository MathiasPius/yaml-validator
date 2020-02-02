use crate::error::{PathSegment, SchemaErrorKind};
use crate::Validate;
use std::convert::TryFrom;
use yaml_rust::{Yaml, YamlLoader};

fn load_simple(source: &'static str) -> Yaml {
    YamlLoader::load_from_str(source).unwrap().remove(0)
}

mod schemaobject {
    use super::*;
    use crate::SchemaObject;
    #[test]
    fn from_yaml() {
        SchemaObject::try_from(&load_simple(
            r#"
            items:
              - name: something
                type: string
        "#,
        ))
        .unwrap();
    }

    #[test]
    fn extra_fields() {
        assert_eq!(
            SchemaObject::try_from(&load_simple(
                r#"
            items:
              - name: something
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
        assert_eq!(
            SchemaObject::try_from(&load_simple(
                r#"
            items:
              hello: world
        "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "array",
                actual: "hash"
            }
            .into(),
        );
    }

    #[test]
    fn multiple_errors() {
        assert_eq!(
            SchemaObject::try_from(&load_simple(
                r#"
            items:
              - name: valid
                type: string
              - name: error 1
                type: unknown1
              - name: error 2
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
                        PathSegment::Name("items")
                    ]),
                    SchemaErrorKind::UnknownType {
                        unknown_type: "unknown2"
                    }
                    .with_path(vec![
                        PathSegment::Name("error 2"),
                        PathSegment::Name("items")
                    ]),
                ]
            }
            .into()
        );
    }

    #[test]
    fn from_string() {
        assert_eq!(
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
        assert_eq!(
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
        assert_eq!(
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

        assert_eq!(
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

        assert_eq!(
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

        assert_eq!(
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
              - name: hello
                type: string
              - name: world
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
              - name: hello
                type: string
              - name: world
                type: integer
            "#,
        );

        let schema = SchemaObject::try_from(&yaml).unwrap();

        assert_eq!(
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
        assert_eq!(
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
        assert_eq!(
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
        assert_eq!(
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
        assert_eq!(
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

        assert_eq!(
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

        assert_eq!(
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

        assert_eq!(
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

        assert_eq!(
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
        assert_eq!(
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
        assert_eq!(
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
        assert_eq!(
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

        assert_eq!(
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

        assert_eq!(
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

        assert_eq!(
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
        assert_eq!(
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
        assert_eq!(
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
        assert_eq!(
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

        assert_eq!(
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

        assert_eq!(
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

        assert_eq!(
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
