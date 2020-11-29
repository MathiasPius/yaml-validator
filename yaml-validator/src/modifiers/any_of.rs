use crate::error::{add_path_name, condense_errors, SchemaError, SchemaErrorKind};
use crate::utils::YamlUtils;
use crate::{Context, PropertyType, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug)]
pub(crate) struct SchemaAnyOf<'schema> {
    items: Vec<PropertyType<'schema>>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaAnyOf<'schema> {
    type Error = SchemaError<'schema>;

    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&["anyOf"], &[])?;
        let (items, errs): (Vec<_>, Vec<_>) = yaml
            .lookup("anyOf", "array", Yaml::as_vec)?
            .iter()
            .map(|property| PropertyType::try_from(property).map_err(add_path_name("items")))
            .partition(Result::is_ok);

        condense_errors(&mut errs.into_iter())?;

        if items.is_empty() {
            return Err(SchemaErrorKind::MalformedField {
                error: "anyOf modifier requires an array of schemas to validate against".to_owned(),
            }
            .with_path_name("anyOf"));
        }

        Ok(SchemaAnyOf {
            items: items.into_iter().map(Result::unwrap).collect(),
        })
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaAnyOf<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        let (valid, errs): (Vec<_>, Vec<_>) = self
            .items
            .iter()
            .map(|schema| schema.validate(ctx, yaml).map_err(add_path_name("anyOf")))
            .partition(Result::is_ok);

        if valid.is_empty() {
            Err(condense_errors(&mut errs.into_iter()).unwrap_err())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::load_simple;

    #[test]
    fn one_of_from_yaml() {
        SchemaAnyOf::try_from(&load_simple(
            r#"
            anyOf:
              - type: integer
              - type: string
        "#,
        ))
        .unwrap();

        assert_eq!(
            SchemaAnyOf::try_from(&load_simple(
                r#"
                anyOff:
                  - type: integer
            "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::Multiple {
                errors: vec![
                    SchemaErrorKind::FieldMissing { field: "anyOf" }.into(),
                    SchemaErrorKind::ExtraField { field: "anyOff" }.into(),
                ]
            }
            .into()
        )
    }

    #[test]
    fn validate_unit_case() {
        let yaml = load_simple(
            r#"
            anyOf:
              - type: integer
            "#,
        );
        let schema = SchemaAnyOf::try_from(&yaml).unwrap();

        schema
            .validate(&Context::default(), &load_simple("10"))
            .unwrap();
    }

    #[test]
    fn validate_multiple_valid() {
        let yaml = load_simple(
            r#"
            anyOf:
              - type: integer
              - type: real
            "#,
        );

        let schema = SchemaAnyOf::try_from(&yaml).unwrap();

        schema
            .validate(&Context::default(), &load_simple("10"))
            .unwrap();

        schema
            .validate(&Context::default(), &load_simple("10.0"))
            .unwrap();
    }

    #[test]
    fn validate_complex_subvalidators() {
        let yaml = load_simple(
            r#"
                anyOf:
                  - type: string
                    minLength: 10
                  - type: string
                    maxLength: 10
                "#,
        );

        let schema = SchemaAnyOf::try_from(&yaml).unwrap();

        // Validate against a 9-character long string
        schema
            .validate(&Context::default(), &load_simple("hello you"))
            .unwrap();

        // Validate against a 10-character long string
        schema
            .validate(&Context::default(), &load_simple("hello you"))
            .unwrap();

        // Validate against a 11-character long string
        schema
            .validate(&Context::default(), &load_simple("hello world"))
            .unwrap();
    }
}
