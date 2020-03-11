use crate::error::{add_path_name, SchemaError, SchemaErrorKind};
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

        if !errs.is_empty() {
            let mut errors: Vec<SchemaError<'schema>> =
                errs.into_iter().map(Result::unwrap_err).collect();
            if errors.len() == 1 {
                return Err(errors.pop().unwrap());
            } else {
                return Err(SchemaErrorKind::Multiple { errors }.into());
            }
        }

        let required: Option<Result<(Vec<_>, Vec<_>), _>> = yaml.lookup("required", "array", Option::from)
            .map(|_| {
                Ok(yaml.lookup("required", "array", Yaml::as_vec)
                    .map_err(add_path_name("required"))?
                    .into_iter()
                    .map(|item| {
                        item.as_type("string", Yaml::as_str)
                            .map_err(add_path_name("required"))
                    })
                    .partition(Result::is_ok))
            })
            .ok();

        let required = match required {
            Some(Ok((required, errs))) => {
                if !errs.is_empty() {
                    let errors: Vec<SchemaError<'schema>> =
                        errs.into_iter().map(Result::unwrap_err).collect();
                    return Err(SchemaErrorKind::Multiple { errors }.into());
                } else {
                    Some(required)
                }
            },
            Some(Err(err)) => {
                return Err(err);
            },
            None => None,
        };

        Ok(SchemaObject {
            items: items.into_iter().map(Result::unwrap).collect(),
            required: required.map(|req| req.into_iter().map(Result::unwrap).collect()),
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

        let keys = self.items.keys().copied();
        let (required, optional): (Vec<_>, Vec<_>) = match self.required {
            Some(ref required) => {
                        keys
                        .partition(|key| {
                            required.contains(&key)
                        })
                }
            _ => (vec![], keys.collect()),
        };
        yaml.strict_contents(&required, &optional)?;

        let mut errors: Vec<SchemaError<'yaml>> = self
            .items
            .iter()
            .map(|(name, schema_item)| {
                let item: &Yaml = yaml
                    .lookup(name, "yaml", Option::from)
                    .map_err(add_path_name(name))?;

                schema_item
                    .validate(ctx, item)
                    .map_err(add_path_name(name))?;
                Ok(())
            })
            .filter_map(Result::err)
            .collect();

        if errors.is_empty() {
            Ok(())
        } else if errors.len() == 1 {
            Err(errors.pop().unwrap())
        } else {
            Err(SchemaErrorKind::Multiple { errors }.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::load_simple;
    use crate::SchemaObject;

    #[cfg(feature = "smallvec")]
    use smallvec::smallvec;

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
                    .with_path(path!["hello"]),
                    SchemaErrorKind::WrongType {
                        expected: "integer",
                        actual: "string"
                    }
                    .with_path(path!["world"])
                ]
            }
            .into()
        );
    }
}

#[cfg(test)]
mod test_required_field {
    use super::*;
    use crate::utils::load_simple;
    use crate::SchemaObject;

    #[cfg(feature = "smallvec")]
    use smallvec::smallvec;

    #[test]
    fn from_yaml() {
        SchemaObject::try_from(&load_simple(
            r#"
            items:
              something:
                type: string
            required: []
        "#,
        ))
        .unwrap();
    }

macro_rules! add_creation_fail_tests {
    ($(name: $name:ident, value: $value:literal, ty: $ty:expr),+) => {
        $(
            #[test]
            fn $name() {
                let data = format!(r#"
                        items:
                          something:
                            type: string
                        required: {}
                "#, $value);
                assert_eq!(
                    SchemaObject::try_from(&load_simple(&data))
                    .unwrap_err(),
                    SchemaErrorKind::WrongType {
                        expected: "array",
                        actual: $ty
                    }
                    .with_path(path!["required"]),
                );
            }
        )+
    };
}
    add_creation_fail_tests!(
        name: required_is_string, value: "some string", ty: "string",
        name: required_is_int, value: 123, ty: "integer",
        name: required_is_real, value: 1.23, ty: "real",
        name: required_is_hash, value: "{ name: value }", ty: "hash"
    );

}
