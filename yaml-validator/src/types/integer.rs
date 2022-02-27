use crate::errors::{schema::schema_optional, SchemaError, SchemaErrorKind};
use crate::errors::{ValidationError, ValidationErrorKind};
use crate::utils::{Limit, YamlUtils};
use crate::{Context, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug, Default)]
pub(crate) struct SchemaInteger {
    minimum: Option<Limit<i64>>,
    maximum: Option<Limit<i64>>,
    multiple_of: Option<i64>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaInteger {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(
            &[],
            &[
                "type",
                "minimum",
                "exclusiveMinimum",
                "maximum",
                "exclusiveMaximum",
                "multipleOf",
            ],
        )
        .map_err(SchemaErrorKind::from)?;

        yaml.check_exclusive_fields(&["minimum", "exclusiveMinimum"])?;
        yaml.check_exclusive_fields(&["maximum", "exclusiveMaximum"])?;

        let minimum = yaml
            .lookup("minimum", "integer", Yaml::as_i64)
            .map_err(SchemaErrorKind::from)
            .map_err(SchemaError::from)
            .map(Limit::Inclusive)
            .map(Option::from)
            .or_else(schema_optional(None))?
            .or(yaml
                .lookup("exclusiveMinimum", "integer", Yaml::as_i64)
                .map_err(SchemaErrorKind::from)
                .map_err(SchemaError::from)
                .map(Limit::Exclusive)
                .map(Option::from)
                .or_else(schema_optional(None))?);

        let maximum = yaml
            .lookup("maximum", "integer", Yaml::as_i64)
            .map_err(SchemaErrorKind::from)
            .map_err(SchemaError::from)
            .map(Limit::Inclusive)
            .map(Option::from)
            .or_else(schema_optional(None))?
            .or(yaml
                .lookup("exclusiveMaximum", "integer", Yaml::as_i64)
                .map_err(SchemaErrorKind::from)
                .map_err(SchemaError::from)
                .map(Limit::Exclusive)
                .map(Option::from)
                .or_else(schema_optional(None))?);

        if let (Some(lower), Some(upper)) = (&minimum, &maximum) {
            if !lower.has_span(&upper) {
                return Err(SchemaErrorKind::MalformedField {
                    error: "range given for real value spans 0 possible values".into(),
                }
                .into());
            }
        }

        let multiple_of = yaml
            .lookup("multipleOf", "integer", Yaml::as_i64)
            .map_err(SchemaErrorKind::from)
            .map_err(SchemaError::from)
            .and_then(|number| {
                if number <= 0 {
                    Err(SchemaErrorKind::MalformedField {
                        error: "must be greater than zero".into(),
                    }
                    .with_path_name("multipleOf"))
                } else {
                    Ok(number)
                }
            })
            .map(Option::from)
            .or_else(schema_optional(None))?;

        Ok(SchemaInteger {
            minimum,
            maximum,
            multiple_of,
        })
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaInteger {
    fn validate(
        &self,
        _: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), ValidationError<'yaml>> {
        let value = yaml
            .as_type("integer", Yaml::as_i64)
            .map_err(ValidationErrorKind::from)?;

        if let Some(minimum) = &self.minimum {
            if !minimum.is_greater(&value) {
                return Err(ValidationErrorKind::ValidationError {
                    error: "value violates lower limit constraint",
                }
                .into());
            }
        }

        if let Some(maximum) = &self.maximum {
            if !maximum.is_lesser(&value) {
                return Err(ValidationErrorKind::ValidationError {
                    error: "value violates upper limit constraint",
                }
                .into());
            }
        }

        if let Some(multiple_of) = &self.multiple_of {
            if value.rem_euclid(*multiple_of) != 0 {
                return Err(ValidationErrorKind::ValidationError {
                    error: "value must be a multiple of the multipleOf field",
                }
                .into());
            }
        }

        Ok(())
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
    fn with_exclusive_conflict() {
        assert_eq!(
            SchemaInteger::try_from(&load_simple(
                r#"
                type: integer
                exclusiveMinimum: 10
                minimum: 10
            "#)).unwrap_err(),
            SchemaErrorKind::MalformedField {
                error: "conflicting constraints: minimum, exclusiveMinimum cannot be used at the same time".into()
            }.into()
        )
    }

    #[test]
    fn with_mixed_limits() {
        SchemaInteger::try_from(&load_simple(
            r#"
                type: integer
                exclusiveMaximum: 12
                minimum: 10
            "#,
        ))
        .unwrap();
    }

    #[test]
    fn with_zero_width_limits() {
        assert_eq!(
            SchemaInteger::try_from(&load_simple(
                r#"
                type: real
                exclusiveMinimum: 10
                exclusiveMaximum: 11
            "#
            ))
            .unwrap_err(),
            SchemaErrorKind::MalformedField {
                error: "range given for real value spans 0 possible values".into()
            }
            .into()
        )
    }

    #[test]
    fn validate_string() {
        let schema = SchemaInteger::default();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("hello world"))
                .unwrap_err(),
            ValidationErrorKind::WrongType {
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
            ValidationErrorKind::WrongType {
                expected: "integer",
                actual: "real"
            }
            .into()
        );
    }

    #[test]
    fn validate_narrow_inclusive_set() {
        let schema = SchemaInteger::try_from(&load_simple(
            r#"
                type: integer
                minimum: 10
                maximum: 10
            "#,
        ))
        .unwrap();

        schema
            .validate(&Context::default(), &load_simple("10"))
            .unwrap();
    }

    #[test]
    fn validate_narrow_set() {
        let schema = SchemaInteger::try_from(&load_simple(
            r#"
                type: integer
                exclusiveMinimum: 10
                maximum: 11
            "#,
        ))
        .unwrap();

        schema
            .validate(&Context::default(), &load_simple("11"))
            .unwrap();
    }

    #[test]
    fn validate_min_exclusion() {
        let schema = SchemaInteger::try_from(&load_simple(
            r#"
                type: integer
                exclusiveMinimum: 10
            "#,
        ))
        .unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("10"))
                .unwrap_err(),
            ValidationErrorKind::ValidationError {
                error: "value violates lower limit constraint".into()
            }
            .into()
        );
    }

    #[test]
    fn validate_min_valid() {
        let schema = SchemaInteger::try_from(&load_simple(
            r#"
                type: real
                minimum: 10
            "#,
        ))
        .unwrap();

        schema
            .validate(&Context::default(), &load_simple("10"))
            .unwrap();
    }

    #[test]
    fn validate_min_invalid() {
        let schema = SchemaInteger::try_from(&load_simple(
            r#"
                type: integer
                minimum: 10
            "#,
        ))
        .unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("5"))
                .unwrap_err(),
            ValidationErrorKind::ValidationError {
                error: "value violates lower limit constraint".into()
            }
            .into()
        );
    }

    #[test]
    fn validate_max_exclusion() {
        let schema = SchemaInteger::try_from(&load_simple(
            r#"
                type: integer
                exclusiveMaximum: 10
            "#,
        ))
        .unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("10"))
                .unwrap_err(),
            ValidationErrorKind::ValidationError {
                error: "value violates upper limit constraint".into()
            }
            .into()
        );
    }

    #[test]
    fn validate_max_valid() {
        let schema = SchemaInteger::try_from(&load_simple(
            r#"
                type: integer
                maximum: 10
            "#,
        ))
        .unwrap();

        schema
            .validate(&Context::default(), &load_simple("10"))
            .unwrap();
    }

    #[test]
    fn validate_max_invalid() {
        let schema = SchemaInteger::try_from(&load_simple(
            r#"
                type: integer
                maximum: 10
            "#,
        ))
        .unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("20"))
                .unwrap_err(),
            ValidationErrorKind::ValidationError {
                error: "value violates upper limit constraint".into()
            }
            .into()
        );
    }

    #[test]
    fn validate_multiple_of() {
        let schema = SchemaInteger::try_from(&load_simple(
            r#"
                type: integer
                multipleOf: 3
            "#,
        ))
        .unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("10"))
                .unwrap_err(),
            ValidationErrorKind::ValidationError {
                error: "value must be a multiple of the multipleOf field"
            }
            .into()
        );
    }

    #[test]
    fn validate_multiple_of_success() {
        let schema = SchemaInteger::try_from(&load_simple(
            r#"
                type: integer
                multipleOf: 325
            "#,
        ))
        .unwrap();

        schema
            .validate(&Context::default(), &load_simple("4875"))
            .unwrap();
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
            ValidationErrorKind::WrongType {
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
            ValidationErrorKind::WrongType {
                expected: "integer",
                actual: "hash"
            }
            .into()
        );
    }
}
