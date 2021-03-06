use crate::error::{add_path_name, condense_errors, optional, SchemaError};
use crate::utils::YamlUtils;
use crate::{Context, PropertyType, Validate};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug, Default)]
pub(crate) struct SchemaObject<'schema> {
    items: BTreeMap<&'schema str, PropertyType<'schema>>,
    required: Option<Vec<&'schema str>>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaObject<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&["items"], &["type", "required"])?;

        let items = yaml.lookup("items", "hash", Yaml::as_hash)?;

        let (items, errs): (Vec<_>, Vec<_>) = items
            .iter()
            .map(|property| {
                let name = property.0.as_type("string", Yaml::as_str)?;
                PropertyType::try_from(property.1)
                    .map_err(add_path_name(name))
                    .map_err(add_path_name("items"))
                    .map(|prop| (name, prop))
            })
            .partition(Result::is_ok);

        condense_errors(&mut errs.into_iter())?;

        let required: Option<(Vec<_>, Vec<_>)> = yaml
            .lookup("required", "array", Yaml::as_vec)
            .map_err(add_path_name("required"))
            .map(Option::from)
            .or_else(optional(None))?
            .map(|fields| {
                fields
                    .iter()
                    .map(|field| -> Result<&'schema str, Self::Error> {
                        field.as_type("string", Yaml::as_str)
                    })
                    .partition(Result::is_ok)
            });

        let required = if let Some((required, errs)) = required {
            condense_errors(&mut errs.into_iter())?;
            Some(required.into_iter().map(Result::unwrap).collect())
        } else {
            None
        };

        Ok(SchemaObject {
            items: items.into_iter().map(Result::unwrap).collect(),
            required,
        })
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaObject<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        yaml.as_type("hash", Yaml::as_hash)?;

        let items: Vec<&'schema str> = self.items.keys().copied().collect();
        let required = self.required.as_ref().cloned().unwrap_or_default();
        yaml.strict_contents(&required, &items)?;

        let mut errors = self.items.iter().map(|(name, schema_item)| {
            let item = yaml
                .lookup(name, "yaml", Option::from)
                .map(Option::Some)
                .map_err(add_path_name(name))
                .or_else(optional(None))?;

            if let Some(item) = item {
                schema_item
                    .validate(ctx, item)
                    .map_err(add_path_name(name))?;
            }

            Ok(())
        });

        condense_errors(&mut errors)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::load_simple;
    use crate::{SchemaErrorKind, SchemaObject};

    #[test]
    fn from_yaml() {
        SchemaObject::try_from(&load_simple(
            r#"
            items:
              something:
                type: string
        "#,
        ))
        .unwrap();
    }

    #[test]
    fn extra_fields() {
        assert_eq!(
            SchemaObject::try_from(&load_simple(
                r#"
            items:
              something:
                type: hello
            extra: extra field test
        "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::ExtraField { field: "extra" }.into(),
        );
    }

    #[test]
    fn malformed_items() {
        assert_eq!(
            SchemaObject::try_from(&load_simple(
                r#"
            items:
              hello: world
        "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "string"
            }
            .with_path(path!["hello", "items"]),
        );
    }

    #[test]
    fn multiple_errors() {
        assert_eq!(
            SchemaObject::try_from(&load_simple(
                r#"
            items:
              valid:
                type: string
              error 1:
                type: unknown1
              error 2:
                type: unknown2
        "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::Multiple {
                errors: vec![
                    SchemaErrorKind::UnknownType {
                        unknown_type: "unknown1"
                    }
                    .with_path(path!["error 1", "items"]),
                    SchemaErrorKind::UnknownType {
                        unknown_type: "unknown2"
                    }
                    .with_path(path!["error 2", "items"]),
                ]
            }
            .into()
        );
    }

    #[test]
    fn from_string() {
        assert_eq!(
            SchemaObject::try_from(&load_simple("world")).unwrap_err(),
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
            SchemaObject::try_from(&load_simple("10")).unwrap_err(),
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
            SchemaObject::try_from(&load_simple(
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
        let schema = SchemaObject::default();

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
        let schema = SchemaObject::default();

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
    fn validate_array() {
        let schema = SchemaObject::default();

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
                expected: "hash",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_hash() {
        let yaml = load_simple(
            r#"
            items:
              hello:
                type: string
              world:
                type: integer
            "#,
        );

        let schema = SchemaObject::try_from(&yaml).unwrap();

        schema
            .validate(
                &Context::default(),
                &load_simple(
                    r#"
            hello: world
            world: 20
        "#,
                ),
            )
            .unwrap();
    }

    #[test]
    fn validate_noncompliant() {
        let yaml = load_simple(
            r#"
            items:
              hello:
                type: string
              world:
                type: integer
            "#,
        );

        let schema = SchemaObject::try_from(&yaml).unwrap();

        assert_eq!(
            schema
                .validate(
                    &Context::default(),
                    &load_simple(
                        r#"
            hello: 20
            world: world
        "#,
                    )
                )
                .unwrap_err(),
            SchemaErrorKind::Multiple {
                errors: vec![
                    SchemaErrorKind::WrongType {
                        expected: "string",
                        actual: "integer"
                    }
                    .with_path_name("hello"),
                    SchemaErrorKind::WrongType {
                        expected: "integer",
                        actual: "string"
                    }
                    .with_path_name("world")
                ]
            }
            .into()
        );
    }

    #[test]
    fn validate_optional() {
        let yaml = load_simple(
            r#"
            items:
              hello:
                type: string
              world:
                type: integer
            "#,
        );

        let schema = SchemaObject::try_from(&yaml).unwrap();

        schema
            .validate(
                &Context::default(),
                &load_simple(
                    r#"
                            hello: world
                        "#,
                ),
            )
            .unwrap();
    }

    #[test]
    fn validate_eq() {
        let yaml = load_simple(
            r#"
            items:
              hello:
                type: string
              world:
                type: integer
            "#,
        );

        let schema = SchemaObject::try_from(&yaml).unwrap();

        schema
            .validate(
                &Context::default(),
                &load_simple(
                    r#"
                            hello: world
                        "#,
                ),
            )
            .unwrap();
    }

    #[test]
    fn validate_required() {
        let yaml = load_simple(
            r#"
            items:
              hello:
                type: string
              world:
                type: integer
            required:
                - hello
                - world
            "#,
        );

        let schema = SchemaObject::try_from(&yaml).unwrap();

        assert_eq!(
            schema
                .validate(
                    &Context::default(),
                    &load_simple(
                        r#"
                            hello: world
                        "#,
                    )
                )
                .unwrap_err(),
            SchemaErrorKind::FieldMissing { field: "world" }.into()
        );
    }
}
