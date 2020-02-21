use crate::error::SchemaError;
use crate::utils::YamlUtils;
use crate::{Context, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[cfg(feature = "regex")]
use crate::error::{optional, SchemaErrorKind};

#[derive(Debug, Default)]
pub(crate) struct SchemaString {
    #[cfg(feature = "regex")]
    pattern: Option<regex::Regex>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaString {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        #[cfg(feature = "regex")]
        {
            yaml.strict_contents(&[], &["type", "pattern"])?;

            let pattern = yaml
                .lookup("pattern", "string", Yaml::as_str)
                .map(Option::from)
                .or_else(optional(None))?
                .map(|inner| {
                    regex::Regex::new(inner).map_err(|e| {
                        SchemaErrorKind::MalformedField {
                            field: "pattern",
                            error: format!("{}", e),
                        }
                        .into()
                    })
                })
                .transpose()?;

            Ok(SchemaString { pattern })
        }

        #[cfg(not(feature = "regex"))]
        {
            yaml.strict_contents(&[], &["type"])?;
            Ok(SchemaString {})
        }
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaString {
    fn validate(
        &self,
        _: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        #[cfg(feature = "regex")]
        {
            let value = yaml.as_type("string", Yaml::as_str)?;

            if let Some(regex) = &self.pattern {
                if !regex.is_match(value) {
                    return Err(SchemaErrorKind::ValidationError {
                        error: "supplied value does not match regex pattern for field",
                    }.into());
                }
            }

            Ok(())
        }

        #[cfg(not(feature = "regex"))]
        {
            yaml.as_type("string", Yaml::as_str).and_then(|_| Ok(()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SchemaErrorKind;
    use crate::utils::load_simple;
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
    #[cfg(feature = "regex")]
    fn with_regex() {
        SchemaString::try_from(&load_simple(
            r#"
                type: string
                pattern: \w+.*
            "#,
        ))
        .unwrap();
    }

    #[test]
    fn validate_string() {
        let schema = SchemaString::default();
        schema
            .validate(&Context::default(), &load_simple("hello world"))
            .unwrap();
    }

    #[test]
    fn validate_integer() {
        let schema = SchemaString::default();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("10"))
                .unwrap_err(),
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
                expected: "string",
                actual: "hash"
            }
            .into()
        );
    }

    #[test]
    #[cfg(feature = "regex")]
    fn validate_regex() {
        let yaml = load_simple(
            r#"
                type: string
                pattern: "[a-zA-Z0-9]+\\[\\]\\d{3}f"
            "#,
        );

        let schema = SchemaString::try_from(&yaml).unwrap();

        schema.validate(&Context::default(), &load_simple("woRd5[]123f")).unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("world"))
                .unwrap_err(),
            SchemaErrorKind::ValidationError {
                error: "supplied value does not match regex pattern for field",
            }
            .into()
        );
    }
}
