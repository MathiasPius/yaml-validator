use crate::errors::{schema::condense_errors, SchemaError, SchemaErrorKind};
use crate::utils::YamlUtils;
use crate::{Context, PropertyType, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug)]
pub(crate) struct SchemaOneOf<'schema> {
    items: Vec<PropertyType<'schema>>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaOneOf<'schema> {
    type Error = SchemaError<'schema>;

    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&["oneOf"], &[])?;
        let (items, errs): (Vec<_>, Vec<_>) = yaml
            .lookup("oneOf", "array", Yaml::as_vec)?
            .iter()
            .map(|property| {
                PropertyType::try_from(property).map_err(SchemaError::add_path_name("items"))
            })
            .partition(Result::is_ok);

        condense_errors(&mut errs.into_iter())?;

        if items.is_empty() {
            return Err(SchemaErrorKind::MalformedField {
                error: "oneOf modifier requires an array of schemas to validate against".to_owned(),
            }
            .with_path_name("oneOf"));
        }

        Ok(SchemaOneOf {
            items: items.into_iter().map(Result::unwrap).collect(),
        })
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaOneOf<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        let (valid, errs): (Vec<_>, Vec<_>) = self
            .items
            .iter()
            .enumerate()
            .map(|(id, schema)| schema.validate(ctx, yaml).map(|valid| (valid, id)))
            .partition(Result::is_ok);

        match valid.len() {
            0 => {
                // If none of the options matched, return the errors from ALL the arms
                Err(condense_errors(&mut errs.into_iter()).unwrap_err())
            }
            1 => Ok(()),
            _ => {
                // Generate an 'error' for each of the arms that validated correctly, using their index. in the oneOf array
                Err(SchemaErrorKind::Multiple {
                    errors: valid
                        .into_iter()
                        .map(Result::unwrap)
                        .map(|(_, id)| {
                            SchemaErrorKind::ValidationError {
                                error: "multiple branches of oneOf validated successfully. oneOf must only contain a single valid branch",
                            }
                            .with_path_index(id)
                        })
                        .collect(),
                }
                .with_path_name("oneOf"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::load_simple;

    #[test]
    fn one_of_from_yaml() {
        SchemaOneOf::try_from(&load_simple(
            r#"
            oneOf:
              - type: integer
              - type: string
        "#,
        ))
        .unwrap();

        assert_eq!(
            SchemaOneOf::try_from(&load_simple(
                r#"
                oneOff:
                  - type: integer
            "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::Multiple {
                errors: vec![
                    SchemaErrorKind::FieldMissing { field: "oneOf" }.into(),
                    SchemaErrorKind::ExtraField { field: "oneOff" }.into(),
                ]
            }
            .into()
        )
    }

    #[test]
    fn validate_unit_case() {
        let yaml = load_simple(
            r#"
            oneOf:
              - type: integer
            "#,
        );
        let schema = SchemaOneOf::try_from(&yaml).unwrap();

        schema
            .validate(&Context::default(), &load_simple("10"))
            .unwrap();
    }

    #[test]
    fn validate_multiple_valid() {
        assert_eq!(
            SchemaOneOf::try_from(&load_simple(
                r#"
                oneOf:
                  - type: integer
                  - type: integer
                "#,
            ))
            .unwrap()
            .validate(&Context::default(), &load_simple("10"))
            .unwrap_err(),
            SchemaErrorKind::Multiple { errors: vec![
                SchemaErrorKind::ValidationError { error: "multiple branches of oneOf validated successfully. oneOf must only contain a single valid branch"}.with_path_index(0),
                SchemaErrorKind::ValidationError { error: "multiple branches of oneOf validated successfully. oneOf must only contain a single valid branch"}.with_path_index(1),
            ]}.with_path_name("oneOf")
        )
    }

    #[test]
    fn validate_multiple_possible() {
        SchemaOneOf::try_from(&load_simple(
            r#"
                oneOf:
                  - type: integer
                  - type: string
                "#,
        ))
        .unwrap()
        .validate(&Context::default(), &load_simple("10"))
        .unwrap();
    }

    #[test]
    fn validate_complex_subvalidators() {
        let yaml = load_simple(
            r#"
                oneOf:
                  - type: string
                    minLength: 10
                  - type: string
                    maxLength: 10
                "#,
        );

        let schema = SchemaOneOf::try_from(&yaml).unwrap();

        // Validate against a 11-character long string
        schema
            .validate(&Context::default(), &load_simple("hello world"))
            .unwrap();

        // Validate against a 9-character long string
        schema
            .validate(&Context::default(), &load_simple("hello you"))
            .unwrap();

        // Validate against a 10-character long string, causing overlap!
        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("hello you!"))
                .unwrap_err(),
            SchemaErrorKind::Multiple { errors: vec![
                SchemaErrorKind::ValidationError { error: "multiple branches of oneOf validated successfully. oneOf must only contain a single valid branch"}.with_path_index(0),
                SchemaErrorKind::ValidationError { error: "multiple branches of oneOf validated successfully. oneOf must only contain a single valid branch"}.with_path_index(1),
            ]}.with_path_name("oneOf")
        );
    }
}
