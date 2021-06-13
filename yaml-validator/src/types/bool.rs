use crate::error::{add_path_name, optional, SchemaError, SchemaErrorKind};
use crate::utils::{try_into_usize, YamlUtils};
use crate::{Context, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug, Default)]
pub(crate) struct SchemaBool {}

impl<'schema> TryFrom<&'schema Yaml> for SchemaBool {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&[], &["type"])?;
        Ok(SchemaBool {})
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaBool {
    fn validate(
        &self,
        _: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        let _value = yaml.as_type("bool", Yaml::as_bool)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SchemaErrorKind;
    use crate::types::SchemaInteger;
    use crate::utils::load_simple;
    use crate::SchemaString;

    #[test]
    fn from_yaml() {
        SchemaBool::try_from(&load_simple("type: bool")).unwrap();
    }

    #[test]
    fn from_string() {
        assert_eq!(
            SchemaString::try_from(&load_simple("true")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "boolean"
            }
            .into()
        );
    }

    #[test]
    fn from_integer() {
        assert_eq!(
            SchemaInteger::try_from(&load_simple("true")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "boolean"
            }
            .into()
        );
    }

    #[test]
    fn from_array() {
        assert_eq!(
            SchemaBool::try_from(&load_simple(
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
        let schema = SchemaBool::default();
        schema
            .validate(&Context::default(), &load_simple("true"))
            .unwrap();
    }

    #[test]
    fn validate_integer() {
        let schema = SchemaBool::default();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("10"))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "bool",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn validate_array() {
        let schema = SchemaBool::default();

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
                expected: "bool",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_hash() {
        let schema = SchemaBool::default();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("hello: true"))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "bool",
                actual: "hash"
            }
            .into()
        );
    }
}
