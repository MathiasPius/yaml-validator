use crate::error::{optional, SchemaError, SchemaErrorKind};
use crate::utils::{Limit, YamlUtils};
use crate::{Context, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug, Default)]
pub(crate) struct SchemaReal {
    minimum: Option<Limit<f64>>,
    maximum: Option<Limit<f64>>,
    multiple_of: Option<f64>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaReal {
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
        )?;

        yaml.check_exclusive_fields(&["minimum", "exclusiveMinimum"])?;
        yaml.check_exclusive_fields(&["maximum", "exclusiveMaximum"])?;

        let minimum = yaml
            .lookup("minimum", "real", Yaml::as_f64)
            .map(Limit::Inclusive)
            .map(Option::from)
            .or_else(optional(None))?
            .or(yaml
                .lookup("exclusiveMinimum", "real", Yaml::as_f64)
                .map(Limit::Exclusive)
                .map(Option::from)
                .or_else(optional(None))?);

        let maximum = yaml
            .lookup("maximum", "real", Yaml::as_f64)
            .map(Limit::Inclusive)
            .map(Option::from)
            .or_else(optional(None))?
            .or(yaml
                .lookup("exclusiveMaximum", "real", Yaml::as_f64)
                .map(Limit::Exclusive)
                .map(Option::from)
                .or_else(optional(None))?);

        let multiple_of = yaml
            .lookup("multipleOf", "real", Yaml::as_f64)
            .and_then(|number| {
                if number <= 0.0 {
                    Err(SchemaErrorKind::MalformedField {
                        error: "must be greater than zero".into(),
                    }
                    .with_path_name("multipleOf"))
                } else {
                    Ok(number)
                }
            })
            .map(Option::from)
            .or_else(optional(None))?;

        if let (Some(lower), Some(upper)) = (&minimum, &maximum) {
            if !lower.has_span(&upper) {
                return Err(SchemaErrorKind::MalformedField {
                    error: "range given for real value spans 0 possible values".into(),
                }
                .into());
            }
        }

