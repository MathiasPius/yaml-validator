use crate::error::{add_path_index, add_path_name, optional, SchemaError, SchemaErrorKind};
use crate::utils::{try_into_usize, YamlUtils};
use crate::{Context, PropertyType, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug, Default)]
pub(crate) struct SchemaArray<'schema> {
    items: Option<Box<PropertyType<'schema>>>,
    min_items: Option<usize>,
    max_items: Option<usize>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaArray<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(
            &[],
            &["type", "items", "maxItems", "minItems", "uniqueItems"],
        )?;

        let min_items = yaml
            .lookup("minItems", "integer", Yaml::as_i64)
            .and_then(try_into_usize)
            .map_err(add_path_name("minItems"))
            .map(Option::from)
            .or_else(optional(None))?;

        let max_items = yaml
            .lookup("maxItems", "integer", Yaml::as_i64)
            .and_then(try_into_usize)
            .map_err(add_path_name("maxItems"))
            .map(Option::from)
            .or_else(optional(None))?;

        // I'm using Option::from here because I don't actually want to transform
        // the resulting yaml object into a specific type, but need the yaml itself
        // to be passed into PropertyType::try_from
        yaml.lookup("items", "yaml", Option::from)
            .map(|inner| {
                yaml.lookup("items", "hash", Yaml::as_hash)
                    .map_err(add_path_name("items"))?;

                Ok(SchemaArray {
                    items: Some(Box::new(
                        PropertyType::try_from(inner).map_err(add_path_name("items"))?,
                    )),
                    min_items,
                    max_items,
                })
            })
            .or_else(optional(Ok(SchemaArray {
                items: None,
                min_items,
                max_items,
            })))?
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaArray<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        let items = yaml.as_type("array", Yaml::as_vec)?;

        if let Some(min_items) = &self.min_items {
            if items.len() < *min_items {
                return Err(SchemaErrorKind::ValidationError {
                    error: "array contains fewer than minItems items",
                }
                .into());
            }
        }

        if let Some(max_items) = &self.min_items {
            if items.len() > *max_items {
                return Err(SchemaErrorKind::ValidationError {
                    error: "array contains more than maxItems items",
                }
                .into());
            }
        }

        if let Some(schema) = &self.items {
            let mut errors: Vec<SchemaError<'yaml>> = items
                .iter()
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
    use crate::utils::load_simple;
    use crate::SchemaArray;

    #[cfg(feature = "smallvec")]
    use smallvec::smallvec;

    #[test]
    fn from_yaml() {
        SchemaArray::try_from(&load_simple(
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
            SchemaArray::try_from(&load_simple(
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
            .with_path(path!["items"])
            .into(),
        );
    }

    #[test]
    fn from_string() {
        assert_eq!(
            SchemaArray::try_from(&load_simple("world")).unwrap_err(),
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
            SchemaArray::try_from(&load_simple("10")).unwrap_err(),
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
            SchemaArray::try_from(&load_simple(
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
        let schema = SchemaArray::default();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("hello world"))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "array",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn validate_integer() {
        let schema = SchemaArray::default();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("10"))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "array",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn validate_untyped_array() {
        SchemaArray::default()
            .validate(
                &Context::default(),
                &load_simple(
                    r#"
                - abc
                - 123
            "#,
                ),
            )
            .unwrap();
    }

    #[test]
    fn validate_typed_array() {
        let yaml = load_simple(
            r#"
        items:
          type: integer
        "#,
        );

        assert_eq!(
            SchemaArray::try_from(&yaml)
                .unwrap()
                .validate(
                    &Context::default(),
                    &load_simple(
                        r#"
                - abc
                - 1
                - 2
                - 3
                - def
                - 4
                - hello: world
            "#,
                    )
                )
                .unwrap_err(),
            SchemaErrorKind::Multiple {
                errors: vec![
                    SchemaErrorKind::WrongType {
                        expected: "integer",
                        actual: "string"
                    }
                    .with_path(path![0]),
                    SchemaErrorKind::WrongType {
                        expected: "integer",
                        actual: "string"
                    }
                    .with_path(path![4]),
                    SchemaErrorKind::WrongType {
                        expected: "integer",
                        actual: "hash"
                    }
                    .with_path(path![6])
                ]
            }
            .into()
        );
    }

    #[test]
    fn validate_hash() {
        let schema = SchemaArray::default();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("hello: world"))
                .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "array",
                actual: "hash"
            }
            .into()
        );
    }
}
