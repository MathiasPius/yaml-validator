use crate::error::{
    add_path_index, add_path_name, condense_errors, optional, SchemaError, SchemaErrorKind,
};
use crate::utils::{try_into_usize, YamlUtils};
use crate::{Context, PropertyType, Validate};
use std::collections::HashSet;
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug, Default)]
pub(crate) struct SchemaArray<'schema> {
    items: Option<Box<PropertyType<'schema>>>,
    min_items: Option<usize>,
    max_items: Option<usize>,
    unique_items: bool,
    contains: Option<Box<PropertyType<'schema>>>,
    min_contains: Option<usize>,
    max_contains: Option<usize>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaArray<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(
            &[],
            &[
                "type",
                "items",
                "maxItems",
                "minItems",
                "uniqueItems",
                "contains",
                "minContains",
                "maxContains",
            ],
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

        let unique_items = yaml
            .lookup("uniqueItems", "bool", Yaml::as_bool)
            .map_err(add_path_name("uniqueItems"))
            .map(Option::from)
            .or_else(optional(None))?
            .unwrap_or(false);

        if let (Some(min_items), Some(max_items)) = (min_items, max_items) {
            if min_items > max_items {
                return Err(SchemaErrorKind::MalformedField {
                    error: "minItems cannot be greater than maxItems".into(),
                }
                .into());
            }
        }

        let items = yaml
            .lookup("items", "yaml", Option::from)
            .map_err(add_path_name("items"))
            .map(Option::from)
            .or_else(optional(None))?
            .map(PropertyType::try_from)
            .transpose()
            .map_err(add_path_name("items"))?
            .map(Box::new);

        let contains = yaml
            .lookup("contains", "yaml", Option::from)
            .map_err(add_path_name("contains"))
            .map(Option::from)
            .or_else(optional(None))?
            .map(PropertyType::try_from)
            .transpose()
            .map_err(add_path_name("contains"))?
            .map(Box::new);

        let min_contains = yaml
            .lookup("minContains", "integer", Yaml::as_i64)
            .and_then(try_into_usize)
            .map_err(add_path_name("minContains"))
            .map(Option::from)
            .or_else(optional(None))?;

        let max_contains = yaml
            .lookup("maxContains", "integer", Yaml::as_i64)
            .and_then(try_into_usize)
            .map_err(add_path_name("maxContains"))
            .map(Option::from)
            .or_else(optional(None))?;

        // This does not seem like the nicest way to do this...
        match (&contains, &min_contains, &max_contains) {
            (None   , Some(_)  , None     ) => Err(SchemaErrorKind::MalformedField {
                error: "minContains requires 'contains' to specify a schema to validate against".into()
            }.into()),
            (None   , None     , Some(_)  ) => Err(SchemaErrorKind::MalformedField {
                error: "maxContains requires 'contains' to specify a schema to validate against".into()
            }.into()),
            (None   , Some(_)  , Some(_)  ) => Err(SchemaErrorKind::MalformedField {
                error: "minContains and maxContains requires 'contains' to specify a schema to validate against".into()
            }.into()),
            (Some(_), Some(min), Some(max)) => {
                if min > max {
                    Err(SchemaErrorKind::MalformedField {
                        error: "minContains cannot be greater than maxContains".into()
                    }.into())
                } else {
                    Ok(())
                }
            },
            _ => Ok(())
        }?;

        Ok(SchemaArray {
            items,
            min_items,
            max_items,
            unique_items,
            contains,
            min_contains,
            max_contains,
        })
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

        if let Some(max_items) = &self.max_items {
            if items.len() > *max_items {
                return Err(SchemaErrorKind::ValidationError {
                    error: "array contains more than maxItems items",
                }
                .into());
            }
        }

        if self.unique_items {
            let mut set = HashSet::new();
            for (i, item) in items.iter().enumerate() {
                if set.contains(item) {
                    return Err(SchemaErrorKind::ValidationError {
                        error: "array contains duplicate key",
                    }
                    .with_path_index(i));
                }

                set.insert(item);
            }
        }

        if let Some(contains) = &self.contains {
            let contained = items
                .iter()
                .enumerate()
                .map(|(i, item)| contains.validate(ctx, item).map_err(add_path_index(i)))
                .filter(Result::is_ok)
                .count();

            if let Some(min) = self.min_contains {
                if contained < min {
                    return Err(SchemaErrorKind::ValidationError {
                        error:
                            "fewer than minContains items validated against schema in 'contains'",
                    }
                    .into());
                }
            } else if contained < 1 {
                return Err(SchemaErrorKind::ValidationError {
                    error: "at least one item in the array must match the 'contains' schema",
                }
                .into());
            }

            if let Some(max) = self.max_contains {
                if contained > max {
                    return Err(SchemaErrorKind::ValidationError {
                        error: "more than minContains items validated against schema in 'contains'",
                    }
                    .into());
                }
            }
        };

        if let Some(schema) = &self.items {
            let errors = items
                .iter()
                .enumerate()
                .map(|(i, item)| schema.validate(ctx, item).map_err(add_path_index(i)));

            condense_errors(&mut errors.into_iter())?;
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
    fn with_invalid_range() {
        SchemaArray::try_from(&load_simple(
            r#"
            minItems: 10
            maxItems: 10
        "#,
        ))
        .unwrap();

        assert_eq!(
            SchemaArray::try_from(&load_simple(
                r#"
                minItems: 10
                maxItems: 5
            "#
            ))
            .unwrap_err(),
            SchemaErrorKind::MalformedField {
                error: "minItems cannot be greater than maxItems".into()
            }
            .into()
        );
    }

    #[test]
    fn with_contains() {
        SchemaArray::try_from(&load_simple(
            r#"
            minContains: 5
            maxContains: 10
            contains:
              type: integer
        "#,
        ))
        .unwrap();

        SchemaArray::try_from(&load_simple(
            r#"
            minContains: 5
            contains:
              type: integer
        "#,
        ))
        .unwrap();

        assert_eq!(
            SchemaArray::try_from(&load_simple(
                r#"
                minContains: 10
                maxContains: 5
            "#
            ))
            .unwrap_err(),
            SchemaErrorKind::MalformedField {
                error: "minContains and maxContains requires 'contains' to specify a schema to validate against".into()
            }
            .into()
        );
    }

    #[test]
    fn with_contain_invalid_range() {
        assert_eq!(
            SchemaArray::try_from(&load_simple(
                r#"
                minContains: 10
                maxContains: 5
                contains:
                  type: integer
            "#
            ))
            .unwrap_err(),
            SchemaErrorKind::MalformedField {
                error: "minContains cannot be greater than maxContains".into()
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
    fn validate_bounded_typed_array() {
        let yaml = load_simple(
            r#"
            minItems: 2
            maxItems: 10
            items:
              type: integer
        "#,
        );

        SchemaArray::try_from(&yaml)
            .unwrap()
            .validate(
                &Context::default(),
                &load_simple(
                    r#"
                    - 1234
                    - 5678
                    - 9876
                "#,
                ),
            )
            .unwrap();
    }

    #[test]
    fn validate_unique_typed_array() {
        let yaml = load_simple(
            r#"
            uniqueItems: true
            items:
              type: integer
        "#,
        );

        SchemaArray::try_from(&yaml)
            .unwrap()
            .validate(
                &Context::default(),
                &load_simple(
                    r#"
                    - 1234
                    - 5678
                    - 9876
                "#,
                ),
            )
            .unwrap();
    }

    #[test]
    fn validate_unique_typed_array_duplicates() {
        let yaml = load_simple(
            r#"
            uniqueItems: true
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
                    - 1234
                    - 5678
                    - 9876
                    - 1234
                    - 8286
                "#,
                    ),
                )
                .unwrap_err(),
            SchemaErrorKind::ValidationError {
                error: "array contains duplicate key"
            }
            .with_path_index(3)
        );
    }

    #[test]
    fn validate_narrow_array() {
        let yaml = load_simple(
            r#"
            minItems: 2
            maxItems: 2
            items:
              type: integer
        "#,
        );

        SchemaArray::try_from(&yaml)
            .unwrap()
            .validate(
                &Context::default(),
                &load_simple(
                    r#"
                    - 1234
                    - 5678
                "#,
                ),
            )
            .unwrap();
    }

    #[test]
    fn validate_too_large_array() {
        assert_eq!(
            SchemaArray::try_from(&load_simple("maxItems: 2"))
                .unwrap()
                .validate(
                    &Context::default(),
                    &load_simple(
                        r#"
                    - 1234
                    - 5678
                    - acbd
                "#,
                    ),
                )
                .unwrap_err(),
            SchemaErrorKind::ValidationError {
                error: "array contains more than maxItems items"
            }
            .into()
        )
    }

    #[test]
    fn validate_too_small_array() {
        assert_eq!(
            SchemaArray::try_from(&load_simple("minItems: 5"))
                .unwrap()
                .validate(
                    &Context::default(),
                    &load_simple(
                        r#"
                    - 1234
                    - 5678
                    - acbd
                "#,
                    ),
                )
                .unwrap_err(),
            SchemaErrorKind::ValidationError {
                error: "array contains fewer than minItems items"
            }
            .into()
        )
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
    fn validate_array_contains() {
        let yaml = load_simple(
            r#"
            minContains: 2
            maxContains: 10
            contains:
              type: integer
              minimum: 5
              maximum: 20
        "#,
        );

        SchemaArray::try_from(&yaml)
            .unwrap()
            .validate(
                &Context::default(),
                &load_simple(
                    r#"
                    - 1234
                    - 5678
                    - 9876
                    - 50
                    - 15
                    - 10
                "#,
                ),
            )
            .unwrap();
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
