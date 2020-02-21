use crate::error::{add_path_name, SchemaError};
use crate::utils::YamlUtils;
use crate::{Context, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[cfg(feature = "regex")]
use crate::error::{optional, SchemaErrorKind};

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
            .and_then(|length| {
                if length < 0 {
                    return Err(SchemaErrorKind::MalformedField {
                        error: "must be a non-negative integer value".into(),
                    }
                    .into());
                }

                usize::try_from(length).map_err(|_| {
                    SchemaErrorKind::MalformedField {
                        error: "value does not fit in a usize on this system".into(),
                    }
                    .into()
                })
            })
            .map_err(add_path_name("minLength"))
            .map(Option::from)
            .or_else(optional(None))?;

        let max_length = yaml
            .lookup("maxLength", "integer", Yaml::as_i64)
            .and_then(|length| {
                if length < 0 {
                    return Err(SchemaErrorKind::MalformedField {
                        error: "must be a non-negative integer value".into(),
                    }
                    .into());
                }

                usize::try_from(length).map_err(|_| {
                    SchemaErrorKind::MalformedField {
                        error: "value does not fit in a usize on this system".into(),
                    }
                    .into()
                })
            })
            .map_err(add_path_name("maxLength"))
            .map(Option::from)
            .or_else(optional(None))?;

        #[cfg(feature = "regex")]
        {
            let pattern = yaml
                .lookup("pattern", "string", Yaml::as_str)
                .map(Option::from)
                .or_else(optional(None))?
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
                min_length,
                max_length,
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
    ) -> Result<(), SchemaError<'yaml>> {
        let value = yaml.as_type("string", Yaml::as_str)?;

        if let Some(min_length) = self.min_length {
            if value.len() < min_length {
                return Err(SchemaErrorKind::ValidationError {
                    error: "string length is less than minLength",
                }
                .into());
            }
        }

        if let Some(max_length) = self.max_length {
            if value.len() > max_length {
                return Err(SchemaErrorKind::ValidationError {
                    error: "string length is greater than maxLength",
                }
                .into());
            }
        }

        #[cfg(feature = "regex")]
        {
            if let Some(regex) = &self.pattern {
                if !regex.is_match(value) {
                    return Err(SchemaErrorKind::ValidationError {
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
    use crate::error::SchemaErrorKind;
    use crate::utils::load_simple;
    use crate::SchemaString;

    #[cfg(feature = "smallvec")]
    use smallvec::smallvec;

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
            .with_path(path!["maxLength"])
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
            .with_path(path!["minLength"])
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

        schema
            .validate(&Context::default(), &load_simple("woRd5[]123f"))
            .unwrap();

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
