use crate::error::SchemaError;
use crate::utils::YamlUtils;
use crate::{Context, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug, Default)]
pub(crate) struct SchemaInteger {}

impl<'schema> TryFrom<&'schema Yaml> for SchemaInteger {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&[], &["type"])?;

        Ok(SchemaInteger {})
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaInteger {
    fn validate(
        &self,
        _: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        yaml.as_type("integer", Yaml::as_i64).and_then(|_| Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::load_simple;
    use crate::SchemaErrorKind;

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
            schema
                .validate(&Context::default(), &load_simple("hello world"))
                .unwrap_err(),
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

        schema
            .validate(&Context::default(), &load_simple("10"))
            .unwrap();
    }

    #[test]
    fn validate_real() {
        let schema = SchemaInteger::default();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("3.1415"))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "integer",
                actual: "real"
            }
            .into()
        );
    }

    #[test]
    fn validate_array() {
        let schema = SchemaInteger::default();

        assert_eq!(
            schema
                .validate(
                    &Context::default(),
                    &load_simple(
                        r#"
                - abc
                - 123
            "#
                    )
                )
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
                .validate(
                    &Context::default(),
                    &load_simple(
                        r#"
                hello: world
            "#
                    )
                )
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "integer",
                actual: "hash"
            }
            .into()
        );
    }
}
