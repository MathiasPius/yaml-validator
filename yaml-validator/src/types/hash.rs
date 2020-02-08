use crate::error::{add_path_index, add_path_name, optional, SchemaError, SchemaErrorKind};
use crate::utils::YamlUtils;
use crate::{Context, PropertyType, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug, Default)]
pub(crate) struct SchemaHash<'schema> {
    items: Option<Box<PropertyType<'schema>>>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaHash<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&[], &["items", "type"])?;

        // I'm using Option::from here because I don't actually want to transform
        // the resulting yaml object into a specific type, but need the yaml itself
        // to be passed into PropertyType::try_from
        yaml.lookup("items", "yaml", Option::from)
            .map(|inner| {
                yaml.lookup("items", "hash", Yaml::as_hash)
                    .map_err(add_path_name("items"))?;

                Ok(SchemaHash {
                    items: Some(Box::new(
                        PropertyType::try_from(inner).map_err(add_path_name("items"))?,
                    )),
                })
            })
            .or_else(optional(Ok(SchemaHash { items: None })))?
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaHash<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        let items = yaml.as_type("hash", Yaml::as_hash)?;

        if let Some(schema) = &self.items {
            let mut errors: Vec<SchemaError<'yaml>> = items
                .values()
                .enumerate()
                .map(|(i, item)| schema.validate(ctx, item).map_err(add_path_index(i)))
                .filter(Result::is_err)
                .map(Result::unwrap_err)
                .collect();

            return if errors.is_empty() {
                Ok(())
            } else if errors.len() == 1 {
                Err(errors.pop().unwrap())
            } else {
                Err(SchemaErrorKind::Multiple { errors }.into())
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PathSegment;
    use crate::utils::load_simple;
    use crate::SchemaHash;

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
            .with_path(vec![PathSegment::Name("items")])
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
            SchemaErrorKind::WrongType {
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
            SchemaErrorKind::WrongType {
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
            SchemaErrorKind::WrongType {
                expected: "integer",
                actual: "string"
            }
            .with_path(vec![PathSegment::Index(1)])
        )
    }
}
