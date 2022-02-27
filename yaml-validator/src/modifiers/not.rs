use crate::errors::{SchemaError, ValidationError, ValidationErrorKind};
use crate::utils::YamlUtils;
use crate::{Context, PropertyType, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug)]
pub(crate) struct SchemaNot<'schema> {
    item: Box<PropertyType<'schema>>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaNot<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&["not"], &[])?;

        // I'm using Option::from here because I don't actually want to transform
        // the resulting yaml object into a specific type, but need the yaml itself
        // to be passed into PropertyType::try_from
        yaml.lookup("not", "yaml", Option::from)
            .map_err(SchemaError::from)
            .map(|inner| {
                yaml.lookup("not", "hash", Yaml::as_hash)
                    .map_err(SchemaError::from)
                    .map_err(SchemaError::add_path_name("not"))?;
                Ok(SchemaNot {
                    item: Box::new(
                        PropertyType::try_from(inner).map_err(SchemaError::add_path_name("not"))?,
                    ),
                })
            })?
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaNot<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), ValidationError<'yaml>> {
        match self.item.validate(ctx, yaml) {
            Err(_) => Ok(()),
            Ok(_) => Err(ValidationErrorKind::ValidationError {
                error: "validation inversion failed because inner result matched",
            }
            .with_path_name("not")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{utils::load_simple, SchemaErrorKind};

    #[test]
    fn not_from_yaml() {
        SchemaNot::try_from(&load_simple(
            r#"
            not:
              type: integer
        "#,
        ))
        .unwrap();

        assert_eq!(
            SchemaNot::try_from(&load_simple(
                r#"
                note:
                  type: integer
            "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::Multiple {
                errors: vec![
                    SchemaErrorKind::FieldMissing { field: "not" }.into(),
                    SchemaErrorKind::ExtraField { field: "note" }.into(),
                ]
            }
            .into()
        )
    }

    #[test]
    fn extra_fields() {
        assert_eq!(
            SchemaNot::try_from(&load_simple(
                r#"
                not:
                  type: hello
                extra: extra field test
            "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::ExtraField { field: "extra" }.into(),
        );
    }

    #[test]
    fn validate_inversion_failure() {
        let yaml = load_simple(
            r#"
            not:
              type: integer
            "#,
        );
        let schema = SchemaNot::try_from(&yaml).unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("20"))
                .unwrap_err(),
            ValidationErrorKind::ValidationError {
                error: "validation inversion failed because inner result matched"
            }
            .with_path_name("not")
        );
    }

    #[test]
    fn validate_inversion_success() {
        let yaml = load_simple(
            r#"
            not:
              type: integer
            "#,
        );
        let schema = SchemaNot::try_from(&yaml).unwrap();

        schema
            .validate(&Context::default(), &load_simple("hello world"))
            .unwrap();
    }

    #[test]
    fn validate_double_inversion() {
        let yaml = load_simple(
            r#"
            not:
              not:
                type: integer
            "#,
        );
        let schema = SchemaNot::try_from(&yaml).unwrap();

        schema
            .validate(&Context::default(), &load_simple("20"))
            .unwrap();
    }
}