        Ok(SchemaReal {
            minimum,
            maximum,
            multiple_of,
        })
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaReal {
    fn validate(
        &self,
        _: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        let value = yaml.as_type("real", Yaml::as_f64)?;

        if let Some(minimum) = &self.minimum {
            if !minimum.is_greater(&value) {
                return Err(SchemaErrorKind::ValidationError {
                    error: "value violates lower limit constraint",
                }
                .into());
            }
        }

        if let Some(maximum) = &self.maximum {
            if !maximum.is_lesser(&value) {
                return Err(SchemaErrorKind::ValidationError {
                    error: "value violates upper limit constraint",
                }
                .into());
            }
        }

        if let Some(multiple_of) = &self.multiple_of {
            if value.rem_euclid(*multiple_of) != 0.0 {
                return Err(SchemaErrorKind::ValidationError {
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
        SchemaReal::try_from(&load_simple("type: real")).unwrap();
    }

    #[test]
    fn from_string() {
        assert_eq!(
            SchemaReal::try_from(&load_simple("world")).unwrap_err(),
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
            SchemaReal::try_from(&load_simple("10")).unwrap_err(),
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
            SchemaReal::try_from(&load_simple(
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
            SchemaReal::try_from(&load_simple(
                r#"
                type: real
                exclusiveMinimum: 10.0
                minimum: 10.0
            "#)).unwrap_err(),
            SchemaErrorKind::MalformedField {
                error: "conflicting constraints: minimum, exclusiveMinimum cannot be used at the same time".into()
            }.into()
        )
    }

    #[test]
    fn with_mixed_limits() {
        SchemaReal::try_from(&load_simple(
            r#"
                type: real
                exclusiveMaximum: 12.0
                minimum: 10.0
            "#,
        ))
        .unwrap();
    }

    #[test]
    fn with_zero_width_limits() {
        assert_eq!(
            SchemaReal::try_from(&load_simple(
                r#"
                type: real
                exclusiveMinimum: 10.0
                exclusiveMaximum: 10.0
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
        let schema = SchemaReal::default();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("hello world"))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "real",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn validate_integer() {
        let schema = SchemaReal::default();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("10"))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "real",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn validate_real() {
        let schema = SchemaReal::default();

        schema
            .validate(&Context::default(), &load_simple("3.1415"))
            .unwrap();
    }

    #[test]
    fn validate_narrow_inclusive_set() {
        let schema = SchemaReal::try_from(&load_simple(
            r#"
                type: real
                minimum: 10.0
                maximum: 10.0
            "#,
        ))
        .unwrap();

        schema
            .validate(&Context::default(), &load_simple("10.0"))
            .unwrap();
    }

    #[test]
    fn validate_narrow_set() {
        let schema = SchemaReal::try_from(&load_simple(
            r#"
                type: real
                exclusiveMinimum: 10.0
                maximum: 11.0
            "#,
        ))
        .unwrap();

        schema
            .validate(&Context::default(), &load_simple("10.5"))
            .unwrap();
    }

    #[test]
    fn validate_min_exclusion() {
        let schema = SchemaReal::try_from(&load_simple(
            r#"
                type: real
                exclusiveMinimum: 10.0
            "#,
        ))
        .unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("10.0"))
                .unwrap_err(),
            SchemaErrorKind::ValidationError {
                error: "value violates lower limit constraint".into()
            }
            .into()
        );
    }

    #[test]
    fn validate_min_valid() {
        let schema = SchemaReal::try_from(&load_simple(
            r#"
                type: real
                minimum: 10.0
            "#,
        ))
        .unwrap();

        schema
            .validate(&Context::default(), &load_simple("10.0"))
            .unwrap();
    }

    #[test]
    fn validate_min_invalid() {
        let schema = SchemaReal::try_from(&load_simple(
            r#"
                type: real
                minimum: 10.0
            "#,
        ))
        .unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("5.0"))
                .unwrap_err(),
            SchemaErrorKind::ValidationError {
                error: "value violates lower limit constraint".into()
            }
            .into()
        );
    }

    #[test]
    fn validate_max_exclusion() {
        let schema = SchemaReal::try_from(&load_simple(
            r#"
                type: real
                exclusiveMaximum: 10.0
            "#,
        ))
        .unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("10.0"))
                .unwrap_err(),
            SchemaErrorKind::ValidationError {
                error: "value violates upper limit constraint".into()
            }
            .into()
        );
    }

    #[test]
    fn validate_max_valid() {
        let schema = SchemaReal::try_from(&load_simple(
            r#"
                type: real
                maximum: 10.0
            "#,
        ))
        .unwrap();

        schema
            .validate(&Context::default(), &load_simple("10.0"))
            .unwrap();
    }

    #[test]
    fn validate_max_invalid() {
        let schema = SchemaReal::try_from(&load_simple(
            r#"
                type: real
                maximum: 10.0
            "#,
        ))
        .unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("20.0"))
                .unwrap_err(),
            SchemaErrorKind::ValidationError {
                error: "value violates upper limit constraint"
            }
            .into()
        );
    }

    #[test]
    fn validate_multiple_of() {
        let schema = SchemaReal::try_from(&load_simple(
            r#"
                type: real
                multipleOf: 3.0
            "#,
        ))
        .unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("10.0"))
                .unwrap_err(),
            SchemaErrorKind::ValidationError {
                error: "value must be a multiple of the multipleOf field"
            }
            .into()
        );
    }

    #[test]
    fn validate_multiple_of_success() {
        let schema = SchemaReal::try_from(&load_simple(
            r#"
                type: real
                multipleOf: 18.5
            "#,
        ))
        .unwrap();

        schema
            .validate(&Context::default(), &load_simple("314.5"))
            .unwrap();
    }

    #[test]
    fn validate_array() {
        let schema = SchemaReal::default();

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
                expected: "real",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_hash() {
        let schema = SchemaReal::default();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("hello: world"))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "real",
                actual: "hash"
            }
            .into()
        );
    }
}
