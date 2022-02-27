use crate::errors::{schema::schema_optional, SchemaError};
use crate::errors::{ValidationError, ValidationErrorKind};
use crate::utils::YamlUtils;
use crate::{Context, PropertyType, SchemaErrorKind, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug, Default)]
pub(crate) struct SchemaHash<'schema> {
    items: Option<Box<PropertyType<'schema>>>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaHash<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&[], &["items", "type"])
            .map_err(SchemaErrorKind::from)?;

        // I'm using Option::from here because I don't actually want to transform
        // the resulting yaml object into a specific type, but need the yaml itself
        // to be passed into PropertyType::try_from
        yaml.lookup("items", "yaml", Option::from)
            .map_err(SchemaErrorKind::from)
            .map_err(SchemaError::from)
            .map(|inner| {
                yaml.lookup("items", "hash", Yaml::as_hash)
                    .map_err(SchemaErrorKind::from)
                    .map_err(SchemaError::from)
                    .map_err(SchemaError::add_path_name("items"))?;

                Ok(SchemaHash {
                    items: Some(Box::new(
                        PropertyType::try_from(inner)
                            .map_err(SchemaError::add_path_name("items"))?,
                    )),
                })
            })
            .or_else(schema_optional(Ok(SchemaHash { items: None })))?
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaHash<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), ValidationError<'yaml>> {
        let items = yaml
            .as_type("hash", Yaml::as_hash)
            .map_err(ValidationErrorKind::from)?;

        if let Some(schema) = &self.items {
            let mut errors: Vec<ValidationError<'yaml>> = items
                .values()
                .enumerate()
                .map(|(i, item)| {
                    schema
                        .validate(ctx, item)
                        .map_err(ValidationError::add_path_index(i))
                })
                .filter(Result::is_err)
                .map(Result::unwrap_err)
                .collect();

            return if errors.is_empty() {
                Ok(())
            } else if errors.len() == 1 {
                Err(errors.pop().unwrap())
            } else {
                Err(ValidationErrorKind::Multiple { errors }.into())
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::load_simple;
    use crate::{SchemaErrorKind, SchemaHash};

    #[test]
    fn from_yaml() {
        SchemaHash::try_from(&load_simple(
            r#"
            items:
              type: string
        "#,
        ))
        .unwrap();
    }

    #[test]
    fn malformed_items() {
        assert_eq!(
            SchemaHash::try_from(&load_simple(
                r#"
            items:
              - type: string
        "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "array"
            }
            .with_path_name("items")
            .into(),
        );
    }

    #[test]
    fn from_string() {
        assert_eq!(
            SchemaHash::try_from(&load_simple("world")).unwrap_err(),
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
            SchemaHash::try_from(&load_simple("10")).unwrap_err(),
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
            SchemaHash::try_from(&load_simple(
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
        let schema = SchemaHash::default();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("hello world"))
                .unwrap_err(),
            ValidationErrorKind::WrongType {
                expected: "hash",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn validate_integer() {
        let schema = SchemaHash::default();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("10"))
                .unwrap_err(),
            ValidationErrorKind::WrongType {
                expected: "hash",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn validate_untyped_hash() {
        let schema = SchemaHash::default();

        schema
            .validate(&Context::default(), &load_simple("hello: world"))
            .unwrap();
    }

    #[test]
    fn validate_typed_hash() {
        let yaml = load_simple("type: hash\nitems:\n  type: integer");
        let schema = SchemaHash::try_from(&yaml).unwrap();

        schema
            .validate(&Context::default(), &load_simple("hello: 20"))
            .unwrap();
    }

    #[test]
    fn validate_invalid_typed_hash() {
        let yaml = load_simple("type: hash\nitems:\n  type: integer");
        let schema = SchemaHash::try_from(&yaml).unwrap();

        assert_eq!(
            schema
                .validate(
                    &Context::default(),
                    &load_simple("hello: 20\nworld: clearly a string")
                )
                .unwrap_err(),
            ValidationErrorKind::WrongType {
                expected: "integer",
                actual: "string"
            }
            .with_path_index(1)
        );
    }
}
