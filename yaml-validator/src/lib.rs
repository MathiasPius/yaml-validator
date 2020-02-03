use std::collections::BTreeMap;
use std::convert::TryFrom;
pub use yaml_rust::ScanError;
pub use yaml_rust::{Yaml, YamlLoader};

mod error;
#[cfg(test)]
mod tests;
mod utils;

pub use error::SchemaError;
use error::{add_path_index, add_path_name, optional, SchemaErrorKind};

use utils::YamlUtils;

pub trait Validate<'yaml, 'schema: 'yaml> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>>;
}

#[derive(Debug, Default)]
pub struct Context<'schema> {
    schemas: BTreeMap<&'schema str, Schema<'schema>>,
}

#[derive(Debug)]
pub struct Schema<'schema> {
    uri: &'schema str,
    schema: PropertyType<'schema>,
}

#[derive(Debug, Default)]
struct SchemaObject<'schema> {
    items: BTreeMap<&'schema str, PropertyType<'schema>>,
}

#[derive(Debug, Default)]
struct SchemaArray<'schema> {
    items: Option<Box<PropertyType<'schema>>>,
}

#[derive(Debug, Default)]
struct SchemaString {}

#[derive(Debug, Default)]
struct SchemaInteger {}

#[derive(Debug, Default)]
struct SchemaReference<'schema> {
    uri: &'schema str,
}

#[derive(Debug)]
enum PropertyType<'schema> {
    Object(SchemaObject<'schema>),
    Array(SchemaArray<'schema>),
    String(SchemaString),
    Integer(SchemaInteger),
    Reference(SchemaReference<'schema>),
}

impl<'schema> Context<'schema> {
    pub fn get_schema(&self, uri: &str) -> Option<&Schema<'schema>> {
        self.schemas.get(uri)
    }
}

impl<'schema> TryFrom<&'schema Vec<Yaml>> for Context<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(documents: &'schema Vec<Yaml>) -> Result<Self, Self::Error> {
        let (schemas, errs): (Vec<_>, Vec<_>) = documents
            .iter()
            .map(Schema::try_from)
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

        Ok(Context {
            schemas: schemas
                .into_iter()
                .map(Result::unwrap)
                .map(|schema| (schema.uri, schema))
                .collect(),
        })
    }
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaObject<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&["items"], &["type"])?;

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

        Ok(SchemaObject {
            items: items.into_iter().map(Result::unwrap).collect(),
        })
    }
}

impl<'schema> TryFrom<&'schema Yaml> for Schema<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&["uri", "schema"], &[])?;

        let uri = yaml.lookup("uri", "string", Yaml::as_str)?;
        let schema = PropertyType::try_from(yaml.lookup("schema", "yaml", Option::from)?)?;

        Ok(Schema { uri, schema })
    }
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaArray<'schema> {
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

                Ok(SchemaArray {
                    items: Some(Box::new(
                        PropertyType::try_from(inner).map_err(add_path_name("items"))?,
                    )),
                })
            })
            .or_else(optional(Ok(SchemaArray { items: None })))?
    }
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaString {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&[], &["type"])?;

        Ok(SchemaString {})
    }
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaInteger {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&[], &["type"])?;

        Ok(SchemaInteger {})
    }
}

impl<'schema> TryFrom<&'schema Yaml> for PropertyType<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        let reference = yaml
            .lookup("$ref", "string", Yaml::as_str)
            .map(Option::from)
            .or_else(optional(None))?;

        if let Some(uri) = reference {
            return Ok(PropertyType::Reference(SchemaReference { uri }));
        }

        let typename = yaml.lookup("type", "string", Yaml::as_str)?;

        match typename {
            "object" => Ok(PropertyType::Object(SchemaObject::try_from(yaml)?)),
            "string" => Ok(PropertyType::String(SchemaString::try_from(yaml)?)),
            "integer" => Ok(PropertyType::Integer(SchemaInteger::try_from(yaml)?)),
            "array" => Ok(PropertyType::Array(SchemaArray::try_from(yaml)?)),
            unknown_type => Err(SchemaErrorKind::UnknownType { unknown_type }.into()),
        }
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for Schema<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        self.schema.validate(ctx, yaml)
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaReference<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        if let Some(schema) = ctx.get_schema(self.uri) {
            schema.validate(ctx, yaml)
        } else {
            Err(SchemaErrorKind::UnknownSchema { uri: self.uri }.into())
        }
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaString {
    fn validate(
        &self,
        _: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        yaml.as_type("string", Yaml::as_str).and_then(|_| Ok(()))
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaInteger {
    fn validate(
        &self,
        _: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        yaml.as_type("integer", Yaml::as_i64).and_then(|_| Ok(()))
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
        yaml.strict_contents(&items, &[])?;

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

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaArray<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        let items = yaml.as_type("array", Yaml::as_vec)?;

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

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for PropertyType<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        match self {
            PropertyType::Integer(p) => p.validate(ctx, yaml),
            PropertyType::String(p) => p.validate(ctx, yaml),
            PropertyType::Object(p) => p.validate(ctx, yaml),
            PropertyType::Array(p) => p.validate(ctx, yaml),
            PropertyType::Reference(p) => p.validate(ctx, yaml),
        }
    }
}
