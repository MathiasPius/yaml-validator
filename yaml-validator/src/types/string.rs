use crate::errors::{schema::schema_optional, SchemaError, SchemaErrorKind};
use crate::errors::{ValidationError, ValidationErrorKind};
use crate::utils::{try_into_usize, YamlUtils};
use crate::{Context, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug, Default)]
pub(crate) struct SchemaString {
    // The yaml_rust library uses i64 internally, but we cast to usize
    // while building the schema, since we'll need to compare them to
    // string lengths later and we want to fail as early as possible.
    max_length: Option<usize>,
    min_length: Option<usize>,

    #[cfg(feature = "regex")]
    pattern: Option<regex::Regex>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaString {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        #[cfg(feature = "regex")]
        yaml.strict_contents(&[], &["type", "minLength", "maxLength", "pattern"])?;

        #[cfg(not(feature = "regex"))]
        yaml.strict_contents(&[], &["type", "minLength", "maxLength"])?;

        let min_length = yaml
            .lookup("minLength", "integer", Yaml::as_i64)
            .map_err(SchemaErrorKind::from)
            .map_err(SchemaError::from)
            .and_then(try_into_usize)
            .map_err(SchemaError::add_path_name("minLength"))
            .map(Option::from)
            .or_else(schema_optional(None))?;

        let max_length = yaml
            .lookup("maxLength", "integer", Yaml::as_i64)
            .map_err(SchemaErrorKind::from)
            .map_err(SchemaError::from)
            .and_then(try_into_usize)
            .map_err(SchemaError::add_path_name("maxLength"))
            .map(Option::from)
            .or_else(schema_optional(None))?;

        if let (Some(min_length), Some(max_length)) = (min_length, max_length) {
            if min_length > max_length {
                return Err(SchemaErrorKind::MalformedField {
                    error: "minLength cannot be greater than maxLength".into(),
                }
                .into());
            }
        }

        #[cfg(feature = "regex")]
        {
            let pattern = yaml
                .lookup("pattern", "string", Yaml::as_str)
                .map_err(SchemaErrorKind::from)
                .map_err(SchemaError::from)
                .map(Option::from)
                .or_else(schema_optional(None))?
                .map(|inner| {
                    regex::Regex::new(inner).map_err(|e| {
                        SchemaErrorKind::MalformedField {
                            error: format!("{}", e),
                        }
                        .with_path_name("pattern")
                    })
                })
                .transpose()?;

            Ok(SchemaString {
                max_length,
                min_length,
                pattern,
            })
        }

        #[cfg(not(feature = "regex"))]
        Ok(SchemaString {
            min_length,
            max_length,
        })
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaString {
    fn validate(
        &self,
        _: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), ValidationError<'yaml>> {
        let value = yaml.as_type("string", Yaml::as_str)?;

        if let Some(min_length) = self.min_length {
            if value.len() < min_length {
                return Err(ValidationErrorKind::ValidationError {
                    error: "string length is less than minLength",
                }
                .into());
            }
        }

        if let Some(max_length) = self.max_length {
            if value.len() > max_length {
                return Err(ValidationErrorKind::ValidationError {
                    error: "string length is greater than maxLength",
                }
                .into());
            }
        }

        #[cfg(feature = "regex")]
        {
            if let Some(regex) = &self.pattern {
                if !regex.is_match(value) {
                    return Err(ValidationErrorKind::ValidationError {
                        error: "supplied value does not match regex pattern for field",
                    }
                    .into());
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::SchemaErrorKind;
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
    fn with_malformed_max_length() {
        assert_eq!(
            SchemaString::try_from(&load_simple(
                r#"
                type: string
                maxLength: -5
            "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::MalformedField {
                error: "must be a non-negative integer value".into()
            }
            .with_path_name("maxLength")
        );
    }

    #[test]
    fn with_malformed_min_length() {
        assert_eq!(
            SchemaString::try_from(&load_simple(
                r#"
                type: string
                minLength: 205.02
            "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::WrongType {
                actual: "real",
                expected: "integer"
            }
            .with_path_name("minLength")
        );
    }

    #[test]
    fn with_min_and_max_length() {
        SchemaString::try_from(&load_simple(
            r#"
                type: string
                minLength: 10
                maxLength: 20
            "#,
        ))
        .unwrap();
    }

    #[test]
    fn with_min_larger_than_max_length() {
        assert_eq!(
            SchemaString::try_from(&load_simple(
                r#"
                type: string
                minLength: 20
                maxLength: 10
            "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::MalformedField {
                error: "minLength cannot be greater than maxLength".into()
            }
            .into()
        );
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
            ValidationErrorKind::WrongType {
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
            ValidationErrorKind::WrongType {
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
                .validate(&Context::default(), &load_simple("hello: world"))
                .unwrap_err(),
            ValidationErrorKind::WrongType {
                expected: "string",
                actual: "hash"
            }
            .into()
        );
    }

    #[test]
    fn validate_min_and_max_length() {
        let schema = SchemaString::try_from(&load_simple(
            r#"
            type: string
            minLength: 10
            maxLength: 20
        "#,
        ))
        .unwrap();

        schema
            .validate(&Context::default(), &load_simple("hello world"))
            .unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("hello"))
                .unwrap_err(),
            ValidationErrorKind::ValidationError {
                error: "string length is less than minLength"
            }
            .into()
        );

        assert_eq!(
            schema
                .validate(
                    &Context::default(),
                    &load_simple("hello woooooooooooooooorld!")
                )
                .unwrap_err(),
            ValidationErrorKind::ValidationError {
                error: "string length is greater than maxLength"
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

        schema
            .validate(&Context::default(), &load_simple("woRd5[]123f"))
            .unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("world"))
                .unwrap_err(),
            ValidationErrorKind::ValidationError {
                error: "supplied value does not match regex pattern for field",
            }
            .into()
        );
    }
}
